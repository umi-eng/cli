use anyhow::Error;
use serde::Deserialize;
use std::io::Cursor;
use std::net::SocketAddr;
use std::{collections::HashMap, net::IpAddr};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, ErrorKind};
use tokio::net::TcpStream;
use uftwo::Block;

use super::UpdateOptions;

/// Manifest file format.
///
/// # Example
///
/// ```json
/// {
///     "schema": "0.1.0",
///     "latest": "v0.3.0",
///     "stable": "v0.2.0",
///     "binaries": {
///         "0.2.0": { ... },
///         "0.3.0": { ... },
///     }
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct Manifest {
    schema: String,
    latest: String,
    stable: String,
    binaries: HashMap<String, FirmwareBinary>,
}

impl Manifest {
    /// Return the metadata for the latest firmware binary.
    ///
    /// Note this will return `None` if the version specified as latest is not
    /// in the map of binaries.
    pub fn latest(&self) -> Option<&FirmwareBinary> {
        self.binaries.get(&self.latest)
    }

    /// Return the metadata for the stable firmware binary.
    ///
    /// Note this will return `None` if the version specified as stable is not
    /// in the map of binaries.
    pub fn stable(&self) -> Option<&FirmwareBinary> {
        self.binaries.get(&self.stable)
    }

    /// The metadata for a specific firmware binary version.
    ///
    /// Note this will return `None` if the version specified is not in the map
    /// of binaries.
    pub fn version(&self, ver: &str) -> Option<&FirmwareBinary> {
        self.binaries.get(ver)
    }

    /// Returns the map of firmware binaries.
    ///
    /// Key: version identifier.
    /// Value: firmware binary metadata.
    pub fn binaries(&self) -> &HashMap<String, FirmwareBinary> {
        &self.binaries
    }
}

/// Metadata for a firmware release binary.
///
/// Each firmware binary _must_ specify the minimum required firmware version
/// so devices can step-up to the latest firmware without issues due to
/// breaking changes.
#[derive(Debug, Deserialize)]
pub struct FirmwareBinary {
    file: String,
    min: String,
    sha256: Option<String>,
}

impl FirmwareBinary {
    /// Firmware binary file URL.
    pub fn file(&self) -> &str {
        &self.file
    }

    /// Minimum version required to upgrade to this binary.
    pub fn minimum_supported_version(&self) -> &str {
        &self.min
    }

    /// SHA256 checksum (if included).
    pub fn sha256(&self) -> Option<&str> {
        self.sha256.as_deref()
    }
}

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
    const PORT: u16 = 0x5546; // firmware update port ("UF" in ASCII)
    let mut stream = TcpStream::connect(SocketAddr::new(ip, PORT)).await?;

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

        let firmware_binary = if let Some(version) = options.version {
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

        let binary = reqwest::get(&firmware_binary.file).await?.bytes().await?;

        upgrade_firmware(output, ip, &binary).await?;
    }

    Ok(())
}
