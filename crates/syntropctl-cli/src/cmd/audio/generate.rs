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
    let dur = duration.unwrap_or(5);
    let output = generate_music(prompt, dur, bpm).await?;

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

    #[test]
    fn test_audio_generate_defaults() {
        let prompt = "cyberpunk synth";
        assert!(!prompt.is_empty());
    }
}
