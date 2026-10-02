//! Generative visual synthesis command handler.

use anyhow::Result;
use syntropctl_core::ops::generate_visual;

use crate::format::print_json;

pub async fn handle_visual_generate(
    prompt: &str,
    model: Option<&str>,
    lora: Option<&str>,
    size: Option<&str>,
    storyboard: Option<usize>,
    allow_degrade: bool,
    json: bool,
) -> Result<()> {
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
        anyhow::bail!("prompt cannot be empty");
    }
    let (width, height) = parse_dimensions(size)?;
    let loras = lora.map(|l| vec![l.to_string()]).unwrap_or_default();

    let output = generate_visual(
        trimmed,
        model,
        &loras,
        width,
        height,
        storyboard,
        allow_degrade,
    )
    .await?;

    if json {
        print_json(&output);
    } else if let Some(sb_path) = &output.storyboard_path {
        println!("{}", sb_path);
        let kf = output.keyframes.unwrap_or(0);
        let mf = output.manifest_path.as_deref().unwrap_or("-");
        eprintln!("[Storyboard: {kf} keyframes | {width}x{height} | Manifest: {mf} | Path: {sb_path}]");
    } else {
        println!("{}", output.image_path);
        eprintln!(
            "[Image: {}x{} {} | {} bytes | Path: {}]",
            output.width, output.height, output.format, output.bytes, output.image_path
        );
    }

    Ok(())
}

fn parse_dimensions(size: Option<&str>) -> Result<(u32, u32)> {
    match size {
        None => Ok((512, 512)),
        Some(s) => {
            let mut parts = s.split('x');
            let w = parts.next().and_then(|p| p.trim().parse::<u32>().ok());
            let h = parts.next().and_then(|p| p.trim().parse::<u32>().ok());
            match (w, h, parts.next()) {
                (Some(w), Some(h), None) if (1..=4096).contains(&w) && (1..=4096).contains(&h) => {
                    Ok((w, h))
                }
                _ => anyhow::bail!("invalid size '{s}', expected WxH (1..=4096)"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dimensions() {
        assert_eq!(parse_dimensions(Some("1024x768")).unwrap(), (1024, 768));
        assert_eq!(parse_dimensions(None).unwrap(), (512, 512));
        assert!(parse_dimensions(Some("bad")).is_err());
        assert!(parse_dimensions(Some("9999x9999")).is_err());
    }
}
