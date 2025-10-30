mod remounter;

use std::path::Path;

use clap::Parser;

use tracing::{error, info, instrument};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::remounter::new_remounter;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The hostname to monitor (e.g., example.com)
    host: String,

    /// The SMB shares to remount (comma-separated paths)
    smb_shares: String,

    /// A script to run after remounting
    #[arg(short, long)]
    post_mount_script: Option<String>,
}

#[instrument]
fn main() {
    // Create a human-readable time formatter
    let custom_format = time::format_description::well_known::Rfc3339;

    // Human-readable console logs (with colours)
    let console_layer = fmt::layer()
        .with_timer(fmt::time::UtcTime::new(custom_format))
        .with_target(true);

    // Initialize the tracing subscriber with both layers
    tracing_subscriber::registry().with(console_layer).init();

    // Parse command-line arguments
    let args = Args::parse();
    let smb_shares: Vec<&Path> = args
        .smb_shares
        .split(',')
        .map(|share| Path::new(share.trim()))
        .collect();

    // Combine the startup message into a single multiline log entry
    let mut startup_message = format!("Starting remounter version {}\n", env!("CARGO_PKG_VERSION"));
    startup_message.push_str(&format!("Monitoring SMB shares on {}:\n", args.host));
    for share in &smb_shares {
        startup_message.push_str(&format!(" - {}\n", share.display()));
    }
    if let Some(script) = &args.post_mount_script {
        startup_message.push_str(&format!("Post-mount script: {}\n", script));
    }
    info!("{}", startup_message.trim_end());

    // Create the remounter
    let remounter = new_remounter(args.host, smb_shares, args.post_mount_script);

    // Handle any errors that occur during remounter creation or execution
    let remounter = match remounter {
        Ok(r) => r,
        Err(e) => {
            error!("Error creating remounter: {}", e);
            std::process::exit(1);
        }
    };

    // Run the remounter
    if let Err(e) = remounter.run() {
        error!("Error running remounter: {}", e);
        std::process::exit(1);
    }

    info!("Remounter exited normally");
}
