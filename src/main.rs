mod gateway;

use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum Commands {
    /// Commands for managing Gateway devices
    Gateway(gateway::Cmd),
}

#[derive(Parser)]
#[clap(name = "UMI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Gateway(command) => command.run(),
    }
    .await
}
