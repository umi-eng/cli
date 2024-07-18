mod reset;
mod restart;
mod status;
mod update;

use clap::{Parser, Subcommand};
use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};
use tokio_modbus::{client::tcp::connect, slave::SlaveContext, Slave};

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
        let mut output = std::io::stdout();

        let socket_addr = SocketAddr::new(self.ip, 502);
        let mut ctx = connect(socket_addr).await?;
        ctx.set_slave(Slave(1));

        match self.subcommand {
            Commands::Status => status::command(&mut output, ctx).await,
            Commands::Update(options) => {
                update::command(output, options, self.ip).await
            }
            Commands::Restart => restart::command(output, ctx).await,
            Commands::Reset => reset::command(output).await,
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
