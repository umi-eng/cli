mod config;
mod manifest;
mod restart;
mod status;
mod update;

use clap::{Parser, Subcommand};
use std::{net::IpAddr, path::PathBuf};

/// Magic numbers used to identify Gateway types.
/// Must be u16 to fit in one Modbus register.
#[derive(Debug, Clone, Copy)]
enum MagicNumId {
    /// "FD" like CAN FD
    CanFd = 0x4644,
    /// "RS" like RS-232/RS-485
    Serial = 0x5253,
}

impl TryFrom<u16> for MagicNumId {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            x if x == MagicNumId::CanFd as u16 => Ok(MagicNumId::CanFd),
            x if x == MagicNumId::Serial as u16 => Ok(MagicNumId::Serial),
            _ => Err(()),
        }
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Show status
    Status,
    /// Update firmware
    Update(UpdateOptions),
    /// Command the device to restart
    Restart,
    /// Read and write configuration
    Config(config::Cmd),
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
        let output = std::io::stdout();

        match self.subcommand {
            Commands::Status => status::command(output, self.ip).await,
            Commands::Update(options) => {
                update::command(output, options, self.ip).await
            }
            Commands::Restart => restart::command(output, self.ip).await,
            Commands::Config(command) => command.run(output, self.ip).await,
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
