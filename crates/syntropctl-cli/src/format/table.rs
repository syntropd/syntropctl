//! Formatted tabular and human-readable plain text output.

use syntropctl_core::ops::{
    DaemonStatus, DeviceReport, DriftEvent, IncidentReport, ModelEntry,
};

/// Print daemon fleet status in a clean columnar table.
pub fn print_status_table(statuses: &[DaemonStatus]) {
    println!(
        "{:<14} {:<20} {:<12} {:<10} {:<8} {:<16}",
        "DAEMON", "UNIT", "SOCKET", "STATUS", "LATENCY", "PRODUCT/VERSION"
    );
    println!("{}", "-".repeat(84));

    for s in statuses {
        let sock_label = if s.socket_exists { "active" } else { "missing" };
        let status_label = if s.responsive {
            "responsive"
        } else if s.socket_exists {
            "unreachable"
        } else {
            "inactive"
        };
        let lat_label = s
            .latency_ms
            .map(|l| format!("{}ms", l))
            .unwrap_or_else(|| "-".to_string());
        let prod_label = match (&s.product, &s.version) {
            (Some(p), Some(v)) => format!("{} v{}", p, v),
            (Some(p), None) => p.clone(),
            _ => s
                .error_message
                .as_deref()
                .unwrap_or("-")
                .to_string(),
        };

        println!(
            "{:<14} {:<20} {:<12} {:<10} {:<8} {:<16}",
            s.name, s.unit_name, sock_label, status_label, lat_label, prod_label
        );
    }
}

/// Print unified model catalog table.
pub fn print_models_table(models: &[ModelEntry]) {
    if models.is_empty() {
        println!("No models registered or currently loaded.");
        return;
    }

    println!(
        "{:<24} {:<10} {:<12} {:<10} {:<12} {:<10}",
        "MODEL", "STATUS", "SIZE", "PARAMS", "BACKEND", "CONTEXT"
    );
    println!("{}", "-".repeat(80));

    for m in models {
        let size_str = format_bytes(m.size_bytes);
        let param_str = m
            .parameter_count
            .map(|p| format!("{:.1}B", p as f64 / 1_000_000_000.0))
            .unwrap_or_else(|| "-".to_string());
        let backend_str = m.backend.as_deref().unwrap_or("-");
        let ctx_str = m
            .context_window
            .map(|c| c.to_string())
            .unwrap_or_else(|| "-".to_string());

        println!(
            "{:<24} {:<10} {:<12} {:<10} {:<12} {:<10}",
            m.name, m.status, size_str, param_str, backend_str, ctx_str
        );
    }
}

/// Print hardware devices and accelerators table.
pub fn print_devices_table(devices: &[DeviceReport]) {
    if devices.is_empty() {
        println!("No hardware compute accelerators detected.");
        return;
    }

    println!(
        "{:<14} {:<8} {:<16} {:<14} {:<10} {:<8}",
        "DEVICE", "TYPE", "VENDOR/MODEL", "VRAM (USED/TOT)", "PSI-10", "STATUS"
    );
    println!("{}", "-".repeat(76));

    for d in devices {
        let name = format!("{} {}", d.vendor, d.model);
        let vram = format!(
            "{}/{}",
            format_bytes(d.memory_used_bytes),
            format_bytes(d.memory_total_bytes)
        );
        let psi = format!("{:.1}%", d.psi_pressure);

        println!(
            "{:<14} {:<8} {:<16} {:<14} {:<10} {:<8}",
            d.id, d.device_type, name, vram, psi, d.status
        );
    }
}

/// Print configuration drift timeline.
pub fn print_drift_table(events: &[DriftEvent]) {
    if events.is_empty() {
        println!("No recent configuration drift or modified units recorded.");
        return;
    }

    println!(
        "{:<28} {:<12} {:<12} {:<24}",
        "PATH/TARGET", "CHANGE", "TIMESTAMP", "DETAILS"
    );
    println!("{}", "-".repeat(78));

    for e in events {
        println!(
            "{:<28} {:<12} {:<12} {:<24}",
            e.path, e.change_type, e.timestamp, e.details
        );
    }
}

/// Print comprehensive incident explanation report.
pub fn print_incident_report(report: &IncidentReport) {
    println!("=== Syntropd Incident Diagnosis: {} ===", report.unit);
    println!("Unit Status:      {}", report.status);
    println!("Root Cause:       {}", report.root_cause);
    println!("Confidence:       {:.1}%", report.confidence * 100.0);
    println!("Recommended Fix:  {}", report.recommended_action);

    if let Some(cmd) = &report.remediation_command {
        println!("Remediation Cmd:  {}", cmd);
    }

    if !report.recent_drift.is_empty() {
        println!("\nCorrelated Configuration Drift:");
        for d in &report.recent_drift {
            println!("  * {}", d);
        }
    }

    if !report.journal_slice.is_empty() {
        println!("\nAssociated Journal Logs:");
        for l in &report.journal_slice {
            println!("  {}", l);
        }
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GiB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MiB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KiB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
