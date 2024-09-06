mod config;
mod manifest;
mod reset;
mod restart;
mod status;
mod update;

use clap::{Parser, Subcommand};
use std::{net::IpAddr, path::PathBuf};

#[derive(Subcommand)]
pub enum Commands {
    /// Show status
    Status,
    /// Update firmware
    Update(UpdateOptions),
    /// Reset all configuration
    Reset,
    /// Restart
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
            Commands::Reset => reset::command(output, self.ip).await,
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
