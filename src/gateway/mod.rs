mod status;

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
    /// Show important status information
    Status,
    /// Perform a firmware update
    Update(UpdateOptions),
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
            Commands::Status => status::command(output, ctx).await,
            Commands::Update(options) => update(output, options, self.ip).await,
            Commands::Restart => restart(output).await,
            Commands::Reset => reset(output).await,
        }
    }
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
    let _ = file.read(&mut block_buf).await?;
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
                let _ = stream.write(&block_buf).await?;

                if block.block_number == 0 {
                    writeln!(output, "Erasing.")?;
                }

                let mut response = [0; 3];
                let _ = stream.read(&mut response).await?;

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
