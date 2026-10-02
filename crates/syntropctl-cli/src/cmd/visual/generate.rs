//! Generative visual synthesis command handler.

use anyhow::Result;
use syntropctl_core::ops::generate_visual;

use crate::format::print_json;

pub async fn handle_visual_generate(
    prompt: &str,
    model: Option<&str>,
    lora: Option<&str>,
    size: Option<&str>,
    json: bool,
) -> Result<()> {
    let (width, height) = parse_dimensions(size);
    let loras = lora.map(|l| vec![l.to_string()]).unwrap_or_default();

    let output = generate_visual(prompt, model, &loras, width, height).await?;

    if json {
        print_json(&output);
    } else {
        println!("{}", output.image_path);
        eprintln!(
            "[Image: {}x{} {} | {} bytes | Path: {}]",
            output.width, output.height, output.format, output.bytes, output.image_path
        );
    }

    Ok(())
}

fn parse_dimensions(size: Option<&str>) -> (u32, u32) {
    size.and_then(|s| {
        let mut parts = s.split('x');
        let w = parts.next()?.trim().parse::<u32>().ok()?;
        let h = parts.next()?.trim().parse::<u32>().ok()?;
        if (1..=4096).contains(&w) && (1..=4096).contains(&h) {
            Some((w, h))
        } else {
            None
        }
    })
    .unwrap_or((512, 512))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dimensions() {
        assert_eq!(parse_dimensions(Some("1024x768")), (1024, 768));
        assert_eq!(parse_dimensions(None), (512, 512));
    }
}
