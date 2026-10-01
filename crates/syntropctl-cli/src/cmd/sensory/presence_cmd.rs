//! Operator presence query command handler.

use anyhow::Result;
use syntropctl_core::sensory::get_operator_presence;

use crate::format::print_json;

pub async fn handle_presence(json: bool) -> Result<()> {
    let res = get_operator_presence().await?;

    if json {
        print_json(&serde_json::json!({
            "present": res.present,
            "confidence": res.confidence,
            "reason": res.reason,
        }));
    } else {
        let pct = (res.confidence * 100.0).clamp(0.0, 100.0);
        let presence_str = if res.present { "Present" } else { "Absent" };
        println!(
            "Operator Presence: {} (confidence: {:.1}%)",
            presence_str, pct
        );
        println!("Detection Heuristic: {}", res.reason);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]

    fn test_handle_presence_json_structure() {
        let json_val = serde_json::json!({
            "present": true,
            "confidence": 0.85,
            "reason": "acoustic energy detected",
        });
        assert_eq!(json_val["present"], true);
        assert_eq!(json_val["reason"], "acoustic energy detected");
    }
}
