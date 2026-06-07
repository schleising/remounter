# SMB Share Remounter

A simple macOS application to automatically remount SMB shares when connections are lost.

## Features

- Automatically detects and remounts lost SMB shares
- Configurable mount points and post-mount scripts
- Runs as a user launchd agent in the background

## Prerequisites

1. Install Rust from [rustup.rs](https://rustup.rs/)
2. Clone this repository:
   ```bash
   git clone https://github.com/schleising/remounter.git
   cd remounter
   ```

## Installation

Run the install script with your SMB host and comma-separated share names:

```bash
./install.sh --host nas.local --shares Media,home
```

Optional post-mount script:

```bash
./install.sh --host nas.local --shares Media,home --post-mount-script /path/to/script.sh
```

The same values can be supplied via environment variables instead of flags:

```bash
REMOUNTER_HOST=nas.local REMOUNTER_SHARES=Media,home ./install.sh
```

The installer will:

1. Run `cargo update`
2. Build and install the `remounter` binary to `~/.cargo/bin`
3. Install a launchd agent plist to `~/Library/LaunchAgents`
4. Start the agent immediately

Logs are written to:

- `~/Library/Logs/remounter.log`
- `~/Library/Logs/remounter.err.log`

## Uninstallation

```bash
./uninstall.sh
```

This stops the launchd agent, removes the plist, uninstalls the binary, and deletes the log files.

## Health check

Place a file named `.smb_remounter` in the root of each SMB share. When remounting, shares that are already mounted and contain this marker are treated as healthy and skipped.

## First run

On the first reboot macOS may prompt for your password to allow mounting the shares. Enter it and check "Remember this password in my keychain" to avoid being prompted again.

On first run macOS may ask if the application can access certain folders; grant access as needed.

## Manual usage

You can also run the binary directly without launchd:

```bash
remounter nas.local Media,home --post-mount-script /path/to/script.sh
```
