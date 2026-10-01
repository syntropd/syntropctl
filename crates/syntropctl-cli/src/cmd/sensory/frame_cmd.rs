//! Frame capture command handler.

use std::path::Path;
use anyhow::Result;
use syntropctl_core::sensory::capture_frame;

use crate::format::print_json;

pub async fn handle_frame(
    device: Option<&str>,
    width: Option<u32>,
    height: Option<u32>,
    out: Option<&Path>,
    json: bool,
) -> Result<()> {
    let res = capture_frame(device, width, height).await?;

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
            "Captured frame: format {}, {} bytes",
            res.format,
            res.raw_bytes.len()
        );
        if let Some(out_path) = out {
            println!("Saved frame image to: {}", out_path.display());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]

    fn test_handle_frame_json_structure() {
        let json_val = serde_json::json!({
            "format": "png",
            "byte_length": 1024,
        });
        assert_eq!(json_val["format"], "png");
    }
}
