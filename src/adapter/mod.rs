mod list;
mod update;

use clap::{Parser, Subcommand};
use std::{net::IpAddr, path::PathBuf};

#[derive(Subcommand)]
pub enum Commands {
    /// List devices.
    List,
    /// Update firmware
    Update(UpdateOptions),
}

#[derive(Parser)]
pub struct Cmd {
    #[clap(subcommand)]
    subcommand: Commands,
}

impl Cmd {
    pub async fn run(self) -> anyhow::Result<()> {
        // so we can log to files later.
        let output = std::io::stdout();

        match self.subcommand {
            Commands::List => list::command(output).await,
            Commands::Update(options) => update::command(output, options).await,
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
