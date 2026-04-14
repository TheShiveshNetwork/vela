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
    },
    /// Unmount a device
    Unmount {
        mountpoint: String,
    },
    /// Start the HTTP server for a specific mount name in /mnt/
    Serve {
        /// The name of the mount in /mnt/ (e.g., 'temp' for /mnt/temp)
        mount_name: String,
    },
}

impl Cli {
    pub async fn run() -> anyhow::Result<()> {
        match Cli::parse().command {
            Commands::Scan => {
                device::scan()?;
            }

            Commands::Mount { device, mount_name } => {
                device::mount_device(&device, &mount_name)?;
            }

            Commands::Unmount { mountpoint } => {
                device::unmount_device(&mountpoint)?;
            }

            Commands::Serve { mount_name } => {
                let mount_path = format!("/mnt/{}", mount_name);
                server::start(mount_path).await?;
            }
        }
        Ok(())
    }
}
