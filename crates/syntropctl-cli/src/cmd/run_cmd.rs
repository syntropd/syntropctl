//! Run command handler.

use anyhow::Result;
use std::process::ExitCode;
use syntropctl_core::ops::execute_sandboxed_tool;

use crate::format::print_json;

pub async fn handle_run(
    tool: &str,
    args: &[String],
    profile: Option<&str>,
    json: bool,
) -> Result<ExitCode> {
    let result = execute_sandboxed_tool(tool, args, profile).await?;

    if json {
        print_json(&result);
    } else {
        if !result.stdout.is_empty() {
            print!("{}", result.stdout);
        }
        if !result.stderr.is_empty() {
            eprint!("{}", result.stderr);
        }
    }

    if result.exit_code == 0 {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from((result.exit_code as u8) & 0xFF))
    }
}
