mod remounter;

use std::path::Path;

use crate::remounter::new_remounter;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The hostname to monitor (e.g., example.com)
    host: String,

    /// The SMB shares to monitor
    smb_shares: String,
}

fn main() {
    // Parse command-line arguments
    let args = Args::parse();
    let smb_shares: Vec<&Path> = args.smb_shares.split(',').map(Path::new).collect();

    // Print the parsed arguments and version information from Cargo.toml
    println!("Remounter version {}", env!("CARGO_PKG_VERSION"));
    println!("Monitoring SMB shares on {}:", args.host);
    for share in &smb_shares {
        println!(" - {}", share.display());
    }

    // Create the remounter
    let remounter = new_remounter(args.host, smb_shares);

    // Handle any errors that occur during remounter creation or execution
    let remounter = match remounter {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error creating remounter: {}", e);
            std::process::exit(1);
        }
    };

    // Run the remounter
    if let Err(e) = remounter.run() {
        eprintln!("Error running remounter: {}", e);
        std::process::exit(1);
    }

    println!("Remounter exited normally");
}
