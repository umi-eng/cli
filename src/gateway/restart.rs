use std::time::Duration;
use tokio_modbus::client::{Context as ModbusContext, Writer};

#[allow(unused_variables, unused_mut)]
pub async fn command(
    mut output: impl std::io::Write,
    mut ctx: ModbusContext,
) -> anyhow::Result<()> {
    writeln!(output, "Resetting gateway...")?;

    // write reset coil
    let _ = tokio::time::timeout(
        Duration::from_secs(1),
        ctx.write_single_coil(1, true),
    )
    .await;

    writeln!(output, "Done")?;

    Ok(())
}
