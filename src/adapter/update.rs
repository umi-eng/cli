//! Firmware update.

use crate::{adapter::UpdateOptions, http::client, write_with_header};
use anyhow::Error;
use colored::Colorize;
use serde::Deserialize;

use super::list::list_devices;

pub async fn command(
    mut output: impl std::io::Write,
    options: UpdateOptions,
) -> anyhow::Result<()> {
    write_with_header(&mut output, "Finding latest firmware...".green(), " ");

    let releases = client()?
        .get("https://api.github.com/repos/umi-eng/adapter/releases")
        .send()
        .await?
        .json::<Vec<Release>>()
        .await?;

    if releases.is_empty() {
        return Err(Error::msg("No releases found."));
    }

    /// Find the first stable release.
    let firmware = if let Some(release) =
        releases.iter().find(|r| !r.draft && !r.prerelease)
    {
        /// Find firmware file in assets.
        if let Some(asset) = release.assets.iter().find(|a| {
            a.name.starts_with("umi-adapter-v") && a.name.ends_with(".bin")
        }) {
            write_with_header(
                &mut output,
                "Downloading firmware...".green(),
                &release.tag_name,
            );
            client()?
                .get(&asset.browser_download_url)
                .send()
                .await?
                .bytes()
                .await?
        } else {
            return Err(Error::msg("Could not find firmware file in release."));
        }
    } else {
        return Err(Error::msg("Could not find stable release."));
    };

    write_with_header(&mut output, "Finding devices...".green(), " ");

    let devices = list_devices()?;

    if devices.is_empty() {
        return Err(Error::msg("No Adapter devices found."));
    }

    write_with_header(&mut output, "Loading new firmware...".green(), " ");
    let device = devices.first().unwrap().open()?;
    let interface = device.claim_interface(4)?; // todo: find proper interface number.
    let mut dfu =
        dfu_nusb::DfuNusb::open(device, interface, 0)?.into_async_dfu();
    dfu.download_from_slice(&firmware).await?;

    write_with_header(&mut output, "Done...".green(), " ");

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct Release {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
pub struct Asset {
    name: String,
    browser_download_url: String,
}
