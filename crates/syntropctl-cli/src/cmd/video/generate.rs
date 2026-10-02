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
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
        anyhow::bail!("prompt cannot be empty");
    }
    if let Some(f) = frames {
        if f == 0 {
            anyhow::bail!("frames must be greater than 0");
        }
    }
    if let Some(rate) = fps {
        if rate == 0 {
            anyhow::bail!("fps must be greater than 0");
        }
    }

    let frame_count = frames.unwrap_or(16);
    let frame_rate = fps.unwrap_or(8);
    let output = generate_video(trimmed, frame_count, frame_rate).await?;

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

    #[tokio::test]
    async fn test_video_generate_validation() {
        let res_empty = handle_video_generate("   ", Some(16), None, false).await;
        assert!(res_empty.is_err());
        assert!(res_empty.unwrap_err().to_string().contains("prompt cannot be empty"));

        let res_zero_frames = handle_video_generate("ocean", Some(0), None, false).await;
        assert!(res_zero_frames.is_err());
        assert!(res_zero_frames.unwrap_err().to_string().contains("frames must be greater than 0"));

        let res_zero_fps = handle_video_generate("ocean", Some(16), Some(0), false).await;
        assert!(res_zero_fps.is_err());
        assert!(res_zero_fps.unwrap_err().to_string().contains("fps must be greater than 0"));
    }
}
