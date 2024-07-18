mod gateway;
mod manifest;

use clap::{Parser, Subcommand};
use colored::ColoredString;

#[derive(Subcommand)]
pub enum Commands {
    /// Commands for managing Gateway devices
    Gateway(gateway::Cmd),
}

#[derive(Parser)]
#[command(about, version)]
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

fn write_with_header(
    mut output: impl std::io::Write,
    header: ColoredString,
    msg: &str,
) {
    let _ = write!(output, "{: >1$} ", header, 16);

    let mut lines = msg.lines();

    if let Some(first_line) = lines.next() {
        let _ = writeln!(output, "{first_line}");
    }

    for line in lines {
        let _ = writeln!(output, "            {line}");
    }
}
