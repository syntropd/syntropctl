//! Models command handler.

use anyhow::Result;
use syntropctl_core::ops::query_models;

use crate::format::{print_json, print_models_table};

pub async fn handle_models(json: bool) -> Result<()> {
    let models = query_models().await?;

    if json {
        print_json(&models);
    } else {
        print_models_table(&models);
    }

    Ok(())
}
