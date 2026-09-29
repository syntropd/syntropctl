//! Models command handler inspecting unified local and cached catalog.

use anyhow::Result;
use serde_json::json;
use syntropctl_core::ops::{query_models, query_storage_stats};

use crate::format::{print_json, print_models_table_with_stats};

/// Dispatches the `syntropctl models` command, querying active and cached models with CAS stats.
pub async fn handle_models(json: bool) -> Result<()> {
    let models = query_models().await?;
    let stats = query_storage_stats().await.ok();

    if json {
        if let Some(s) = &stats {
            print_json(&json!({
                "models": models,
                "storage": s,
            }));
        } else {
            print_json(&models);
        }
    } else {
        print_models_table_with_stats(&models, stats.as_ref());
    }

    Ok(())
}
