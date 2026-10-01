//! Interactive single-keystroke TUI and CLI for pending System One triage decisions.

use super::queue::{get_base_dir, load_incidents, resolve_incident, with_lock};
use rustix::stdio::stdin;
use rustix::termios::{tcgetattr, tcsetattr, OptionalActions};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use tokio::process::Command;

async fn execute_action(unit: &str, action: &str) -> anyhow::Result<()> {
    let act = action.to_ascii_uppercase();
    let sub = if act.contains("RESTART") {
        "restart"
    } else if act.contains("RELOAD") {
        "reload"
    } else if act.contains("RESET") {
        "reset-failed"
    } else {
        return Ok(());
    };
    let status = Command::new("systemctl")
        .args(["--no-ask-password", sub, unit])
        .status()
        .await?;
    if !status.success() {
        anyhow::bail!("systemctl {sub} {unit} failed with status: {status}");
    }
    Ok(())
}

fn read_key() -> char {
    let stdin_fd = stdin();
    if let Ok(orig) = tcgetattr(stdin_fd) {
        let mut raw = orig.clone();
        raw.make_raw();
        let _ = tcsetattr(stdin_fd, OptionalActions::Now, &raw);
        let mut buf = [0u8; 1];
        let _ = std::io::stdin().read_exact(&mut buf);
        let _ = tcsetattr(stdin_fd, OptionalActions::Now, &orig);
        buf[0] as char
    } else {
        let mut line = String::new();
        let _ = std::io::stdin().read_line(&mut line);
        line.chars().next().unwrap_or('q')
    }
}

pub async fn handle_decide(
    approve_id: Option<&str>,
    reject_id: Option<&str>,
    json: bool,
) -> anyhow::Result<()> {
    let base = get_base_dir();
    fs::create_dir_all(base.join("pending"))?;

    if let Some(id) = approve_id {
        return process_targeted(&base, id, true, json).await;
    }
    if let Some(id) = reject_id {
        return process_targeted(&base, id, false, json).await;
    }

    let incidents = with_lock(&base, || Ok(load_incidents(&base)))?;
    if incidents.is_empty() {
        if json {
            println!("{}", serde_json::json!({ "pending": 0 }));
        } else {
            println!("No pending incidents for decision.");
        }
        return Ok(());
    }

    for (path, inc) in incidents {
        let act_str = inc.proposed_action.as_str().unwrap_or("RESTART");
        println!("\nIncident: {}", inc.incident_id);
        println!("Unit:     {}", inc.unit);
        println!("Action:   {} (Confidence: {:.2})", act_str, inc.confidence);
        println!("Details:  {}", inc.explanation);

        loop {
            print!("[y] Approve  [n] Reject  [e] Explain  [m] Escalate  [q] Quit: ");
            std::io::stdout().flush()?;
            let key = read_key();
            println!("{key}");

            match key {
                'y' | 'Y' => {
                    if let Err(e) = execute_action(&inc.unit, act_str).await {
                        eprintln!("Warning: action failed on {}: {e}", inc.unit);
                    }
                    with_lock(&base, || {
                        resolve_incident(&base, &path, &inc, act_str, "approved");
                        Ok(())
                    })?;
                    println!("Approved incident {} on {}", inc.incident_id, inc.unit);
                    break;
                }
                'n' | 'N' => {
                    with_lock(&base, || {
                        resolve_incident(&base, &path, &inc, act_str, "rejected");
                        Ok(())
                    })?;
                    println!("Rejected incident {}", inc.incident_id);
                    break;
                }
                'e' | 'E' => {
                    println!("\n--- Journal Excerpt ---");
                    for line in &inc.journal_excerpt {
                        println!("  {line}");
                    }
                    println!("-----------------------\n");
                }
                'm' | 'M' => {
                    with_lock(&base, || {
                        resolve_incident(&base, &path, &inc, act_str, "escalated");
                        Ok(())
                    })?;
                    println!("Escalated incident {} to operator backlog", inc.incident_id);
                    break;
                }
                'q' | 'Q' | '\x03' | '\x04' => {
                    println!("Exiting decision triage.");
                    return Ok(());
                }
                _ => println!("Invalid choice."),
            }
        }
    }

    Ok(())
}

async fn process_targeted(base: &Path, id: &str, approve: bool, json: bool) -> anyhow::Result<()> {
    let (path, inc) = with_lock(base, || {
        let incidents = load_incidents(base);
        let found = incidents
            .into_iter()
            .find(|(_, inc)| inc.incident_id.starts_with(id));
        found.ok_or_else(|| anyhow::anyhow!("Incident '{id}' not found in pending queue"))
    })?;

    let act_str = inc.proposed_action.as_str().unwrap_or("RESTART");
    if approve {
        if let Err(e) = execute_action(&inc.unit, act_str).await {
            eprintln!("Warning: action failed on {}: {e}", inc.unit);
        }
        with_lock(base, || {
            resolve_incident(base, &path, &inc, act_str, "approved");
            Ok(())
        })?;
        if json {
            println!(
                "{}",
                serde_json::json!({ "status": "approved", "incident_id": inc.incident_id, "action": act_str })
            );
        } else {
            println!("Approved incident {} for {}", inc.incident_id, inc.unit);
        }
    } else {
        with_lock(base, || {
            resolve_incident(base, &path, &inc, act_str, "rejected");
            Ok(())
        })?;
        if json {
            println!(
                "{}",
                serde_json::json!({ "status": "rejected", "incident_id": inc.incident_id })
            );
        } else {
            println!("Rejected incident {}", inc.incident_id);
        }
    }
    Ok(())
}
