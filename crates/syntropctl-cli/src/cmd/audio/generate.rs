//! Generative audio and music synthesis command handler.

use anyhow::Result;
use syntropctl_core::ops::generate_music;

use crate::format::print_json;

pub async fn handle_audio_generate(
    prompt: &str,
    duration: Option<u32>,
    bpm: Option<u32>,
    json: bool,
) -> Result<()> {
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
        anyhow::bail!("prompt cannot be empty");
    }
    if let Some(d) = duration {
        if d == 0 {
            anyhow::bail!("duration must be greater than 0");
        }
    }
    if let Some(b) = bpm {
        if b == 0 {
            anyhow::bail!("bpm must be greater than 0");
        }
    }

    let dur = duration.unwrap_or(5);
    let output = generate_music(trimmed, dur, bpm).await?;

    if json {
        print_json(&output);
    } else {
        println!("{}", output.audio_path);
        eprintln!(
            "[Audio: {}Hz stereo | {}ms | Path: {}]",
            output.sample_rate, output.duration_ms, output.audio_path
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audio_generate_validation() {
        let res_empty = handle_audio_generate("   ", Some(5), None, false).await;
        assert!(res_empty.is_err());
        assert!(res_empty.unwrap_err().to_string().contains("prompt cannot be empty"));

        let res_zero_dur = handle_audio_generate("synth", Some(0), None, false).await;
        assert!(res_zero_dur.is_err());
        assert!(res_zero_dur.unwrap_err().to_string().contains("duration must be greater than 0"));

        let res_zero_bpm = handle_audio_generate("synth", Some(5), Some(0), false).await;
        assert!(res_zero_bpm.is_err());
        assert!(res_zero_bpm.unwrap_err().to_string().contains("bpm must be greater than 0"));
    }
}
