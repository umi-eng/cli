use clap::{Parser, Subcommand};
use std::net::{IpAddr, SocketAddr};
use tokio::time::Instant;
use tokio_modbus::{
    client::{tcp::connect, Context, Reader},
    slave::SlaveContext,
    Slave,
};

#[derive(Subcommand)]
pub enum Commands {
    /// Show important status information
    Status,
    /// Open web dashboard in default browser
    Dashboard,
    /// Perform a firmware update
    Update,
    /// Restart a gateway
    Restart,
    /// Reset a gateway to default configuration
    Reset,
}

#[derive(Parser)]
pub struct Cmd {
    #[clap(subcommand)]
    subcommand: Commands,
    #[arg(value_name = "IP")]
    ip: IpAddr,
}

impl Cmd {
    pub async fn run(self) -> anyhow::Result<()> {
        // so we can log to files later.
        let output = std::io::stdout().lock();

        let socket_addr = SocketAddr::new(self.ip, 502);
        let ctx = connect(socket_addr).await?;

        match self.subcommand {
            Commands::Status => status(output, ctx).await,
            Commands::Dashboard => dashboard(output, self.ip).await,
            Commands::Update => update(output).await,
            Commands::Restart => restart(output).await,
            Commands::Reset => reset(output).await,
        }
    }
}

#[derive(Debug)]
enum ModbusSlave {
    System = 1,
    Network = 2,
    Canbus = 3,
}

impl Into<Slave> for ModbusSlave {
    fn into(self) -> Slave {
        Slave(self as u8)
    }
}

async fn status(mut output: impl std::io::Write, mut ctx: Context) -> anyhow::Result<()> {
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

    let uptime = u16_words_to_u32(&ctx.read_holding_registers(7, 2).await?)?;
    writeln!(output, "Uptime: {} seconds", uptime)?;

    writeln!(
        output,
        "Got status in {} seconds.",
        start.elapsed().as_secs_f64()
    )?;

    Ok(())
}

async fn dashboard(mut output: impl std::io::Write, ip: IpAddr) -> anyhow::Result<()> {
    let url = format!("http://{}:80", ip);
    writeln!(output, "Opening dashboard: {}", url)?;
    open::that(url)?;

    Ok(())
}

async fn update(mut output: impl std::io::Write) -> anyhow::Result<()> {
    todo!();
}

async fn restart(mut output: impl std::io::Write) -> anyhow::Result<()> {
    todo!();
}

async fn reset(mut output: impl std::io::Write) -> anyhow::Result<()> {
    todo!();
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
