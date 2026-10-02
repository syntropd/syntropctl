//! Telemetry query client communicating with inferenced or runtimed over Varlink.

use super::policy::load_active_tuning_config;
use super::types::{PressureMetrics, TelemetryStatusReport};
use crate::error::SyntropctlError;
use crate::varlink::{VarlinkClient, DEFAULT_RPC_TIMEOUT};
use std::fs;
use std::path::{Path, PathBuf};

const TELEMETRY_SOCKETS: &[&str] = &[
    "/run/syntrop/io.syntrop.Telemetry1",
    "/run/syntrop/io.syntrop.Inference1",
    "/run/syntrop/io.syntrop.Runtime1",
];

/// Queries instantaneous kernel pressure metrics via Varlink or non-blocking fallback.
pub async fn query_telemetry_status() -> Result<TelemetryStatusReport, SyntropctlError> {
    let config = load_active_tuning_config();
    let metrics = match try_query_varlink_telemetry().await {
        Some(m) => m,
        None => read_local_kernel_telemetry(),
    };

    let memory_spike_active = metrics.memory_some_avg10 > config.memory_some_threshold
        || metrics.memory_full_avg10 > config.memory_full_threshold;
    let cpu_contention_active = metrics.runqueue_latency_us > 50_000;

    Ok(TelemetryStatusReport {
        metrics,
        config,
        memory_spike_active,
        cpu_contention_active,
    })
}

async fn try_query_varlink_telemetry() -> Option<PressureMetrics> {
    for sock_str in TELEMETRY_SOCKETS {
        let sock = PathBuf::from(sock_str);
        if !sock.exists() {
            continue;
        }

        if let Ok(val) = VarlinkClient::call(
            &sock,
            "io.syntrop.Telemetry1.GetKernelPressure",
            None,
            DEFAULT_RPC_TIMEOUT,
        )
        .await
        {
            if let Some(obj) = val.as_object() {
                let memory_some = obj.get("memory_some").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let memory_full = obj.get("memory_full").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let cpu_some = obj.get("cpu_some").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let io_some = obj.get("io_some").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let runqueue = obj.get("runqueue_latency_us").and_then(|v| v.as_u64()).unwrap_or(0);
                let ebpf = obj.get("ebpf_active").and_then(|v| v.as_bool()).unwrap_or(false);

                return Some(PressureMetrics {
                    memory_some_avg10: memory_some,
                    memory_full_avg10: memory_full,
                    cpu_some_avg10: cpu_some,
                    io_some_avg10: io_some,
                    runqueue_latency_us: runqueue,
                    ebpf_active: ebpf,
                    source: format!("varlink:{}", sock_str),
                });
            }
        }
    }
    None
}

fn read_local_kernel_telemetry() -> PressureMetrics {
    let (mem_some, mem_full) = parse_psi_file("/proc/pressure/memory");
    let (cpu_some, _) = parse_psi_file("/proc/pressure/cpu");
    let (io_some, _) = parse_psi_file("/proc/pressure/io");
    let runqueue = read_loadavg_runqueue_latency();

    PressureMetrics {
        memory_some_avg10: mem_some,
        memory_full_avg10: mem_full,
        cpu_some_avg10: cpu_some,
        io_some_avg10: io_some,
        runqueue_latency_us: runqueue,
        ebpf_active: false,
        source: "local:/proc/pressure".to_string(),
    }
}

fn parse_psi_file(path: &str) -> (f64, f64) {
    let text = match fs::read_to_string(Path::new(path)) {
        Ok(t) => t,
        Err(_) => return (0.0, 0.0),
    };
    let mut some_avg = 0.0;
    let mut full_avg = 0.0;
    for line in text.lines() {
        if line.starts_with("some ") {
            if let Some(pos) = line.find("avg10=") {
                let rest = &line[pos + 6..];
                let val_str = rest.split_whitespace().next().unwrap_or("0");
                some_avg = val_str.parse().unwrap_or(0.0);
            }
        } else if line.starts_with("full ") {
            if let Some(pos) = line.find("avg10=") {
                let rest = &line[pos + 6..];
                let val_str = rest.split_whitespace().next().unwrap_or("0");
                full_avg = val_str.parse().unwrap_or(0.0);
            }
        }
    }
    (some_avg, full_avg)
}

fn read_loadavg_runqueue_latency() -> u64 {
    if let Ok(load) = fs::read_to_string("/proc/loadavg") {
        if let Some(slash_pos) = load.find('/') {
            let left = load[..slash_pos].trim();
            if let Some(space_pos) = left.rfind(' ') {
                if let Ok(runnable) = left[space_pos + 1..].trim().parse::<u64>() {
                    return runnable.saturating_mul(120);
                }
            }
        }
    }
    120
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query_telemetry_status_fallback() {
        let rep = query_telemetry_status().await.expect("telemetry status query must succeed");
        assert!(!rep.metrics.source.is_empty());
        assert!(rep.config.memory_some_threshold > 0.0);
    }
}
