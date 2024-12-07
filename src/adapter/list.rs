use crate::write_with_header;
use colored::Colorize;
use nusb::DeviceInfo;
use nusb::Error;

/// List attached Adapter devices.
pub fn list_devices() -> Result<Vec<DeviceInfo>, Error> {
    Ok(nusb::list_devices()?
        .filter(|dev| dev.vendor_id() == 0x1209 && dev.product_id() == 0x2323)
        .filter(|dev| {
            dev.manufacturer_string() == Some("Universal Machine Intelligence")
        })
        .collect())
}

pub async fn command(mut output: impl std::io::Write) -> anyhow::Result<()> {
    let devices = list_devices()?;

    if devices.is_empty() {
        write_with_header(
            &mut output,
            "No Adapter devices found...".green(),
            " ",
        );
        return Ok(());
    }

    println!("{0: <10} {1: <10}", "Serial No.", "Product");
    devices.iter().for_each(|dev| {
        println!(
            "{0: <10} {1: <10}",
            dev.serial_number().unwrap_or("-"),
            dev.product_string().unwrap_or("-")
        )
    });

    Ok(())
}
