//! Ambient listening daemon session CLI command.

use anyhow::Result;
use syntropctl_core::companion::{
    listen_session_with_callback, CompanionListenEvent, CompanionListenOptions,
};

/// Execute `companion listen` CLI command.
pub async fn handle_listen(
    voice: bool,
    hotkey: Option<&str>,
    once: bool,
    as_json: bool,
) -> Result<()> {
    let effective_voice = voice || hotkey.is_none();
    let opts = CompanionListenOptions {
        voice: effective_voice,
        hotkey: hotkey.map(ToString::to_string),
        once,
    };

    if !once && !as_json {
        println!(
            "Companion Listen Session active (voice={}, hotkey={:?}). Listening...",
            opts.voice, opts.hotkey
        );
    }

    let on_event = move |ev: &CompanionListenEvent| {
        if as_json {
            if let Ok(line) = serde_json::to_string(ev) {
                println!("{}", line);
            }
        } else {
            println!("  [{}] {}", ev.trigger_type, ev.description);
        }
    };

    let events = listen_session_with_callback(&opts, on_event).await?;

    if once && as_json {
        println!("{}", serde_json::to_string_pretty(&events)?);
    } else if once && !as_json {
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
