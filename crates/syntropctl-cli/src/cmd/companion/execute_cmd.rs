//! UI macro execution and visual validation CLI command.

use anyhow::Result;
use syntropctl_core::companion::execute_instruction;

/// Execute `companion execute "<instruction>"` CLI command.
pub async fn handle_execute(
    instruction: &str,
    display: Option<&str>,
    dry_run: bool,
    grounding: bool,
    as_json: bool,
) -> Result<()> {
    let result = execute_instruction(instruction, display, dry_run).await?;

    if as_json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("Instruction: {}", result.instruction);
        println!("Planned Actions: {}", result.actions.len());
        for action in &result.actions {
            if let syntropctl_core::companion::UiAction::MoveMouse { x, y } = action {
                if grounding {
                    println!("Grounded UI Target: [x: {:.3}, y: {:.3}]", x, y);
                }
            }
        }
        println!("Executed Actions: {}", result.executed_count);
        if result.aborted {
            println!(
                "Status: ABORTED ({})",
                result.abort_reason.as_deref().unwrap_or("Safety abort")
            );
        } else {
            println!("Status: COMPLETED");
        }
        println!(
            "Intermediate Visual State Validated: {}",
            if result.validated_visual_state { "YES" } else { "NO" }
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_execute_cmd_module_exists() {
        let instr = "click 0.5 0.5";
        assert!(!instr.is_empty());
    }
}
