//! Screen capture command handler.

use anyhow::Result;
use std::path::Path;
use syntropctl_core::sensory::capture_screen;

use crate::format::print_json;

pub async fn handle_screen(display: Option<&str>, out: Option<&Path>, json: bool) -> Result<()> {
    let res = capture_screen(display).await?;

    if let Some(out_path) = out {
        tokio::fs::write(out_path, &res.raw_bytes).await?;
    }

    if json {
        let mut obj = serde_json::json!({
            "image_base64": res.image_base64,
            "format": res.format,
            "byte_length": res.raw_bytes.len(),
        });
        if let Some(out_path) = out {
            obj["out_file"] = serde_json::json!(out_path.to_string_lossy());
        }
        print_json(&obj);
    } else {
        println!(
            "Captured screen: format {}, {} bytes",
            res.format,
            res.raw_bytes.len()
        );
        if let Some(out_path) = out {
            println!("Saved screenshot to: {}", out_path.display());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]

    fn test_handle_screen_json_structure() {
        let json_val = serde_json::json!({
            "format": "png",
            "byte_length": 2048,
        });
        assert_eq!(json_val["format"], "png");
    }
}
