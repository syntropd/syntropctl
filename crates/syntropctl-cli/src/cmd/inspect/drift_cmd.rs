//! Drift command handler.

use anyhow::Result;
use syntropctl_core::ops::query_drift;

use crate::format::{print_drift_table, print_json};

pub async fn handle_drift(unit: Option<&str>, json: bool) -> Result<()> {
    let events = query_drift(unit).await?;

    if json {
        print_json(&events);
    } else {
        print_drift_table(&events);
    }

    Ok(())
}
