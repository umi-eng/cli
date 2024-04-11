use anyhow::Error;
use clap::{Parser, Subcommand};
use std::{
    io::{ErrorKind, SeekFrom},
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    time::Instant,
};
use tokio_modbus::{
    client::{tcp::connect, Context as ModbusContext, Reader},
    slave::SlaveContext,
    Slave,
};
use uftwo::Block;

#[derive(Subcommand)]
pub enum Commands {
    // Show important status information
    // Status,
    // Open web dashboard in default browser
    // Dashboard,
    /// Perform a firmware update
    Update(UpdateOptions),
    // Restart a gateway
    // Restart,
    // Reset a gateway to default configuration
    // Reset,
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
        // let ctx = connect(socket_addr).await?;

        match self.subcommand {
            // Commands::Status => status(output, ctx).await,
            // Commands::Dashboard => dashboard(output, self.ip).await,
            Commands::Update(options) => update(output, options, self.ip).await,
            // Commands::Restart => restart(output).await,
            // Commands::Reset => reset(output).await,
        }
    }
}

#[derive(Debug)]
#[allow(unused)]
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

async fn status(
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

    let uptime = u16_words_to_u32(&ctx.read_holding_registers(7, 2).await?)?;
    writeln!(output, "Uptime: {} seconds", uptime)?;

    writeln!(
        output,
        "Got status in {} seconds.",
        start.elapsed().as_secs_f64()
    )?;

    Ok(())
}

async fn dashboard(
    mut output: impl std::io::Write,
    ip: IpAddr,
) -> anyhow::Result<()> {
    let url = format!("http://{}:80", ip);
    writeln!(output, "Opening dashboard: {}", url)?;
    open::that(url)?;

    Ok(())
}

#[derive(Parser)]
pub struct UpdateOptions {
    /// Firmware file.
    #[clap(short, long)]
    file: PathBuf,
}

#[allow(unused_variables, unused_mut)]
async fn update(
    mut output: impl std::io::Write,
    options: UpdateOptions,
    ip: IpAddr,
) -> anyhow::Result<()> {
    // open the firmware file and get metadata
    writeln!(output, "Reading firmware file.")?;
    let mut file = File::open(options.file).await?;
    let meta = file.metadata().await?;

    if meta.len() % 512 != 0 {
        return Err(Error::msg(
            "Failed to read firmware file: firmware file did not align to 512 byte block.",
        ));
    }

    // read first block
    // later we'll use this to read extension tags for important metadata
    let mut block_buf = [0; 512];
    file.read(&mut block_buf).await?;
    let first_block = match Block::from_bytes_ref(&block_buf) {
        Ok(b) => b,
        Err(err) => {
            return Err(Error::msg("Failed to read firmware file block."))
        }
    };
    // return seek position to start
    file.seek(SeekFrom::Start(0)).await?;

    let mut block_buf = [0; 512];
    let mut uf2 = BufReader::new(file);

    // open TCP connection to UF2 endpoint
    writeln!(output, "Connecting to gateway.")?;
    const PORT: u16 = 0x5546; // firmware update port ("UF" in ASCII)
    let mut stream = TcpStream::connect(SocketAddr::new(ip, PORT)).await?;

    writeln!(output, "Starting firmware upgrade.")?;

    loop {
        match uf2.read(&mut block_buf).await {
            Ok(0) => break,
            Ok(n) => {
                if n != 512 {
                    return Err(Error::msg(
                        "Failed to read firmware file: firmware file did not align to 512 byte block.",
                    ));
                }

                let block = match Block::from_bytes_ref(&block_buf) {
                    Ok(b) => b,
                    Err(err) => {
                        return Err(Error::msg(
                            "Failed to read firmware file block.",
                        ))
                    }
                };

                // send block to gateway
                stream.write(&block_buf).await?;

                if block.block_number == 0 {
                    writeln!(output, "Erasing.")?;
                }

                let mut response = [0; 3];
                stream.read(&mut response).await?;

                if &response == b"ok\0" {
                    continue;
                } else {
                    return Err(Error::msg("An error occurred. Please reset the device and try again."));
                }
            }
            Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => {
                return Err(Error::msg("Failed to read firmware file block."))
            }
        }
    }

    writeln!(output, "Finished.")?;

    Ok(())
}

#[allow(unused_variables, unused_mut)]
async fn restart(mut output: impl std::io::Write) -> anyhow::Result<()> {
    todo!();
}

#[allow(unused_variables, unused_mut)]
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
