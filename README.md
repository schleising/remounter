# SMB Share Remounter

A simple macOS application to automatically remount SMB shares when connections are lost.

## Features
- Automatically detects and remounts lost SMB shares
- Configurable mount points and post-mount scripts

## Installation
1. Install Rust from [rustup.rs](https://rustup.rs/)
2. Clone this repository:
   ```bash
   git clone https://github.com/schleising/remounter.git
   cd remounter
   ```
3. Build and install the executable:
   ```bash
   cargo install --path .
   ```
4. Use Automator to create a new application that runs the `remounter` command.
  - Open Automator and create a new "Application".
  - Add a "Run Shell Script" action.
  - Set the shell to `/bin/zsh`.
  - Enter the command and arguments to run `remounter`, for example:
    ```bash
    /Users/yourusername/.cargo/bin/remounter <HOST> <SHARES> --post-mount-script /path/to/your/script.sh
    ```
  - Save the application to your desired location.
5. In System Settings > General > Login Items & Extensions, add the application to start it at login.