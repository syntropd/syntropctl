//! Physical user input safety monitoring and macro preemption.

use rustix::event::{poll, PollFd, PollFlags};
use rustix::fd::AsFd;
use rustix::fs::{open, Mode, OFlags};
use std::path::Path;
use std::time::{Duration, SystemTime};

/// Check whether an input device name belongs to a virtual actuator.
pub fn is_virtual_actuator_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains("syntrop-virtual-actuator")
        || lower.contains("syntrop")
        || lower.contains("uinput")
}

/// Inspect sysfs to check if an evdev node belongs to a virtual actuator.
pub fn is_virtual_input_node(node_name: &str) -> bool {
    let sys_name = format!("/sys/class/input/{}/device/name", node_name);
    if let Ok(content) = std::fs::read_to_string(&sys_name) {
        if is_virtual_actuator_name(&content) {
            return true;
        }
    }
    let sys_path = format!("/sys/class/input/{}", node_name);
    if let Ok(canon) = std::fs::canonicalize(&sys_path) {
        if canon.to_string_lossy().contains("/devices/virtual/") {
            return true;
        }
    }
    false
}

/// Check whether physical user input has occurred.
///
/// Returns true if physical user activity is detected, signaling
/// that any automated macro playback must abort immediately to avoid
/// fighting the human operator for desktop focus or control.
pub fn check_physical_user_input() -> bool {
    if let Ok(sim) = std::env::var("SYNTROP_SIMULATE_PHYSICAL_INPUT") {
        if sim == "1" || sim.eq_ignore_ascii_case("true") {
            return true;
        }
    }
    if std::env::var("SYNTROP_DISABLE_PHYSICAL_INPUT_CHECK").is_ok() {
        return false;
    }

    if let Ok(entries) = std::fs::read_dir("/dev/input") {
        let threshold = Duration::from_millis(500);
        let now = SystemTime::now();

        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();
            if (name_str.starts_with("event") || name_str.starts_with("mouse"))
                && !is_virtual_input_node(&name_str)
            {
                if has_pending_input_event(&path) {
                    return true;
                }
                if is_recent_input(&path, now, threshold) {
                    return true;
                }
            }
        }
    }

    false
}

/// Poll character device with non-blocking I/O for pending input events.
fn has_pending_input_event(path: &Path) -> bool {
    if let Ok(fd) = open(path, OFlags::NONBLOCK | OFlags::RDONLY, Mode::empty()) {
        let mut pfd = [PollFd::from_borrowed_fd(fd.as_fd(), PollFlags::IN)];
        if let Ok(res) = poll(&mut pfd, 0) {
            if res > 0 && pfd[0].revents().contains(PollFlags::IN) {
                return true;
            }
        }
    }
    false
}

/// Checks whether an input device node was modified within threshold window.
fn is_recent_input(path: &Path, now: SystemTime, threshold: Duration) -> bool {
    if let Ok(meta) = std::fs::metadata(path) {
        if let Ok(mtime) = meta.modified() {
            if let Ok(diff) = now.duration_since(mtime) {
                return diff <= threshold;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulate_physical_input_flag() {
        std::env::remove_var("SYNTROP_SIMULATE_PHYSICAL_INPUT");
        assert!(!check_physical_user_input());

        std::env::set_var("SYNTROP_SIMULATE_PHYSICAL_INPUT", "1");
        assert!(check_physical_user_input());

        std::env::remove_var("SYNTROP_SIMULATE_PHYSICAL_INPUT");
    }

    #[test]
    fn test_disable_physical_input_flag() {
        std::env::set_var("SYNTROP_DISABLE_PHYSICAL_INPUT_CHECK", "1");
        assert!(!check_physical_user_input());
        std::env::remove_var("SYNTROP_DISABLE_PHYSICAL_INPUT_CHECK");
    }

    #[test]
    fn test_is_virtual_actuator_name() {
        assert!(is_virtual_actuator_name("syntrop-virtual-actuator"));
        assert!(is_virtual_actuator_name("uinput-syntrop"));
        assert!(!is_virtual_actuator_name("AT Translated Set 2 keyboard"));
    }
}
