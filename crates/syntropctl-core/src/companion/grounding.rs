//! Vision-guided desktop UI element grounding and normalized coordinate discovery.

use super::router_stream::query_router_multimodal;
use crate::error::SyntropctlError;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::{Deserialize, Serialize};

/// Grounded UI element coordinate detection payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroundedElement {
    pub element: String,
    pub point: [f32; 2],
    pub confidence: f32,
}

/// Parses grounded point coordinates from multimodal model JSON response.
pub fn parse_grounding_response(response: &str) -> Option<(f32, f32)> {
    let cleaned = response
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    if let Ok(grounded) = serde_json::from_str::<GroundedElement>(cleaned) {
        return Some((
            grounded.point[0].clamp(0.0, 1.0),
            grounded.point[1].clamp(0.0, 1.0),
        ));
    }

    if let (Some(start), Some(end)) = (cleaned.find('{'), cleaned.rfind('}')) {
        if start < end {
            if let Ok(grounded) = serde_json::from_str::<GroundedElement>(&cleaned[start..=end]) {
                return Some((
                    grounded.point[0].clamp(0.0, 1.0),
                    grounded.point[1].clamp(0.0, 1.0),
                ));
            }
        }
    }
    None
}

/// Ground a natural language element instruction against screenshot bytes.
pub async fn ground_ui_element(
    instruction: &str,
    image_png: &[u8],
) -> Result<Option<(f32, f32)>, SyntropctlError> {
    let b64 = BASE64.encode(image_png);
    let prompt = format!(
        "Locate the UI element for instruction: \"{}\". Output ONLY a JSON object: {{\"element\": \"...\", \"point\": [x, y], \"confidence\": 0.95}} with normalized coordinates x, y in range [0.0, 1.0]. Do not include markdown fences.",
        instruction
    );
    let system = "You are an accurate desktop UI vision grounding engine. Find the target element and return its normalized center point [x, y] in [0.0, 1.0].";

    match query_router_multimodal(&prompt, &b64, Some(system)).await {
        Ok(reply) => {
            if let Some(pt) = parse_grounding_response(&reply) {
                return Ok(Some(pt));
            }
        }
        Err(_) => {
            // Routerd multimodal unavailable; try local OCR grounding fallback
            if let Some(actions) = super::execute::discover_ui_element_fast(instruction, &b64).await {
                for act in actions {
                    if let super::execute::UiAction::MoveMouse { x, y } = act {
                        return Ok(Some((x, y)));
                    }
                }
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_grounding_response_clean_json() {
        let raw = r#"{"element": "save_button", "point": [0.45, 0.82], "confidence": 0.96}"#;
        let pt = parse_grounding_response(raw).unwrap();
        assert!((pt.0 - 0.45).abs() < 1e-4);
        assert!((pt.1 - 0.82).abs() < 1e-4);
    }

    #[test]
    fn test_parse_grounding_response_fenced_and_clamped() {
        let raw = "```json\n{\"element\": \"close\", \"point\": [-0.1, 1.5], \"confidence\": 0.88}\n```";
        let pt = parse_grounding_response(raw).unwrap();
        assert_eq!(pt, (0.0, 1.0));
    }
}
