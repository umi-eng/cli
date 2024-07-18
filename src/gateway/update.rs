use super::UpdateOptions;
use crate::manifest::Manifest;
use anyhow::Error;
use std::io::Cursor;
use std::net::IpAddr;
use std::net::SocketAddr;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, ErrorKind};
use tokio::net::TcpStream;
use uftwo::Block;

async fn upgrade_firmware(
    mut output: impl std::io::Write,
    ip: IpAddr,
    binary: &[u8],
) -> anyhow::Result<()> {
    writeln!(output, "Starting firmware update.")?;

    if binary.len() % 512 != 0 {
        return Err(Error::msg(
            "Failed to read firmware file: firmware file did not align to 512 byte block.",
        ));
    }

    // setup reader
    let cursor = Cursor::new(binary);
    let mut uf2 = BufReader::new(cursor);

    // open TCP connection to UF2 endpoint
    writeln!(output, "Connecting to gateway.")?;
    let mut stream = TcpStream::connect(SocketAddr::new(ip, 21830)).await?;

    writeln!(output, "Starting firmware upgrade.")?;

    let mut block_buf = [0; 512];

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
                    Err(_err) => {
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
            Err(_err) => {
                return Err(Error::msg("Failed to read firmware file block."))
            }
        }
    }

    writeln!(output, "Finished.")?;

    Ok(())
}

pub async fn command(
    mut output: impl std::io::Write,
    options: UpdateOptions,
    ip: IpAddr,
) -> anyhow::Result<()> {
    if let Some(file_path) = options.file {
        writeln!(output, "Reading firmware file.")?;
        let mut file = File::open(file_path).await?;
        let meta = file.metadata().await?;

        let mut contents = vec![0; meta.len() as usize];
        file.read(&mut contents).await?;

        upgrade_firmware(output, ip, &contents).await?;
    } else {
        writeln!(output, "Getting firmware.")?;

        let manifest = reqwest::get(
            "https://cdn.umi.engineering/firmware/gateway/manifest.json",
        )
        .await?
        .json::<Manifest>()
        .await?;

        let firmware = if let Some(version) = options.version {
            match manifest.version(&version) {
                Some(fw) => fw,
                None => {
                    return Err(Error::msg(format!(
                        "Firmware version {} was not found.",
                        version
                    )))
                }
            }
        } else {
            match manifest.stable() {
                Some(fw) => fw,
                None => {
                    return Err(Error::msg(
                        "Stable firmware version was not found.",
                    ))
                }
            }
        };

        writeln!(output, "Version: {}.", firmware.0)?;

        let binary = reqwest::get(firmware.1.file()).await?.bytes().await?;

        upgrade_firmware(output, ip, &binary).await?;
    }

    Ok(())
}
