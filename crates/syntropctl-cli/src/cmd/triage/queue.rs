//! Advisory-locked disk queue management and structured audit recording.

use rustix::fs::{flock, FlockOperation};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::net::UnixDatagram;
use std::path::{Path, PathBuf};

/// Pending incident loaded from disk queue.
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

/// Returns the base runtime directory (/run/syntrop by default).
pub fn get_base_dir() -> PathBuf {
    std::env::var("RUNTIME_DIRECTORY")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/run/syntrop"))
}

/// Executes a closure holding an exclusive advisory flock on pending.lock.
pub fn with_lock<T, F: FnOnce() -> anyhow::Result<T>>(base: &Path, f: F) -> anyhow::Result<T> {
    let lock_path = base.join("pending.lock");
    let lock_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)?;
    flock(&lock_file, FlockOperation::LockExclusive)
        .map_err(|e| anyhow::anyhow!("failed to acquire pending.lock: {e}"))?;
    let res = f();
    let _ = flock(&lock_file, FlockOperation::Unlock);
    res
}

/// Scans pending directory and atomically refreshes pending_count file.
pub fn update_pending_count(base: &Path) {
    let pending_dir = base.join("pending");
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(&pending_dir) {
        for entry in entries.flatten() {
            if entry.path().extension().is_some_and(|e| e == "json") {
                count += 1;
            }
        }
    }
    let count_path = base.join("pending_count");
    let tmp = count_path.with_extension("tmp");
    if let Ok(mut f) = File::create(&tmp) {
        let _ = writeln!(f, "{count}");
        let _ = f.sync_all();
        let _ = fs::rename(tmp, count_path);
    }
}

/// Loads all staged pending incidents ordered by arrival timestamp.
pub fn load_incidents(base: &Path) -> Vec<(PathBuf, PendingIncident)> {
    let pending_dir = base.join("pending");
    let mut list = Vec::new();
    if let Ok(entries) = fs::read_dir(pending_dir) {
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

/// Emits audit records to both persistent audit.log and journald socket.
pub fn append_audit(inc: &PendingIncident, action: &str, status: &str) {
    let base = get_base_dir();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_default();
    let fault = inc.fault_class.as_str().unwrap_or("TransientRestart");
    let record = serde_json::json!({
        "incident_id": inc.incident_id,
        "timestamp": ts,
        "unit": inc.unit,
        "fault_class": fault,
        "tier": "OPERATOR",
        "action": action,
        "confidence": inc.confidence,
        "status": status,
        "explanation": inc.explanation,
    });
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(base.join("audit.log")) {
        let _ = writeln!(f, "{record}");
    }
    let payload = format!(
        "MESSAGE=Operator decision for {}: {} ({})\n\
         PRIORITY=6\n\
         SYSLOG_IDENTIFIER=systemd-sentry\n\
         SYNTROP_INCIDENT_ID={}\n\
         SYNTROP_UNIT={}\n\
         SYNTROP_FAULT_CLASS={}\n\
         SYNTROP_TIER=OPERATOR\n\
         SYNTROP_ACTION={}\n\
         SYNTROP_STATUS={}\n\
         SYNTROP_CONFIDENCE={:.4}\n",
        inc.unit, action, status, inc.incident_id, inc.unit, fault, action, status, inc.confidence
    );
    let sock = "/run/systemd/journal/socket";
    if Path::new(sock).exists() {
        if let Ok(s) = UnixDatagram::unbound() {
            let _ = s.send_to(payload.as_bytes(), sock);
        }
    }
}

/// Deletes processed pending incident file, refreshes count, and logs audit.
pub fn resolve_incident(base: &Path, path: &Path, inc: &PendingIncident, act: &str, status: &str) {
    let _ = fs::remove_file(path);
    update_pending_count(base);
    append_audit(inc, act, status);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_lifecycle_under_lock() {
        let dir = std::env::temp_dir().join(format!("test_queue_{}", std::process::id()));
        let _ = fs::create_dir_all(dir.join("pending"));

        let inc = PendingIncident {
            incident_id: "test-123".into(),
            unit: "demo.service".into(),
            timestamp: "2026-09-30T10:00:00Z".into(),
            fault_class: serde_json::json!("TransientRestart"),
            confidence: 0.85,
            tier: serde_json::json!("MEDIUM"),
            proposed_action: serde_json::json!("RESTART"),
            explanation: "test incident".into(),
            journal_excerpt: vec!["line 1".into()],
        };

        let file_path = dir.join("pending").join("test-123.json");
        fs::write(&file_path, serde_json::to_string(&inc).unwrap()).unwrap();

        with_lock(&dir, || {
            update_pending_count(&dir);
            let loaded = load_incidents(&dir);
            assert_eq!(loaded.len(), 1);
            assert_eq!(loaded[0].1.incident_id, "test-123");
            resolve_incident(&dir, &file_path, &loaded[0].1, "RESTART", "approved");
            assert_eq!(load_incidents(&dir).len(), 0);
            Ok(())
        })
        .unwrap();

        let _ = fs::remove_dir_all(&dir);
    }
}
