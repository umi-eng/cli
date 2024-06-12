use std::time::Instant;

use tokio_modbus::{
    client::{Context as ModbusContext, Reader},
    slave::SlaveContext,
    Slave,
};

#[derive(Debug)]
#[allow(unused)]
enum ModbusSlave {
    System = 1,
    Network = 2,
    Canbus = 3,
}

impl From<ModbusSlave> for Slave {
    fn from(value: ModbusSlave) -> Self {
        Slave(value as u8)
    }
}

pub async fn command(
    mut output: impl std::io::Write,
    mut ctx: ModbusContext,
) -> anyhow::Result<()> {
    let start = Instant::now();

    ctx.set_slave(ModbusSlave::System.into());

    let hardware_version = ctx.read_holding_registers(1, 3).await?;
    writeln!(
        output,
        "Hardware version: {}.{}.{}",
        hardware_version[0], hardware_version[1], hardware_version[2]
    )?;

    let firmware_version = ctx.read_holding_registers(4, 3).await?;
    writeln!(
        output,
        "Firmware version: {}.{}.{}",
        firmware_version[0], firmware_version[1], firmware_version[2]
    )?;

    writeln!(
        output,
        "Got status in {} seconds.",
        start.elapsed().as_secs_f64()
    )?;

    Ok(())
}

fn u16_words_to_u32(words: &[u16]) -> anyhow::Result<u32> {
    // construct a u32 from u16's observing network byte order.
    let uptime: Vec<u8> = words
        .iter()
        .flat_map(|&word| vec![word as u8, (word >> 8) as u8])
        .collect();

    Ok(u32::from_be_bytes([
        uptime[0], uptime[1], uptime[2], uptime[3],
    ]))
}
