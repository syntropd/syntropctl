//! Physical user input safety monitoring and macro preemption.

use std::path::Path;
use std::time::{Duration, SystemTime};

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

    // Inspect recent device modification timestamps in /dev/input
    if let Ok(entries) = std::fs::read_dir("/dev/input") {
        let threshold = Duration::from_millis(500);
        let now = SystemTime::now();

        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();
            if (name_str.starts_with("event") || name_str.starts_with("mouse"))
                && is_recent_input(&path, now, threshold)
            {
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
}
