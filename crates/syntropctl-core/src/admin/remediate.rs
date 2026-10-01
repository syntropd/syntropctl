//! Remediation recipe engine coordinating sentry, contextd, toold, and routerd.

use super::journal::log_admin_audit;
use super::query_fault::{inspect_context_drift, query_routerd_fallback, query_sentry_safety};
use super::recipe::{RecipeStep, RemediationRecipe};
use super::rollback::execute_rollback;
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
                if !dry_run {
                    if !toold_sock.exists() {
                        let msg = format!("syntax.verify cannot run for {unit}: toold sandbox socket unavailable");
                        log_admin_audit(&incident_id, unit, "syntax.verify", "failure", &msg);
                        return Ok(RemediationOutcome {
                            unit: unit.into(), recipe_name: recipe.name, dry_run, allowed_by_circuit_breaker: true,
                            fault_classification: fault_class, drift_detected: has_drift, executed_steps,
                            llm_diagnosis: llm_diag, rollback_id: captured_rb, success: false, message: msg,
                        });
                    }
                    let p = json!({ "name": "syntax.verify", "args": [unit], "target_unit": unit });
                    let verify_res = VarlinkClient::call(&toold_sock, "io.syntrop.Tool1.ExecuteTool", Some(p), DEFAULT_RPC_TIMEOUT).await;
                    let failed_err = match verify_res {
                        Ok(res) => {
                            let code = res.get("result").and_then(|r| r.get("exit_code")).and_then(|c| c.as_i64()).unwrap_or(0);
                            if code != 0 { Some(format!("syntax.verify failed for {unit} (code {code})")) } else { None }
                        }
                        Err(e) => Some(format!("syntax.verify failed for {unit}: {e}")),
                    };
                    if let Some(err) = failed_err {
                        log_admin_audit(&incident_id, unit, "syntax.verify", "failure", &err);
                        return Ok(RemediationOutcome {
                            unit: unit.into(), recipe_name: recipe.name, dry_run, allowed_by_circuit_breaker: true,
                            fault_classification: fault_class, drift_detected: has_drift, executed_steps,
                            llm_diagnosis: llm_diag, rollback_id: captured_rb, success: false, message: err,
                        });
                    }
                }
            }
            RecipeStep::CreateSnapshot => {
                executed_steps.push("Step: Snapshot management via toold".into());
            }
            RecipeStep::RollbackDrift => {
                executed_steps.push(format!("Step: Revert configuration drift for {unit} via toold"));
                if !dry_run {
                    if let Ok(outcome) = execute_rollback(unit, None).await {
                        captured_rb = Some(outcome.rollback_id.clone());
                        executed_steps.push(format!("Step: Restored snapshot {}", outcome.rollback_id));
                    }
                }
            }
            RecipeStep::RestartService { max_attempts, base_backoff_ms } => {
                executed_steps.push(format!("Step: Restart unit via unit.restart (max {max_attempts})"));
                if !dry_run {
                    if !toold_sock.exists() {
                        let msg = format!("Cannot restart unit {unit}: toold sandbox socket unavailable");
                        log_admin_audit(&incident_id, unit, "unit.restart", "failure", &msg);
                        return Ok(RemediationOutcome {
                            unit: unit.into(), recipe_name: recipe.name, dry_run, allowed_by_circuit_breaker: true,
                            fault_classification: fault_class, drift_detected: has_drift, executed_steps,
                            llm_diagnosis: llm_diag, rollback_id: captured_rb, success: false, message: msg,
                        });
                    }
                    let mut ok = false;
                    let mut last_err = String::new();
                    for attempt in 1..=*max_attempts {
                        let p = json!({ "name": "unit.restart", "args": [unit], "target_unit": unit });
                        match VarlinkClient::call(&toold_sock, "io.syntrop.Tool1.ExecuteTool", Some(p), Duration::from_secs(20)).await {
                            Ok(res) => {
                                if let Some(rb) = res.get("rollback_id").and_then(|r| r.as_str()) {
                                    captured_rb = Some(rb.to_string());
                                }
                                if res.get("result").and_then(|r| r.get("exit_code")).and_then(|c| c.as_i64()) == Some(0) {
                                    ok = true; break;
                                } else {
                                    last_err = format!("attempt {attempt} non-zero exit");
                                }
                            }
                            Err(e) => { last_err = format!("attempt {attempt}: {e}"); }
                        }
                        if attempt < *max_attempts {
                            tokio::time::sleep(Duration::from_millis(base_backoff_ms * (1 << (attempt - 1)))).await;
                        }
                    }
                    if !ok {
                        let msg = format!("Restart failed after {max_attempts} attempts: {last_err}");
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
