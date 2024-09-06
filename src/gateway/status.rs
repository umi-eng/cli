use crate::write_with_header;
use colored::Colorize;
use std::{net::IpAddr, time::Instant};

pub async fn command(
    mut output: impl std::io::Write,
    ip: IpAddr,
) -> anyhow::Result<()> {
    let mut client = gateway_client::Client::connect(ip).await?;

    let start = Instant::now();

    write_with_header(
        &mut output,
        "Serial".green(),
        &format!("{}", client.serial().await??),
    );

    write_with_header(
        &mut output,
        "Hardware Version".green(),
        &format!("{}", client.hardware_version().await??),
    );

    write_with_header(
        &mut output,
        "Firmware Version".green(),
        &format!("{}", client.firmware_version().await??,),
    );

    writeln!(output, "Got status in {:?}", start.elapsed())?;

    Ok(())
}
