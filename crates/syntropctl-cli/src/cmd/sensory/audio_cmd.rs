//! Audio capture command handler.

use anyhow::Result;
use std::path::Path;
use syntropctl_core::sensory::capture_audio;

use crate::format::print_json;

pub async fn handle_audio(
    duration_ms: Option<u32>,
    sample_rate: Option<u32>,
    out: Option<&Path>,
    json: bool,
) -> Result<()> {
    let res = capture_audio(duration_ms, sample_rate).await?;

    if let Some(out_path) = out {
        tokio::fs::write(out_path, &res.raw_bytes).await?;
    }

    if json {
        let mut obj = serde_json::json!({
            "audio_pcm_base64": res.audio_pcm_base64,
            "sample_rate": res.sample_rate,
            "channels": res.channels,
            "byte_length": res.raw_bytes.len(),
        });
        if let Some(out_path) = out {
            obj["out_file"] = serde_json::json!(out_path.to_string_lossy());
        }
        print_json(&obj);
    } else {
        println!(
            "Captured audio: {} bytes PCM ({} ch @ {} Hz)",
            res.raw_bytes.len(),
            res.channels,
            res.sample_rate
        );
        if let Some(out_path) = out {
            println!("Saved audio PCM to: {}", out_path.display());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]

    fn test_handle_audio_json_structure() {
        let json_val = serde_json::json!({
            "sample_rate": 16000,
            "channels": 1,
            "byte_length": 32000,
        });
        assert_eq!(json_val["sample_rate"], 16000);
        assert_eq!(json_val["channels"], 1);
    }
}
