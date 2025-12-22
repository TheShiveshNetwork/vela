use clap::{Parser, Subcommand};
use crate::device;
use crate::config;
use crate::server;

#[derive(Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Scan,
    Mount {
        device: String,
        mount_name: String,
    },
    Unmount {
        mountpoint: String,
    },
    Select {
        mount_path: String,
    },
    Serve,
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

            Commands::Select { mount_path } => {
                config::save_selection(mount_path)?;
            }

            Commands::Serve => {
                let mount = config::load_selection()?;
                server::start(mount).await?;
            }
        }
        Ok(())
    }
}

