//! Explain command handler.

use anyhow::Result;
use syntropctl_core::ops::explain_unit;

use crate::format::{print_incident_report, print_json};

pub async fn handle_explain(unit: &str, json: bool) -> Result<()> {
    let report = explain_unit(unit).await?;

    if json {
        print_json(&report);
    } else {
        print_incident_report(&report);
    }

    Ok(())
}
