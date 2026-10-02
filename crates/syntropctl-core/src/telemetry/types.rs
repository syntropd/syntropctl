//! Telemetry domain types, pressure thresholds, and tuning policies.

use serde::{Deserialize, Serialize};

/// Dynamic closed-loop tuning policy presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TuningPolicy {
    /// Conservative: tighter pressure thresholds and constrained draft horizons.
    Conservative,
    /// Balanced: default operational profile balancing latency and throughput.
    #[default]
    Balanced,
    /// Aggressive: higher pressure thresholds allowing wider draft horizons.
    Aggressive,
}

impl TuningPolicy {
    pub fn parse_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "conservative" => Some(Self::Conservative),
            "balanced" => Some(Self::Balanced),
            "aggressive" => Some(Self::Aggressive),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Conservative => "conservative",
            Self::Balanced => "balanced",
            Self::Aggressive => "aggressive",
        }
    }

    pub fn to_config(&self) -> TuningConfig {
        match self {
            Self::Conservative => TuningConfig {
                policy: Self::Conservative,
                memory_some_threshold: 15.0,
                memory_full_threshold: 5.0,
                k_draft_horizon: 2,
                max_tokens_clamp: 64,
                cooperative_yield_deadline_ms: 250,
            },
            Self::Balanced => TuningConfig {
                policy: Self::Balanced,
                memory_some_threshold: 25.0,
                memory_full_threshold: 10.0,
                k_draft_horizon: 4,
                max_tokens_clamp: 128,
                cooperative_yield_deadline_ms: 250,
            },
            Self::Aggressive => TuningConfig {
                policy: Self::Aggressive,
                memory_some_threshold: 40.0,
                memory_full_threshold: 20.0,
                k_draft_horizon: 8,
                max_tokens_clamp: 256,
                cooperative_yield_deadline_ms: 250,
            },
        }
    }
}

/// Dynamic tuning configuration and governor parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TuningConfig {
    pub policy: TuningPolicy,
    pub memory_some_threshold: f64,
    pub memory_full_threshold: f64,
    pub k_draft_horizon: usize,
    pub max_tokens_clamp: usize,
    pub cooperative_yield_deadline_ms: u64,
}

impl Default for TuningConfig {
    fn default() -> Self {
        TuningPolicy::Balanced.to_config()
    }
}

/// Instantaneous kernel pressure and scheduler latency metrics.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PressureMetrics {
    pub memory_some_avg10: f64,
    pub memory_full_avg10: f64,
    pub cpu_some_avg10: f64,
    pub io_some_avg10: f64,
    pub runqueue_latency_us: u64,
    pub ebpf_active: bool,
    pub source: String,
}

/// Full telemetry status report including active policy evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryStatusReport {
    pub metrics: PressureMetrics,
    pub config: TuningConfig,
    pub memory_spike_active: bool,
    pub cpu_contention_active: bool,
}

/// Outcome of tuning policy configuration invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningOutcome {
    pub applied: bool,
    pub path: String,
    pub config: TuningConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_parse_and_string() {
        assert_eq!(TuningPolicy::parse_str("conservative"), Some(TuningPolicy::Conservative));
        assert_eq!(TuningPolicy::parse_str("BALANCED"), Some(TuningPolicy::Balanced));
        assert_eq!(TuningPolicy::parse_str("Aggressive"), Some(TuningPolicy::Aggressive));
        assert_eq!(TuningPolicy::parse_str("unknown"), None);
        assert_eq!(TuningPolicy::Balanced.as_str(), "balanced");
    }

    #[test]
    fn test_policy_thresholds() {
        let b = TuningPolicy::Balanced.to_config();
        assert_eq!(b.memory_some_threshold, 25.0);
        assert_eq!(b.memory_full_threshold, 10.0);
        assert_eq!(b.k_draft_horizon, 4);

        let c = TuningPolicy::Conservative.to_config();
        assert_eq!(c.memory_some_threshold, 15.0);
        assert_eq!(c.k_draft_horizon, 2);

        let a = TuningPolicy::Aggressive.to_config();
        assert_eq!(a.memory_some_threshold, 40.0);
        assert_eq!(a.k_draft_horizon, 8);
    }
}
