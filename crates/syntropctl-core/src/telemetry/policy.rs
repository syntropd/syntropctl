//! Tuning policy persistence and governor state management.

use super::types::{TuningConfig, TuningOutcome, TuningPolicy};
use crate::error::SyntropctlError;
use std::fs;
use std::path::{Path, PathBuf};

const TUNING_RUN_PATH: &str = "/run/syntrop/tuning.json";
const TUNING_ETC_PATH: &str = "/etc/syntrop/tuning.json";

/// Returns candidate paths for the tuning configuration file.
pub fn tuning_config_paths() -> Vec<PathBuf> {
    vec![PathBuf::from(TUNING_RUN_PATH), PathBuf::from(TUNING_ETC_PATH)]
}

/// Loads the currently active tuning configuration, defaulting to Balanced if none found.
pub fn load_active_tuning_config() -> TuningConfig {
    for path in tuning_config_paths() {
        if path.exists() {
            if let Ok(bytes) = fs::read(&path) {
                if let Ok(cfg) = serde_json::from_slice::<TuningConfig>(&bytes) {
                    return cfg;
                }
            }
        }
    }
    TuningConfig::default()
}

/// Saves the tuning configuration to the active filesystem location.
pub fn apply_tuning_policy(policy: TuningPolicy) -> Result<TuningOutcome, SyntropctlError> {
    let cfg = policy.to_config();
    let json = serde_json::to_string_pretty(&cfg)
        .map_err(|e| SyntropctlError::OperationFailed(format!("failed to serialize tuning config: {e}")))?;

    let mut chosen_path = None;
    for target in [TUNING_RUN_PATH, TUNING_ETC_PATH] {
        let p = Path::new(target);
        if let Some(parent) = p.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if fs::write(p, json.as_bytes()).is_ok() {
            chosen_path = Some(target.to_string());
            break;
        }
    }

    let path = chosen_path.unwrap_or_else(|| TUNING_RUN_PATH.to_string());
    Ok(TuningOutcome {
        applied: true,
        path,
        config: cfg,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_default_tuning_config() {
        let cfg = load_active_tuning_config();
        assert_eq!(cfg.policy, TuningPolicy::Balanced);
        assert_eq!(cfg.k_draft_horizon, 4);
    }

    #[test]
    fn test_candidate_paths() {
        let paths = tuning_config_paths();
        assert_eq!(paths.len(), 2);
    }
}
