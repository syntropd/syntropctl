//! Physical user input safety monitoring and macro preemption.

pub mod input;
pub use input::check_physical_user_input;

use std::process::Command;

/// Check whether an input device name belongs to a virtual actuator.
pub fn is_virtual_actuator_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains("syntrop-virtual-actuator")
        || lower.contains("syntrop")
        || lower.contains("uinput")
}

/// Inspect sysfs to check if an evdev node belongs to a virtual actuator.
pub fn is_virtual_input_node(node_name: &str) -> bool {
    let sys_name = format!("/sys/class/input/{node_name}/device/name");
    if std::fs::read_to_string(&sys_name).map(|c| is_virtual_actuator_name(&c)).unwrap_or(false) {
        return true;
    }
    let sys_path = format!("/sys/class/input/{node_name}");
    std::fs::canonicalize(&sys_path)
        .map(|p| p.to_string_lossy().contains("/devices/virtual/"))
        .unwrap_or(false)
}

/// Identifies if a process name, command, or window title represents an elevated auth prompter.
pub fn is_elevated_auth_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    let trimmed = lower.trim();
    if trimmed == "polkitd" || trimmed.starts_with("polkitd ") {
        return false;
    }
    trimmed == "sudo"
        || trimmed == "pkexec"
        || trimmed == "doas"
        || trimmed == "su"
        || trimmed.contains("polkit-agent")
        || trimmed.contains("polkit-gnome")
        || trimmed.contains("polkit-kde")
        || trimmed.contains("polkit-mate")
        || trimmed.contains("lxpolkit")
        || trimmed.contains("pinentry")
        || trimmed.contains("gcr-prompter")
        || trimmed.contains("authentication-agent")
        || trimmed.contains("authentication required")
        || trimmed.contains("authenticate")
}

/// Inspects the active desktop window for elevated authentication dialog focus (X11 / Wayland).
pub fn inspect_active_window_auth() -> bool {
    if std::env::var_os("DISPLAY").is_none() && std::env::var_os("WAYLAND_DISPLAY").is_none() {
        return false;
    }
    if let Ok(output) = Command::new("xprop").args(["-root", "_NET_ACTIVE_WINDOW"]).output() {
        if output.status.success() {
            let out_str = String::from_utf8_lossy(&output.stdout);
            if let Some(id_part) = out_str.split('#').nth(1) {
                let win_id = id_part.split_whitespace().next().unwrap_or("");
                if !win_id.is_empty() && win_id != "0x0" {
                    if let Ok(w_out) = Command::new("xprop")
                        .args(["-id", win_id, "WM_CLASS", "_NET_WM_NAME", "WM_NAME"])
                        .output()
                    {
                        if w_out.status.success() && is_elevated_auth_name(&String::from_utf8_lossy(&w_out.stdout)) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
        if let Ok(w) = Command::new("hyprctl").arg("activewindow").output() {
            if w.status.success() && is_elevated_auth_name(&String::from_utf8_lossy(&w.stdout)) {
                return true;
            }
        }
    }
    false
}

/// Inspects `/proc` to detect running elevated authentication prompters or foreground sudo processes.
pub fn inspect_proc_elevated_auth() -> bool {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return false;
    };
    for entry in entries.flatten() {
        let name_str = entry.file_name().to_string_lossy().to_string();
        if !name_str.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let pid_path = entry.path();
        if let Ok(comm) = std::fs::read_to_string(pid_path.join("comm")) {
            let comm_trim = comm.trim();
            if comm_trim == "polkitd" {
                continue;
            }
            if is_elevated_auth_name(comm_trim) {
                if comm_trim.starts_with("polkit-agent")
                    || comm_trim.contains("pinentry")
                    || comm_trim.contains("gcr-prompter")
                {
                    return true;
                }
                if comm_trim == "sudo" || comm_trim == "doas" || comm_trim == "su" || comm_trim == "pkexec" {
                    if let Ok(stat) = std::fs::read_to_string(pid_path.join("stat")) {
                        if let Some(after_paren) = stat.rfind(')') {
                            let fields: Vec<&str> = stat[after_paren + 1..].split_whitespace().collect();
                            if fields.len() >= 6 {
                                let pgrp = fields[2];
                                let tty_nr = fields[4].parse::<i32>().unwrap_or(0);
                                let tpgid = fields[5];
                                if tty_nr > 0 && pgrp == tpgid && tpgid != "-1" {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

/// Checks whether an elevated authentication prompt (Polkit, pkexec, or sudo) has focus.
pub fn check_elevated_auth_focus() -> bool {
    if let Ok(sim) = std::env::var("SYNTROP_SIMULATE_ELEVATED_AUTH") {
        if sim == "1" || sim.eq_ignore_ascii_case("true") {
            return true;
        }
    }
    if std::env::var("SYNTROP_DISABLE_ELEVATED_AUTH_CHECK").is_ok() {
        return false;
    }
    if inspect_active_window_auth() {
        return true;
    }
    inspect_proc_elevated_auth()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_virtual_actuator_name() {
        assert!(is_virtual_actuator_name("syntrop-virtual-actuator"));
        assert!(is_virtual_actuator_name("uinput-syntrop"));
        assert!(!is_virtual_actuator_name("AT Translated Set 2 keyboard"));
    }

    #[test]
    fn test_is_elevated_auth_name() {
        for n in &["polkit-gnome-agent", "pkexec", "sudo", "pinentry", "Authenticate", "Authentication Required"] {
            assert!(is_elevated_auth_name(n), "should match: {n}");
        }
        for n in &["firefox", "alacritty", "polkitd", "bash"] {
            assert!(!is_elevated_auth_name(n), "should not match: {n}");
        }
    }

    #[test]
    fn test_elevated_auth_simulation_flag() {
        assert!(!inspect_proc_elevated_auth());
        std::env::remove_var("SYNTROP_SIMULATE_ELEVATED_AUTH");
        std::env::set_var("SYNTROP_DISABLE_ELEVATED_AUTH_CHECK", "1");
        assert!(!check_elevated_auth_focus());
        std::env::remove_var("SYNTROP_DISABLE_ELEVATED_AUTH_CHECK");

        std::env::set_var("SYNTROP_SIMULATE_ELEVATED_AUTH", "1");
        assert!(check_elevated_auth_focus());
        std::env::remove_var("SYNTROP_SIMULATE_ELEVATED_AUTH");
    }
}
