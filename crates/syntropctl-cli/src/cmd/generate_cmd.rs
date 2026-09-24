//! Generate command handler.

use anyhow::Result;
use syntropctl_core::ops::generate_text;

use crate::format::print_json;

pub async fn handle_generate(
    prompt: &str,
    model: &str,
    max_tokens: usize,
    temperature: f32,
    json: bool,
) -> Result<()> {
    let output = generate_text(prompt, model, max_tokens, temperature).await?;

    if json {
        print_json(&output);
    } else {
        println!("{}", output.text);
        eprintln!(
            "\n[Prompt: {} tokens | Completion: {} tokens | Completed in {}ms]",
            output.prompt_tokens, output.completion_tokens, output.duration_ms
        );
    }

    Ok(())
}
