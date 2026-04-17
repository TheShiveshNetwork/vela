use clap::{Parser, Subcommand};
use crate::device;
use crate::server;

#[derive(Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Scan for storage devices
    Scan,
    /// Mount a device to /mnt/<mount_name>
    Mount {
        device: String,
        mount_name: String,
        /// Mode: 'r' for read-only (default), 'w' for read-write
        #[arg(short, long, default_value = "r")]
        mode: String,
    },
    /// Unmount a device
    Unmount {
        mountpoint: String,
    },
    /// Start the HTTP server for a specific mount name in /mnt/
    Serve {
        /// The name of the mount in /mnt/ (e.g., 'temp' for /mnt/temp)
        mount_name: String,
        /// Mode: 'r' for read-only (default), 'w' for read-write
        #[arg(short, long, default_value = "r")]
        mode: String,
        /// Disable authentication
        #[arg(long, default_value_t = false)]
        no_auth: bool,
    },
}

impl Cli {
    pub async fn run() -> anyhow::Result<()> {
        match Cli::parse().command {
            Commands::Scan => {
                device::scan()?;
            }

            Commands::Mount { device, mount_name, mode } => {
                let read_only = mode == "r";
                device::mount_device(&device, &mount_name, read_only)?;
            }

            Commands::Unmount { mountpoint } => {
                device::unmount_device(&mountpoint)?;
            }

            Commands::Serve { mount_name, mode, no_auth } => {
                let mount_path = format!("/mnt/{}", mount_name);
                let read_only = mode == "r";
                server::start(mount_path, read_only, no_auth).await?;
            }
        }
        Ok(())
    }
}
