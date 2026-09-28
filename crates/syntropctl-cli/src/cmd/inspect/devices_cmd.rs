//! Devices command handler.

use anyhow::Result;
use syntropctl_core::ops::query_devices;

use crate::format::{print_devices_table, print_json};

pub async fn handle_devices(json: bool) -> Result<()> {
    let devices = query_devices().await?;

    if json {
        print_json(&devices);
    } else {
        print_devices_table(&devices);
    }

    Ok(())
}
