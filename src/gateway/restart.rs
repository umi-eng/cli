use crate::write_with_header;
use colored::Colorize;
use std::{net::IpAddr, time::Duration};
use tokio_modbus::client::Writer;

use super::connect_modbus;

#[allow(unused_variables, unused_mut)]
pub async fn command(
    mut output: impl std::io::Write,
    ip: IpAddr,
) -> anyhow::Result<()> {
    let mut modbus = connect_modbus(ip).await?;

    write_with_header(&mut output, "Restarting".green(), " ");

    // write reset coil
    let _ = tokio::time::timeout(
        Duration::from_secs(1),
        modbus.write_single_coil(1, true),
    )
    .await;

    write_with_header(&mut output, "Done".green(), " ");

    Ok(())
}
