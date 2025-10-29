use std::{
    fmt::Debug,
    net::{SocketAddr, TcpStream, ToSocketAddrs},
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::sleep,
    time::Duration,
};

use anyhow::Result;

use signal_hook::flag::register;
use tracing::{debug, error, info, instrument};

/// Struct representing the Remounter
pub struct Remounter {
    server: String,
    socket_address: SocketAddr,
    smb_shares: Vec<PathBuf>,
    post_mount_script: Option<String>,
}

/// Create a new Remounter instance
#[instrument]
pub fn new_remounter<S, I, P>(
    server: S,
    smb_shares: I,
    post_mount_script: Option<String>,
) -> Result<Remounter>
where
    S: Into<String> + Debug,
    I: IntoIterator<Item = P> + Debug,
    P: Into<PathBuf>,
{
    // Resolve the server address to a SocketAddr
    let server = server.into();
    let socket_address = format!("{}:445", server);
    let socket_address = socket_address
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| anyhow::anyhow!("Could not resolve to any addresses"))?;

    // Create the Remounter instance
    let remounter = Remounter {
        server,
        socket_address,
        smb_shares: smb_shares.into_iter().map(Into::into).collect(),
        post_mount_script,
    };

    // Return the Remounter instance
    Ok(remounter)
}

impl Remounter {
    /// Run the remounter
    #[instrument(skip(self))]
    pub fn run(&self) -> Result<()> {
        // Run the connection check loop
        self.check_connection()?;

        // If we exit the loop due to a termination signal, return Ok(())
        Ok(())
    }

    /// Check if the server is reachable
    #[instrument(skip(self))]
    fn is_up(&self, address: &SocketAddr) -> bool {
        // Attempt to connect to the address with a timeout of 5 seconds
        TcpStream::connect_timeout(address, Duration::from_secs(5)).is_ok()
    }

    /// Check the connection status and trigger remounting when the connection is restored
    #[instrument(skip(self))]
    fn check_connection(&self) -> Result<()> {
        // Register signal handlers for SIGTERM and SIGINT
        let term = Arc::new(AtomicBool::new(false));
        register(signal_hook::consts::SIGTERM, Arc::clone(&term))?;
        register(signal_hook::consts::SIGINT, Arc::clone(&term))?;

        // Set initial state of was_up to false
        let mut was_up = false;

        // Main loop to check connection status
        while !term.load(Ordering::Relaxed) {
            // Check if the socket is up or down and handle state changes
            if self.is_up(&self.socket_address) {
                if !was_up {
                    // Update state to indicate the connection is now up
                    was_up = true;

                    // Log that the connection is back up
                    info!(
                        "{}:{} is up, attempting to remount...",
                        self.server,
                        self.socket_address.port()
                    );

                    // Attempt to remount drives when the connection is back up
                    match self.remount_shares() {
                        Ok(_) => info!("Remount successful"),
                        Err(e) => {
                            // Log the remount failure
                            error!("Remount failed: {}", e);

                            // Continue to the next iteration of the loop without executing the post-mount script
                            continue;
                        }
                    }

                    // If a post-mount script is provided, execute it
                    if let Some(script) = &self.post_mount_script {
                        info!("Executing post-mount script: {}", script);
                        let status = Command::new("sh").arg("-c").arg(script).status()?;
                        if !status.success() {
                            error!("Post-mount script failed with status: {}", status);
                        }
                    }
                }
            } else if was_up {
                // Log that the connection is down
                info!(
                    "{}:{} is down, will attempt to remount when it is back up",
                    self.server,
                    self.socket_address.port()
                );

                // Update state to indicate the connection is now down
                was_up = false;
            }

            // Sleep for 1 second before the next check
            sleep(Duration::from_secs(1));
        }

        info!("Termination signal received, exiting...");
        Ok(())
    }

    /// Function to handle remounting a single share
    #[instrument(skip(self))]
    fn remount(&self, smb_share: &Path) -> Result<()> {
        // If the share path exists, skip remounting
        if smb_share.exists() {
            info!("Share {} is already mounted, skipping remount", smb_share.display());
            return Err(anyhow::anyhow!("Share {} exists, not mounting", smb_share.display()));
        }

        // Convert the share path to a string
        let share_path = smb_share
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid share path"))?;

        // Construct the mount command
        let mount_command = format!(
            "osascript -e 'mount volume \"smb://{}{}\"'",
            self.server, share_path,
        );

        // Log the mount command for info
        debug!("Executing mount command: {}", mount_command);

        // Execute the mount command using AppleScript
        let status = Command::new("sh").arg("-c").arg(mount_command).status()?;

        // Check if the command was successful, return an error if not
        if !status.success() {
            return Err(anyhow::anyhow!("Failed to execute mount command"));
        }

        Ok(())
    }

    /// Remount all shares
    #[instrument(skip(self))]
    fn remount_shares(&self) -> Result<()> {
        // Collect errors from remount attempts
        let errors = self
            .smb_shares
            .iter()
            .filter_map(|share| self.remount(share).err())
            .collect::<Vec<_>>();

        // If there were any errors, log them and return an error
        if !errors.is_empty() {
            for error in errors {
                error!("Error remounting share: {}", error);
            }
            return Err(anyhow::anyhow!("One or more shares failed to remount"));
        }

        // If all shares were remounted successfully, return Ok(())
        Ok(())
    }
}
