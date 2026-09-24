//! Main executable entry point for syntropctl.

use clap::Parser;
use std::process::ExitCode;
use syntropctl_cli::cli::{Cli, Commands};
use syntropctl_cli::cmd::*;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    if cli.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    }

    let json = cli.json;

    let res: Result<ExitCode, anyhow::Error> = match cli.command {
        Commands::Status { daemon } => handle_status(daemon, json).await.map(|_| ExitCode::SUCCESS),
        Commands::Explain { unit } => handle_explain(&unit, json).await.map(|_| ExitCode::SUCCESS),
        Commands::Models => handle_models(json).await.map(|_| ExitCode::SUCCESS),
        Commands::Devices => handle_devices(json).await.map(|_| ExitCode::SUCCESS),
        Commands::Drift { unit } => handle_drift(unit.as_deref(), json).await.map(|_| ExitCode::SUCCESS),
        Commands::Run {
            tool,
            profile,
            args,
        } => handle_run(&tool, &args, profile.as_deref(), json).await,
        Commands::Generate {
            prompt,
            model,
            max_tokens,
            temperature,
        } => handle_generate(&prompt, &model, max_tokens, temperature, json)
            .await
            .map(|_| ExitCode::SUCCESS),
        Commands::Embed { text, model } => handle_embed(&text, &model, json).await.map(|_| ExitCode::SUCCESS),
        Commands::Info { daemon } => handle_info(daemon, json).await.map(|_| ExitCode::SUCCESS),
        Commands::Completions { shell } => {
            handle_completions(shell);
            Ok(ExitCode::SUCCESS)
        }
    };

    match res {
        Ok(code) => code,
        Err(e) => {
            if json {
                eprintln!(
                    "{}",
                    serde_json::json!({
                        "error": e.to_string(),
                    })
                );
            } else {
                eprintln!("syntropctl error: {}", e);
            }
            ExitCode::FAILURE
        }
    }
}
