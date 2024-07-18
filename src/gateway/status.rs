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

    writeln!(output, "Got status in {:?}", start.elapsed())?;

    Ok(())
}
