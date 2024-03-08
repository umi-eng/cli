use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Show important status information
    Status,
    /// Perform a firmware update
    Update,
    /// Restart a gateway
    Restart,
    /// Reset a gateway to default configuration
    Reset,
}
