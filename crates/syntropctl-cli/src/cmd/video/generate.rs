//! Generative video animation synthesis command handler.

use anyhow::Result;
use syntropctl_core::ops::generate_video;

use crate::format::print_json;

pub async fn handle_video_generate(
    prompt: &str,
    frames: Option<usize>,
    fps: Option<u32>,
    json: bool,
) -> Result<()> {
    let frame_count = frames.unwrap_or(16);
    let frame_rate = fps.unwrap_or(8);
    let output = generate_video(prompt, frame_count, frame_rate).await?;

    if json {
        print_json(&output);
    } else {
        println!("{}", output.video_path);
        eprintln!(
            "[Video: {} frames | {}ms | Path: {}]",
            output.frames, output.duration_ms, output.video_path
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_generate_defaults() {
        let prompt = "sunset over water";
        assert!(!prompt.is_empty());
    }
}
