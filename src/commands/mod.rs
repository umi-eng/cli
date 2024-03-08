mod gw;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Commands for managing Gateway devices
    Gw {
        #[command(subcommand)]
        command: Option<gw::Commands>,
    },
}
