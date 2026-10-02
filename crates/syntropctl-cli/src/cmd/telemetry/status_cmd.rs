//! Handler for `syntropctl telemetry status` displaying kernel PSI and eBPF telemetry.

use std::process::ExitCode;
use syntropctl_core::telemetry::query_telemetry_status;

/// Renders kernel telemetry and pressure status in formatted text or JSON.
pub async fn handle_telemetry_status(json: bool) -> anyhow::Result<ExitCode> {
    let report = query_telemetry_status().await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(ExitCode::SUCCESS);
    }

    println!("=== Syntrop Kernel Telemetry & PSI Status ===");
    println!("  Telemetry Source:    {}", report.metrics.source);
    let ebpf_label = if report.metrics.ebpf_active {
        "Active (Tracepoint scheduler probe)"
    } else {
        "Fallback (Unprivileged /proc/loadavg probe)"
    };
    println!("  eBPF Subsystem:      {}", ebpf_label);
    let rq_status = if report.metrics.runqueue_latency_us > 50_000 {
        "CONTENTION (> 50,000 us)"
    } else {
        "Nominal"
    };
    println!(
        "  Runqueue Latency:    {} us ({})",
        report.metrics.runqueue_latency_us, rq_status
    );
    println!(
        "  CPU Pressure (some): {:.2}%",
        report.metrics.cpu_some_avg10
    );
    println!(
        "  Memory PSI (some):   {:.2}% (Threshold: {:.2}%)",
        report.metrics.memory_some_avg10, report.config.memory_some_threshold
    );
    println!(
        "  Memory PSI (full):   {:.2}% (Threshold: {:.2}%)",
        report.metrics.memory_full_avg10, report.config.memory_full_threshold
    );
    println!(
        "  I/O Pressure (some): {:.2}%",
        report.metrics.io_some_avg10
    );
    println!(
        "  Active Policy:       {} (K={} draft horizon, {} token clamp)",
        report.config.policy.as_str(),
        report.config.k_draft_horizon,
        report.config.max_tokens_clamp
    );

    let spike_status = match (report.memory_spike_active, report.cpu_contention_active) {
        (true, true) => "CRITICAL: Memory pressure spike & CPU contention active",
        (true, false) => "WARNING: Memory pressure spike (Draft K->1, tokens clamped)",
        (false, true) => "WARNING: CPU contention (250ms cooperative yield active)",
        (false, false) => "Nominal (No pressure spikes)",
    };
    println!("  Pressure State:      {}", spike_status);

    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handle_telemetry_status_json() {
        let code = handle_telemetry_status(true).await.expect("json status succeeds");
        assert_eq!(code, ExitCode::SUCCESS);
    }
}
