use clap::{error, Parser, Subcommand};
use colored::Colorize;
use gateway_client::Client;
use std::net::Ipv4Addr;

use crate::write_with_header;

#[derive(Subcommand)]
enum Commands {
    /// DHCPv4 enable/disable.
    Dhcp(Dhcp),
    /// IPv4 address.
    Ipv4(Ipv4),
    /// CAN Bus bitrate.
    CanBitrate(CanBitrate),
}

#[derive(Parser)]
struct Dhcp {
    // Enable or disable DHCP.
    #[arg(value_parser = parse_enable)]
    enable: Option<bool>,
}

#[derive(Parser)]
struct Ipv4 {
    /// Set the static IPv4 address.
    ip: Option<Ipv4Addr>,
}

#[derive(Parser)]
struct CanBitrate {
    /// Set the nominal data rate in bits per second.
    nominal: Option<u32>,
    /// Set the data bitrate in bits per second. (optional)
    data: Option<u32>,
}

#[derive(Parser)]
pub struct Cmd {
    #[clap(subcommand)]
    subcommand: Commands,
}

impl Cmd {
    pub async fn run(
        self,
        mut output: impl std::io::Write,
        ip: std::net::IpAddr,
    ) -> anyhow::Result<()> {
        let mut client = Client::connect(ip).await?;

        match self.subcommand {
            Commands::Dhcp(dhcp) => {
                if let Some(enable) = dhcp.enable {
                    client.set_dhcp(enable).await??;
                    writeln!(output, "Done")?;
                } else {
                    writeln!(output, "{}", client.dhcp().await??)?;
                }
                Ok(())
            }
            Commands::Ipv4(ipv4) => {
                if let Some(ip) = ipv4.ip {
                    client.set_ipv4_address(ip).await??;
                    writeln!(output, "Done")?;
                } else {
                    writeln!(output, "{}", client.ipv4_address().await??)?;
                }

                Ok(())
            }
            Commands::CanBitrate(can_bitrate) => {
                if let Some(nominal) = can_bitrate.nominal {
                    // use same as nominal if not specified
                    let data = can_bitrate.data.unwrap_or(nominal);

                    if nominal < 10_000 {
                        return Err(anyhow::Error::msg(
                            "Nominal bitrate too low.",
                        ));
                    }

                    if nominal > 5_000_000 {
                        return Err(anyhow::Error::msg(
                            "Nominal bitrate too high.",
                        ));
                    }

                    if data < 10_000 {
                        return Err(anyhow::Error::msg(
                            "Data bitrate too low.",
                        ));
                    }

                    if data > 5_000_000 {
                        return Err(anyhow::Error::msg(
                            "Data bitrate too high.",
                        ));
                    }

                    client.set_canbus_bitrate_nominal(data / 100).await??;
                    client.set_canbus_bitrate_data(data / 100).await??;

                    writeln!(output, "Done")?;
                } else {
                    let nominal = client.canbus_bitrate_nominal().await??;
                    let data = client.canbus_bitrate_data().await??;

                    write_with_header(
                        &mut output,
                        "Nominal bitrate".green(),
                        &format!("{} bit/s", nominal),
                    );

                    write_with_header(
                        &mut output,
                        "Data bitrate".green(),
                        &format!("{} bit/s", data),
                    );
                }

                Ok(())
            }
        }
    }
}

/// A more general parser for boolean values such as "enable", "disable", "on"
/// and "off" as well as "true" and "false".
fn parse_enable(arg: &str) -> Result<bool, error::Error> {
    match arg {
        "enable" => Ok(true),
        "true" => Ok(true),
        "on" => Ok(true),
        "disable" => Ok(false),
        "false" => Ok(false),
        "off" => Ok(false),
        _ => Err(error::Error::new(error::ErrorKind::InvalidValue)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_parse() {
        assert_eq!(parse_enable("enable").unwrap(), true);
        assert_eq!(parse_enable("disable").unwrap(), false);
    }
}
