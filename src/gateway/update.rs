use super::manifest::Manifest;
use super::UpdateOptions;
use crate::write_with_header;
use anyhow::Error;
use colored::Colorize;
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
    if binary.len() % 512 != 0 {
        return Err(Error::msg(
            "Failed to read firmware file: firmware file did not align to 512 byte block.",
        ));
    }

    // setup reader
    let cursor = Cursor::new(binary);
    let mut uf2 = BufReader::new(cursor);

    // open TCP connection to UF2 endpoint
    let mut stream = TcpStream::connect(SocketAddr::new(ip, 21830)).await?;

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

                let block = match Block::from_bytes(&block_buf) {
                    Ok(b) => b,
                    Err(_err) => {
                        return Err(Error::msg(
                            "Failed to read firmware file block.",
                        ))
                    }
                };

                // send block to gateway
                let _ = stream.write(&block_buf).await?;

                if block.block == 0 {
                    write_with_header(&mut output, "Erasing".green(), "...");
                }

                if block.block == 1 {
                    write_with_header(&mut output, "Loading".green(), "...");
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

    write_with_header(&mut output, "Done".green(), " ");

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
        let _ = file.read(&mut contents).await?;

        upgrade_firmware(output, ip, &contents).await?;
    } else {
        write_with_header(&mut output, "Downloading".green(), " ");

        let manifest = reqwest::get(
            "https://cdn.umi.engineering/firmware/gateway/manifest.json",
        )
        .await?
        .json::<Manifest>()
        .await?;

        let firmware = if let Some(version) = options.version {
            match manifest.binaries.get_key_value(&version) {
                Some(fw) => fw,
                None => {
                    return Err(Error::msg(format!(
                        "Firmware version {} was not found.",
                        version
                    )))
                }
            }
        } else {
            match manifest.binaries.get_key_value(&manifest.stable) {
                Some(fw) => fw,
                None => {
                    return Err(Error::msg(format!(
                        "Stable firmware version {} was not found.",
                        manifest.stable,
                    )))
                }
            }
        };

        write_with_header(&mut output, "Version".green(), firmware.0);

        let binary = reqwest::get(&firmware.1.file).await?.bytes().await?;

        upgrade_firmware(output, ip, &binary).await?;
    }

    Ok(())
}
