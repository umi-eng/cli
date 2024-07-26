use std::net::{IpAddr, SocketAddr};
use tokio_modbus::{
    client::{tcp::connect, Reader, Writer},
    slave::SlaveContext,
    Result, Slave,
};

#[derive(Debug)]
pub struct Client {
    modbus: tokio_modbus::client::Context,
}

impl Client {
    pub async fn connect(ip: IpAddr) -> std::io::Result<Self> {
        let mut modbus = connect(SocketAddr::new(ip, 502)).await?;
        modbus.set_slave(Slave(1));

        Ok(Self { modbus })
    }

    /// Get hardware version.
    pub async fn hardware_version(&mut self) -> Result<Version> {
        let version = self.modbus.read_holding_registers(1, 3).await?.unwrap();
        Ok(Ok(Version {
            major: version[0],
            minor: version[1],
            patch: version[2],
        }))
    }

    /// Get firmware version.
    pub async fn firmware_version(&mut self) -> Result<Version> {
        let version = self.modbus.read_holding_registers(4, 3).await?.unwrap();
        Ok(Ok(Version {
            major: version[0],
            minor: version[1],
            patch: version[2],
        }))
    }

    /// Restart the gateway gracefully
    pub async fn restart(&mut self) -> Result<()> {
        self.modbus.write_single_coil(1, true).await
    }
}

#[derive(Debug)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}.{}.{}", self.major, self.minor, self.patch)
    }
}
