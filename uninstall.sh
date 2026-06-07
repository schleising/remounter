#!/usr/bin/env bash
set -euo pipefail

HOME_DIR="${HOME}"
USER_ID="$(id -u)"
CARGO_BIN="${HOME_DIR}/.cargo/bin"
INSTALL_BIN="${CARGO_BIN}/remounter"
LAUNCH_AGENTS_DIR="${HOME_DIR}/Library/LaunchAgents"
LOGS_DIR="${HOME_DIR}/Library/Logs"
PLIST_LABEL="com.schleising.remounter"
PLIST_PATH="${LAUNCH_AGENTS_DIR}/${PLIST_LABEL}.plist"
LOG_OUT="${LOGS_DIR}/remounter.log"
LOG_ERR="${LOGS_DIR}/remounter.err.log"
LAUNCHD_DOMAIN="gui/${USER_ID}"

if launchctl print "${LAUNCHD_DOMAIN}/${PLIST_LABEL}" &>/dev/null; then
    echo "Stopping launch agent..."
    launchctl bootout "${LAUNCHD_DOMAIN}" "${PLIST_PATH}" 2>/dev/null || true
fi

if [[ -f "${PLIST_PATH}" ]]; then
    echo "Removing launch agent plist..."
    rm -f "${PLIST_PATH}"
fi

if command -v cargo &>/dev/null; then
    echo "Removing cargo install metadata..."
    cargo uninstall remounter 2>/dev/null || true
fi

if [[ -e "${INSTALL_BIN}" ]]; then
    echo "Removing installed binary..."
    rm -f "${INSTALL_BIN}"
fi

for log_file in "${LOG_OUT}" "${LOG_ERR}"; do
    if [[ -f "${log_file}" ]]; then
        echo "Removing log file ${log_file}..."
        rm -f "${log_file}"
    fi
done

echo "Uninstallation complete."
