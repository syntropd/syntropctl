//! Physical user input safety monitoring and device polling.

use rustix::event::{poll, PollFd, PollFlags};
use rustix::fd::AsFd;
use rustix::fs::{open, Mode, OFlags};
use std::path::Path;
use std::time::{Duration, SystemTime};

/// Check whether physical user input has occurred on evdev nodes.
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
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();
            if (name_str.starts_with("event") || name_str.starts_with("mouse"))
                && !super::is_virtual_input_node(&name_str)
                && (has_pending_input_event(&entry.path())
                    || is_recent_input(&entry.path(), now, threshold))
            {
                return true;
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
}
