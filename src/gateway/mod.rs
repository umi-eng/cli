mod status;
mod update;

use clap::{Parser, Subcommand};
use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};
use tokio_modbus::client::tcp::connect;

#[derive(Subcommand)]
pub enum Commands {
    /// Show important status information
    Status,
    /// Perform a firmware update
    Update(UpdateOptions),
    /// Restart a gateway
    Restart,
    /// Reset a gateway to default configuration
    Reset,
}

#[derive(Parser)]
pub struct Cmd {
    #[clap(subcommand)]
    subcommand: Commands,
    #[arg(value_name = "IP")]
    ip: IpAddr,
}

impl Cmd {
    pub async fn run(self) -> anyhow::Result<()> {
        // so we can log to files later.
        let output = std::io::stdout().lock();

        let socket_addr = SocketAddr::new(self.ip, 502);
        let ctx = connect(socket_addr).await?;

        match self.subcommand {
            Commands::Status => status::command(output, ctx).await,
            Commands::Update(options) => {
                update::command(output, options, self.ip).await
            }
            Commands::Restart => restart(output).await,
            Commands::Reset => reset(output).await,
        }
    }
}

#[derive(Parser)]
pub struct UpdateOptions {
    /// Update using firmware file.
    #[clap(long)]
    file: Option<PathBuf>,
    /// Update to a specific version.
    #[clap(long)]
    version: Option<String>,
}

#[allow(unused_variables, unused_mut)]
async fn restart(mut output: impl std::io::Write) -> anyhow::Result<()> {
    todo!();
}

#[allow(unused_variables, unused_mut)]
async fn reset(mut output: impl std::io::Write) -> anyhow::Result<()> {
    todo!();
}
