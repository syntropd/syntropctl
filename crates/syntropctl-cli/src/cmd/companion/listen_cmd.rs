//! Ambient listening daemon session CLI command.

use anyhow::Result;
use syntropctl_core::companion::{listen_session, CompanionListenOptions};

/// Execute `companion listen` CLI command.
pub async fn handle_listen(
    voice: bool,
    hotkey: Option<&str>,
    once: bool,
    as_json: bool,
) -> Result<()> {
    let opts = CompanionListenOptions {
        voice,
        hotkey: hotkey.map(ToString::to_string),
        once,
    };

    let events = listen_session(&opts).await?;

    if as_json {
        println!("{}", serde_json::to_string_pretty(&events)?);
    } else {
        println!("Companion Listen Session: {} event(s)", events.len());
        for ev in &events {
            println!("  [{}] {}", ev.trigger_type, ev.description);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_listen_cmd_module_exists() {
        let opts = CompanionListenOptions::default();
        assert!(!opts.voice);
    }
}
