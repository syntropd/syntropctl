//! Declarative remediation recipe definitions and planning steps.

use serde::{Deserialize, Serialize};

/// Classification of declarative remediation recipe strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemediationRecipeKind {
    Restart,
    ConfigRollback,
    DaemonReload,
    Auto,
}

/// An individual step inside a remediation recipe execution sequence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "step", rename_all = "snake_case")]
pub enum RecipeStep {
    VerifySyntax,
    CreateSnapshot,
    RestartService {
        max_attempts: usize,
        base_backoff_ms: u64,
    },
    RollbackDrift,
    QueryLlmFallback,
}

/// A structured remediation recipe prescribing self-healing actions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RemediationRecipe {
    pub name: String,
    pub description: String,
    pub kind: RemediationRecipeKind,
    pub steps: Vec<RecipeStep>,
}

impl RemediationRecipe {
    /// Creates a verified restart recipe with syntax checking and backoff.
    pub fn restart() -> Self {
        Self {
            name: "restart".into(),
            description: "Verify unit configuration syntax and restart with exponential backoff"
                .into(),
            kind: RemediationRecipeKind::Restart,
            steps: vec![
                RecipeStep::VerifySyntax,
                RecipeStep::CreateSnapshot,
                RecipeStep::RestartService {
                    max_attempts: 3,
                    base_backoff_ms: 500,
                },
            ],
        }
    }

    /// Creates a configuration rollback recipe reverting recent drift.
    pub fn config_rollback() -> Self {
        Self {
            name: "config-rollback".into(),
            description: "Revert configuration drift, verify syntax, and restart unit".into(),
            kind: RemediationRecipeKind::ConfigRollback,
            steps: vec![
                RecipeStep::CreateSnapshot,
                RecipeStep::RollbackDrift,
                RecipeStep::VerifySyntax,
                RecipeStep::RestartService {
                    max_attempts: 3,
                    base_backoff_ms: 500,
                },
            ],
        }
    }

    /// Creates a daemon reload recipe refreshing systemd state.
    pub fn daemon_reload() -> Self {
        Self {
            name: "daemon-reload".into(),
            description: "Verify syntax and restart service following manager state synchronization"
                .into(),
            kind: RemediationRecipeKind::DaemonReload,
            steps: vec![
                RecipeStep::VerifySyntax,
                RecipeStep::RestartService {
                    max_attempts: 2,
                    base_backoff_ms: 1000,
                },
            ],
        }
    }

    /// Creates an adaptive recipe driven by sentry classification and drift.
    pub fn auto(fault_class: &str, has_drift: bool) -> Self {
        let is_novel = fault_class.is_empty()
            || fault_class.eq_ignore_ascii_case("unknown")
            || fault_class.eq_ignore_ascii_case("unclassified");

        if is_novel {
            Self {
                name: "auto-llm-fallback".into(),
                description:
                    "Novel fault pattern: route context to routerd for diagnostic reasoning".into(),
                kind: RemediationRecipeKind::Auto,
                steps: vec![
                    RecipeStep::QueryLlmFallback,
                    RecipeStep::VerifySyntax,
                    RecipeStep::RestartService {
                        max_attempts: 2,
                        base_backoff_ms: 1000,
                    },
                ],
            }
        } else if has_drift {
            Self::config_rollback()
        } else {
            Self::restart()
        }
    }

    /// Resolves a recipe by its canonical name or shorthand alias.
    pub fn find(name: &str) -> Option<Self> {
        match name.trim().to_lowercase().as_str() {
            "restart" | "service-restart" => Some(Self::restart()),
            "config-rollback" | "rollback" | "drift" => Some(Self::config_rollback()),
            "daemon-reload" | "reload" => Some(Self::daemon_reload()),
            "auto" => Some(Self::auto("", false)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_find_known_names() {
        assert!(RemediationRecipe::find("restart").is_some());
        assert!(RemediationRecipe::find("config-rollback").is_some());
        assert!(RemediationRecipe::find("daemon-reload").is_some());
        assert!(RemediationRecipe::find("auto").is_some());
        assert!(RemediationRecipe::find("nonexistent").is_none());
    }

    #[test]
    fn test_auto_recipe_selects_llm_for_unknown() {
        let rec = RemediationRecipe::auto("Unknown", false);
        assert_eq!(rec.steps[0], RecipeStep::QueryLlmFallback);
    }

    #[test]
    fn test_auto_recipe_selects_rollback_when_drift() {
        let rec = RemediationRecipe::auto("CrashLoop", true);
        assert_eq!(rec.kind, RemediationRecipeKind::ConfigRollback);
    }
}
