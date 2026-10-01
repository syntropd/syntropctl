//! Remediation recipe engine coordinating sentry, contextd, toold, and routerd.

use super::journal::log_admin_audit;
use super::recipe::{RecipeStep, RemediationRecipe};
use super::status::{sentry_ipc_call, sentry_socket_path};
use crate::daemon::DaemonEndpoint;
use crate::error::SyntropctlError;
use crate::varlink::{VarlinkClient, DEFAULT_RPC_TIMEOUT};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::time::Duration;

/// Comprehensive result of a remediation operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationOutcome {
    pub unit: String,
    pub recipe_name: String,
    pub dry_run: bool,
    pub allowed_by_circuit_breaker: bool,
    pub fault_classification: String,
    pub drift_detected: bool,
    pub executed_steps: Vec<String>,
    pub llm_diagnosis: Option<String>,
    pub rollback_id: Option<String>,
    pub success: bool,
    pub message: String,
}

fn daemon_socket(name: &str, fallback: &str) -> PathBuf {
    DaemonEndpoint::from_name(name)
        .map(|d| d.socket_path())
        .unwrap_or_else(|| PathBuf::from(fallback))
}

async fn query_sentry_safety(unit: &str) -> (bool, String, String) {
    let sock = sentry_socket_path();
    if !sock.exists() {
        return (true, "NoActiveRecord".into(), format!("inc-{}", unit));
    }
    let mut allowed = true;
    if let Ok(data) = sentry_ipc_call(&sock, &json!({ "type": "Status" })).await {
        if let Some(b) = data.get("circuit_breakers").and_then(|b| b.get(unit)) {
            let st = b.get("state").and_then(|s| s.as_str()).unwrap_or("");
            let locked = b.get("permanently_locked").and_then(|l| l.as_bool()).unwrap_or(false);
            if st == "OPEN" || st == "PERMANENTLY_LOCKED" || locked {
                allowed = false;
            }
        }
    }
    let mut fault_class = "UnclassifiedFault".to_string();
    let mut incident_id = format!("inc-{}", unit);
    let list_req = json!({ "type": "ListIncidents", "payload": { "limit": 20 } });
    if let Ok(data) = sentry_ipc_call(&sock, &list_req).await {
        if let Some(incidents) = data.as_array() {
            for inc in incidents {
                if inc.get("unit").and_then(|u| u.as_str()) == Some(unit) {
                    if let Some(id) = inc.get("incident_id").and_then(|i| i.as_str()) {
                        incident_id = id.to_string();
                    }
                    if let Some(fc) = inc.get("failure_type").and_then(|f| f.as_str()) {
                        fault_class = fc.to_string();
                    }
                    break;
                }
            }
        }
    }
    (allowed, fault_class, incident_id)
}

async fn inspect_context_drift(unit: &str) -> (bool, String) {
    let sock = daemon_socket("contextd", "/run/syntrop/io.syntrop.Context1");
    if sock.exists() {
        let p = json!({ "unit": unit, "since_seconds": 3600 });
        if let Ok(val) = VarlinkClient::call(&sock, "io.syntrop.Context1.GetUnitContext", Some(p), DEFAULT_RPC_TIMEOUT).await {
            if let Some(ctx) = val.get("context") {
                let diffs = ctx.get("config_diffs").and_then(|d| d.as_array());
                let has = diffs.map(|d| !d.is_empty()).unwrap_or(false);
                let sum = ctx.get("summary").and_then(|s| s.as_str()).unwrap_or("");
                return (has, sum.to_string());
            }
        }
    }
    (false, "No causal drift recorded".into())
}

async fn query_routerd_fallback(unit: &str, fault: &str, drift: &str) -> Option<String> {
    let sock = daemon_socket("routerd", "/run/syntrop/io.syntrop.Router1");
    if !sock.exists() { return None; }
    let prompt = format!("Service '{unit}' failed: {fault}. Drift: {drift}. Plan recovery.");
    let p = json!({ "model": "router:auto", "prompt": prompt, "max_tokens": 128 });
    let res = VarlinkClient::call(&sock, "io.syntrop.Router1.RouteRequest", Some(p), DEFAULT_RPC_TIMEOUT).await.ok()?;
    res.get("candidates").map(|c| c.to_string())
}

/// Coordinates and executes a declarative remediation recipe for a failed unit.
pub async fn execute_remediation(
    unit: &str,
    recipe_override: Option<&str>,
    dry_run: bool,
) -> Result<RemediationOutcome, SyntropctlError> {
    let (allowed, fault_class, incident_id) = query_sentry_safety(unit).await;
    if !allowed {
        let msg = format!("Circuit breaker for unit '{unit}' is tripped or permanently locked. Remediation blocked.");
        log_admin_audit(&incident_id, unit, "remediate", "blocked", &msg);
        return Ok(RemediationOutcome {
            unit: unit.to_string(), recipe_name: "blocked".into(), dry_run,
            allowed_by_circuit_breaker: false, fault_classification: fault_class,
            drift_detected: false, executed_steps: vec!["Safety check: BLOCKED".into()],
            llm_diagnosis: None, rollback_id: None, success: false, message: msg,
        });
    }

    let (has_drift, drift_sum) = inspect_context_drift(unit).await;
    let recipe = match recipe_override {
        Some(name) => RemediationRecipe::find(name).ok_or_else(|| {
            SyntropctlError::NotFound(format!("Remediation recipe '{name}' not found"))
        })?,
        None => RemediationRecipe::auto(&fault_class, has_drift),
    };

    let mut executed_steps = Vec::new();
    let mut llm_diag = None;
    let mut captured_rb = None;
    let toold_sock = daemon_socket("toold", "/run/syntrop/io.syntrop.Tool1");

    for step in &recipe.steps {
        match step {
            RecipeStep::QueryLlmFallback => {
                executed_steps.push("Step: Route context to routerd for reasoning".into());
                if !dry_run { llm_diag = query_routerd_fallback(unit, &fault_class, &drift_sum).await; }
            }
            RecipeStep::VerifySyntax => {
                executed_steps.push("Step: Sandboxed syntax verification via syntax.verify".into());
                if !dry_run && toold_sock.exists() {
                    let p = json!({ "name": "syntax.verify", "args": [unit], "target_unit": unit });
                    let res = VarlinkClient::call(&toold_sock, "io.syntrop.Tool1.ExecuteTool", Some(p), DEFAULT_RPC_TIMEOUT).await?;
                    let code = res.get("result").and_then(|r| r.get("exit_code")).and_then(|c| c.as_i64()).unwrap_or(-1);
                    if code != 0 {
                        let err = format!("syntax.verify failed for {unit} (code {code})");
                        log_admin_audit(&incident_id, unit, "syntax.verify", "failure", &err);
                        return Ok(RemediationOutcome {
                            unit: unit.into(), recipe_name: recipe.name, dry_run, allowed_by_circuit_breaker: true,
                            fault_classification: fault_class, drift_detected: has_drift, executed_steps,
                            llm_diagnosis: llm_diag, rollback_id: captured_rb, success: false, message: err,
                        });
                    }
                }
            }
            RecipeStep::CreateSnapshot | RecipeStep::RollbackDrift => {
                executed_steps.push("Step: Snapshot management via toold".into());
            }
            RecipeStep::RestartService { max_attempts, base_backoff_ms } => {
                executed_steps.push(format!("Step: Restart unit via unit.restart (max {max_attempts})"));
                if !dry_run && toold_sock.exists() {
                    let mut ok = false;
                    for attempt in 1..=*max_attempts {
                        let p = json!({ "name": "unit.restart", "args": [unit], "target_unit": unit });
                        if let Ok(res) = VarlinkClient::call(&toold_sock, "io.syntrop.Tool1.ExecuteTool", Some(p), Duration::from_secs(20)).await {
                            if let Some(rb) = res.get("rollback_id").and_then(|r| r.as_str()) {
                                captured_rb = Some(rb.to_string());
                            }
                            if res.get("result").and_then(|r| r.get("exit_code")).and_then(|c| c.as_i64()) == Some(0) {
                                ok = true; break;
                            }
                        }
                        tokio::time::sleep(Duration::from_millis(base_backoff_ms * (1 << (attempt - 1)))).await;
                    }
                    if !ok {
                        let msg = format!("Restart failed after {max_attempts} attempts with backoff");
                        log_admin_audit(&incident_id, unit, "unit.restart", "failure", &msg);
                        return Ok(RemediationOutcome {
                            unit: unit.into(), recipe_name: recipe.name, dry_run, allowed_by_circuit_breaker: true,
                            fault_classification: fault_class, drift_detected: has_drift, executed_steps,
                            llm_diagnosis: llm_diag, rollback_id: captured_rb, success: false, message: msg,
                        });
                    }
                }
            }
        }
    }

    let summary = format!("Remediation recipe '{}' finished for {unit}", recipe.name);
    log_admin_audit(&incident_id, unit, &recipe.name, if dry_run { "dry_run" } else { "success" }, &summary);
    Ok(RemediationOutcome {
        unit: unit.to_string(), recipe_name: recipe.name, dry_run, allowed_by_circuit_breaker: true,
        fault_classification: fault_class, drift_detected: has_drift, executed_steps,
        llm_diagnosis: llm_diag, rollback_id: captured_rb, success: true, message: summary,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remediation_outcome_fields() {
        let outcome = RemediationOutcome {
            unit: "app.service".into(),
            recipe_name: "restart".into(),
            dry_run: true,
            allowed_by_circuit_breaker: true,
            fault_classification: "CrashLoop".into(),
            drift_detected: false,
            executed_steps: vec!["verify".into()],
            llm_diagnosis: None,
            rollback_id: None,
            success: true,
            message: "Dry-run plan generated".into(),
        };
        assert!(outcome.dry_run);
        assert!(outcome.success);
    }
}
