//! Sub-millisecond shell prompt hook for pending incident indication.

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

/// Executes the shell prompt hook reading /run/syntrop/pending_count.
pub fn handle_prompt(raw: bool) -> anyhow::Result<()> {
    let base = std::env::var("RUNTIME_DIRECTORY")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/run/syntrop"));
    let count_file = base.join("pending_count");

    let count: usize = if count_file.exists() {
        let mut buf = [0u8; 16];
        if let Ok(mut f) = File::open(&count_file) {
            if let Ok(n) = f.read(&mut buf) {
                let s = std::str::from_utf8(&buf[..n]).unwrap_or("0").trim();
                s.parse().unwrap_or(0)
            } else {
                0
            }
        } else {
            0
        }
    } else {
        0
    };

    if raw {
        println!("{count}");
    } else if count > 0 {
        println!("⚡{count}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_prompt_reads_count() {
        let dir = std::env::temp_dir().join(format!("test_prompt_{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let count_path = dir.join("pending_count");
        fs::write(&count_path, "5\n").unwrap();

        std::env::set_var("RUNTIME_DIRECTORY", &dir);
        assert!(handle_prompt(true).is_ok());
        let _ = fs::remove_dir_all(&dir);
    }
}
