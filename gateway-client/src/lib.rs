use std::net::{IpAddr, Ipv4Addr, SocketAddr};
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

    /// Restart the gateway gracefully
    pub async fn restart(&mut self) -> Result<()> {
        self.modbus.write_single_coil(1, true).await
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

    /// Get DHCP enabled.
    pub async fn dhcp(&mut self) -> Result<bool> {
        let enabled = self.modbus.read_coils(1001, 1).await?.unwrap();
        Ok(Ok(enabled[0]))
    }

    /// Set DHCP enabled.
    pub async fn set_dhcp(&mut self, enabled: bool) -> Result<()> {
        self.modbus.write_single_coil(1001, enabled).await
    }

    /// Get the configured IPv4 address.
    pub async fn ipv4_address(&mut self) -> Result<Ipv4Addr> {
        let address = self.modbus.read_input_registers(1001, 4).await?.unwrap();
        Ok(Ok(Ipv4Addr::new(
            address[0] as u8,
            address[1] as u8,
            address[2] as u8,
            address[3] as u8,
        )))
    }

    /// Set the IPv4 address.
    pub async fn set_ipv4_address(&mut self, ip: Ipv4Addr) -> Result<()> {
        let words = ip.octets().map(|o| o as u16);
        self.modbus.write_multiple_registers(1001, &words).await
    }

    /// Get CAN bus receive error count.
    pub async fn canbus_receive_error_count(&mut self) -> Result<u16> {
        let count = self.modbus.read_input_registers(2001, 1).await?.unwrap();
        Ok(Ok(count[0]))
    }

    /// Get CAN bus transmit error count.
    pub async fn canbus_transmit_error_count(&mut self) -> Result<u16> {
        let count = self.modbus.read_input_registers(2002, 1).await?.unwrap();
        Ok(Ok(count[0]))
    }

    /// Get the CAN bus nominal rate in bits per second.
    pub async fn canbus_bitrate_nominal(&mut self) -> Result<u32> {
        let rate = self.modbus.read_holding_registers(2001, 1).await?.unwrap();
        Ok(Ok(rate[0] as u32 * 100))
    }

    /// Set the CAN bus nominal rate in bits per second.
    pub async fn set_canbus_bitrate_nominal(
        &mut self,
        rate: u32,
    ) -> Result<()> {
        let rate = (rate / 100) as u16;
        self.modbus.write_single_register(2001, rate).await
    }

    /// Get the CAN bus data rate in bits per second.
    pub async fn canbus_bitrate_data(&mut self) -> Result<u32> {
        let rate = self.modbus.read_holding_registers(2001, 1).await?.unwrap();
        Ok(Ok(rate[0] as u32 * 100))
    }

    /// Set the CAN bus data rate in bits per second.
    pub async fn set_canbus_bitrate_data(&mut self, rate: u32) -> Result<()> {
        let rate = (rate / 100) as u16;
        self.modbus.write_single_register(2001, rate).await
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
