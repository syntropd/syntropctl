//! Interactive single-keystroke TUI and CLI for pending System One triage decisions.

use rustix::stdio::stdin;
use rustix::termios::{tcgetattr, tcsetattr, OptionalActions};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tokio::process::Command;

/// Pending incident loaded from disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingIncident {
    pub incident_id: String,
    pub unit: String,
    pub timestamp: String,
    pub fault_class: serde_json::Value,
    pub confidence: f32,
    pub tier: serde_json::Value,
    pub proposed_action: serde_json::Value,
    pub explanation: String,
    #[serde(default)]
    pub journal_excerpt: Vec<String>,
}

fn get_pending_dir() -> PathBuf {
    let base = std::env::var("RUNTIME_DIRECTORY")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/run/syntrop"));
    base.join("pending")
}

fn get_count_file() -> PathBuf {
    let base = std::env::var("RUNTIME_DIRECTORY")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/run/syntrop"));
    base.join("pending_count")
}

fn update_pending_count(dir: &Path) {
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.path().extension().is_some_and(|e| e == "json") {
                count += 1;
            }
        }
    }
    let count_path = get_count_file();
    let tmp = count_path.with_extension("tmp");
    if let Ok(mut f) = File::create(&tmp) {
        let _ = write!(f, "{count}\n");
        let _ = f.sync_all();
        let _ = fs::rename(tmp, count_path);
    }
}

fn load_incidents(dir: &Path) -> Vec<(PathBuf, PendingIncident)> {
    let mut list = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().is_some_and(|e| e == "json") {
                if let Ok(data) = fs::read_to_string(&p) {
                    if let Ok(inc) = serde_json::from_str::<PendingIncident>(&data) {
                        list.push((p, inc));
                    }
                }
            }
        }
    }
    list.sort_by(|a, b| a.1.timestamp.cmp(&b.1.timestamp));
    list
}

async fn execute_action(unit: &str, action: &str) -> anyhow::Result<()> {
    let act = action.to_ascii_uppercase();
    if act.contains("RESTART") {
        let _ = Command::new("systemctl").args(["restart", unit]).status().await?;
    } else if act.contains("RELOAD") {
        let _ = Command::new("systemctl").args(["reload", unit]).status().await?;
    } else if act.contains("RESET") {
        let _ = Command::new("systemctl").args(["reset-failed", unit]).status().await?;
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
    let pending_dir = get_pending_dir();
    fs::create_dir_all(&pending_dir)?;

    if let Some(id) = approve_id {
        return process_targeted(&pending_dir, id, true, json).await;
    }
    if let Some(id) = reject_id {
        return process_targeted(&pending_dir, id, false, json).await;
    }

    let incidents = load_incidents(&pending_dir);
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
                    execute_action(&inc.unit, act_str).await?;
                    let _ = fs::remove_file(&path);
                    update_pending_count(&pending_dir);
                    append_audit(&inc.incident_id, &inc.unit, act_str, "approved", &inc.explanation);
                    println!("Approved and executed {} on {}", act_str, inc.unit);
                    break;
                }
                'n' | 'N' => {
                    let _ = fs::remove_file(&path);
                    update_pending_count(&pending_dir);
                    append_audit(&inc.incident_id, &inc.unit, act_str, "rejected", &inc.explanation);
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
                    let _ = fs::remove_file(&path);
                    update_pending_count(&pending_dir);
                    append_audit(&inc.incident_id, &inc.unit, act_str, "escalated", &inc.explanation);
                    println!("Escalated incident {} to operator backlog", inc.incident_id);
                    break;
                }
                'q' | 'Q' => {
                    println!("Exiting decision triage.");
                    return Ok(());
                }
                _ => println!("Invalid choice."),
            }
        }
    }

    Ok(())
}

async fn process_targeted(dir: &Path, id: &str, approve: bool, json: bool) -> anyhow::Result<()> {
    let incidents = load_incidents(dir);
    let found = incidents.into_iter().find(|(_, inc)| inc.incident_id.starts_with(id));

    let (path, inc) = match found {
        Some(item) => item,
        None => {
            anyhow::bail!("Incident '{}' not found in pending queue", id);
        }
    };

    let act_str = inc.proposed_action.as_str().unwrap_or("RESTART");
    if approve {
        execute_action(&inc.unit, act_str).await?;
        let _ = fs::remove_file(&path);
        update_pending_count(dir);
        append_audit(&inc.incident_id, &inc.unit, act_str, "approved", &inc.explanation);
        if json {
            println!("{}", serde_json::json!({ "status": "approved", "incident_id": inc.incident_id, "action": act_str }));
        } else {
            println!("Approved and executed {} for {}", act_str, inc.unit);
        }
    } else {
        let _ = fs::remove_file(&path);
        update_pending_count(dir);
        append_audit(&inc.incident_id, &inc.unit, act_str, "rejected", &inc.explanation);
        if json {
            println!("{}", serde_json::json!({ "status": "rejected", "incident_id": inc.incident_id }));
        } else {
            println!("Rejected incident {}", inc.incident_id);
        }
    }
    Ok(())
}

fn append_audit(id: &str, unit: &str, action: &str, status: &str, explanation: &str) {
    let base = std::env::var("RUNTIME_DIRECTORY")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/run/syntrop"));
    let log_path = base.join("audit.log");
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_default();
    let record = serde_json::json!({
        "incident_id": id,
        "timestamp": ts,
        "unit": unit,
        "action": action,
        "status": status,
        "explanation": explanation,
    });
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(log_path) {
        let _ = writeln!(f, "{record}");
    }
}
