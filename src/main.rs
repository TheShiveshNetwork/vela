mod cli;
mod device;
mod config;
mod server;

use cli::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Cli::run().await
}
