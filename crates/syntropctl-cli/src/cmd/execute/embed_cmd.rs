//! Embed command handler.

use anyhow::Result;
use syntropctl_core::ops::embed_text;

use crate::format::print_json;

pub async fn handle_embed(text: &str, model: &str, json: bool) -> Result<()> {
    let vector = embed_text(text, model).await?;

    if json {
        print_json(&serde_json::json!({
            "model": model,
            "dimensions": vector.len(),
            "embedding": vector,
        }));
    } else {
        println!("Embedding Dimensions: {}", vector.len());
        let sample = if vector.len() > 8 { &vector[..8] } else { &vector };
        let formatted: Vec<String> = sample.iter().map(|v| format!("{:.4}", v)).collect();
        println!("Vector Sample (first {}): [{}]", sample.len(), formatted.join(", "));
    }

    Ok(())
}
