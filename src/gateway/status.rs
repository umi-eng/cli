use super::connect_modbus;
use crate::write_with_header;
use colored::Colorize;
use std::{net::IpAddr, time::Instant};
use tokio_modbus::client::Reader;

pub async fn command(
    mut output: impl std::io::Write,
    ip: IpAddr,
) -> anyhow::Result<()> {
    let mut modbus = connect_modbus(ip).await?;

    let start = Instant::now();

    let hardware_version = modbus.read_holding_registers(1, 3).await?;
    write_with_header(
        &mut output,
        "Hardware Version".green(),
        &format!(
            "v{}.{}.{}",
            hardware_version[0], hardware_version[1], hardware_version[2]
        ),
    );

    let firmware_version = modbus.read_holding_registers(4, 3).await?;
    write_with_header(
        &mut output,
        "Firmware Version".green(),
        &format!(
            "v{}.{}.{}",
            firmware_version[0], firmware_version[1], firmware_version[2],
        ),
    );

    writeln!(output, "Got status in {:?}", start.elapsed())?;

    Ok(())
}
