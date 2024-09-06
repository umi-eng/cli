use crate::write_with_header;
use colored::Colorize;
use std::{net::IpAddr, time::Duration};
use tokio::time::timeout;

#[allow(unused_variables, unused_mut)]
pub async fn command(
    mut output: impl std::io::Write,
    ip: IpAddr,
) -> anyhow::Result<()> {
    let mut client = gateway_client::Client::connect(ip).await?;

    write_with_header(&mut output, "Resetting".green(), " ");
    let _ = timeout(Duration::from_secs(1), client.reset()).await;
    write_with_header(&mut output, "Done".green(), " ");

    Ok(())
}