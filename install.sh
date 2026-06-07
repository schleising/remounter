#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

HOME_DIR="${HOME}"
USER_ID="$(id -u)"
CARGO_BIN="${HOME_DIR}/.cargo/bin"
INSTALL_BIN="${CARGO_BIN}/remounter"
LAUNCH_AGENTS_DIR="${HOME_DIR}/Library/LaunchAgents"
LOGS_DIR="${HOME_DIR}/Library/Logs"
PLIST_LABEL="com.schleising.remounter"
PLIST_FILENAME="${PLIST_LABEL}.plist"
PLIST_PATH="${LAUNCH_AGENTS_DIR}/${PLIST_FILENAME}"
LOG_OUT="${LOGS_DIR}/remounter.log"
LOG_ERR="${LOGS_DIR}/remounter.err.log"
LAUNCHD_DOMAIN="gui/${USER_ID}"

REMOUNTER_HOST="${REMOUNTER_HOST:-}"
REMOUNTER_SHARES="${REMOUNTER_SHARES:-}"
REMOUNTER_POST_MOUNT_SCRIPT="${REMOUNTER_POST_MOUNT_SCRIPT:-}"

usage() {
    cat <<EOF
Usage: $(basename "$0") --host <hostname> --shares <share1,share2> [--post-mount-script <path>]

Environment variables (used when flags are omitted):
  REMOUNTER_HOST              SMB host to monitor
  REMOUNTER_SHARES            Comma-separated share names
  REMOUNTER_POST_MOUNT_SCRIPT Optional script to run after remounting

Example:
  $(basename "$0") --host nas.local --shares Media,home
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --host)
            REMOUNTER_HOST="${2:-}"
            shift 2
            ;;
        --shares)
            REMOUNTER_SHARES="${2:-}"
            shift 2
            ;;
        --post-mount-script)
            REMOUNTER_POST_MOUNT_SCRIPT="${2:-}"
            shift 2
            ;;
        -h | --help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown argument: $1" >&2
            usage >&2
            exit 1
            ;;
    esac
done

if [[ -z "${REMOUNTER_HOST}" || -z "${REMOUNTER_SHARES}" ]]; then
    echo "Error: --host and --shares are required." >&2
    usage >&2
    exit 1
fi

write_plist() {
    local args=("${INSTALL_BIN}" "${REMOUNTER_HOST}" "${REMOUNTER_SHARES}")
    if [[ -n "${REMOUNTER_POST_MOUNT_SCRIPT}" ]]; then
        args+=(--post-mount-script "${REMOUNTER_POST_MOUNT_SCRIPT}")
    fi

    cat >"${PLIST_PATH}" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
 "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>${PLIST_LABEL}</string>

    <key>ProgramArguments</key>
    <array>
$(printf '        <string>%s</string>\n' "${args[@]}")
    </array>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <true/>

    <key>ProcessType</key>
    <string>Background</string>

    <key>ThrottleInterval</key>
    <integer>10</integer>

    <key>StandardOutPath</key>
    <string>${LOG_OUT}</string>

    <key>StandardErrorPath</key>
    <string>${LOG_ERR}</string>
</dict>
</plist>
EOF
}

echo "Updating Rust dependencies..."
(cd "${SCRIPT_DIR}" && cargo update)

echo "Building and installing remounter to ${INSTALL_BIN}..."
(cd "${SCRIPT_DIR}" && cargo install --path . --force)

mkdir -p "${LAUNCH_AGENTS_DIR}" "${LOGS_DIR}"

if launchctl print "${LAUNCHD_DOMAIN}/${PLIST_LABEL}" &>/dev/null; then
    echo "Stopping existing launch agent..."
    launchctl bootout "${LAUNCHD_DOMAIN}" "${PLIST_PATH}" 2>/dev/null || true
fi

echo "Installing launch agent to ${PLIST_PATH}..."
write_plist

echo "Starting launch agent..."
launchctl bootstrap "${LAUNCHD_DOMAIN}" "${PLIST_PATH}"
launchctl enable "${LAUNCHD_DOMAIN}/${PLIST_LABEL}"
launchctl kickstart -k "${LAUNCHD_DOMAIN}/${PLIST_LABEL}"

echo "Installation complete."
echo "  Binary:       ${INSTALL_BIN}"
echo "  Launch agent: ${PLIST_PATH}"
echo "  Logs:         ${LOG_OUT}"
echo "                ${LOG_ERR}"
