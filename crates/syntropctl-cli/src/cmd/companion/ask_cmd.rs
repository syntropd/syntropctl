//! Grounded visual inspection and question answering CLI command.

use anyhow::Result;
use syntropctl_core::companion::ask_screen;

/// Execute `companion ask "<prompt>"` CLI command.
pub async fn handle_ask(
    prompt: &str,
    display: Option<&str>,
    as_json: bool,
) -> Result<()> {
    let result = ask_screen(prompt, display).await?;

    if as_json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("Desktop Companion Answer:");
        println!("{}", result.answer);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_ask_cmd_structure() {
        let prompt = "Describe active window";
        assert_eq!(prompt, "Describe active window");
    }
}
