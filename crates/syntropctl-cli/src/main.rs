//! Main executable entry point for syntropctl.

use clap::Parser;
use std::process::ExitCode;
use syntropctl_cli::cli::{Cli, Commands, SensoryCommands};
use syntropctl_cli::cmd::audio::AudioCommands;
use syntropctl_cli::cmd::video::VideoCommands;
use syntropctl_cli::cmd::visual::VisualCommands;
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
        Commands::Drift { unit } => handle_drift(unit.as_deref(), json)
            .await
            .map(|_| ExitCode::SUCCESS),
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
            effort,
        } => handle_generate(
            &prompt,
            &model,
            max_tokens,
            temperature,
            effort.as_deref(),
            json,
        )
        .await
        .map(|_| ExitCode::SUCCESS),
        Commands::Embed { text, model } => handle_embed(&text, &model, json)
            .await
            .map(|_| ExitCode::SUCCESS),
        Commands::Info { daemon } => handle_info(daemon, json).await.map(|_| ExitCode::SUCCESS),
        Commands::Completions { shell } => {
            handle_completions(shell);
            Ok(ExitCode::SUCCESS)
        }
        Commands::Decide { approve, reject } => {
            handle_decide(approve.as_deref(), reject.as_deref(), json)
                .await
                .map(|_| ExitCode::SUCCESS)
        }
        Commands::Prompt { raw } => handle_prompt(raw).map(|_| ExitCode::SUCCESS),
        Commands::Audit { unit, limit } => handle_audit(unit.as_deref(), limit, json)
            .await
            .map(|_| ExitCode::SUCCESS),
        Commands::Sensory { command } => match command {
            SensoryCommands::Audio {
                duration_ms,
                sample_rate,
                out,
            } => handle_audio(duration_ms, sample_rate, out.as_deref(), json)
                .await
                .map(|_| ExitCode::SUCCESS),
            SensoryCommands::Frame {
                device,
                width,
                height,
                out,
            } => handle_frame(device.as_deref(), width, height, out.as_deref(), json)
                .await
                .map(|_| ExitCode::SUCCESS),
            SensoryCommands::Screen { display, out } => {
                handle_screen(display.as_deref(), out.as_deref(), json)
                    .await
                    .map(|_| ExitCode::SUCCESS)
            }
            SensoryCommands::Presence => handle_presence(json).await.map(|_| ExitCode::SUCCESS),
        },
        Commands::Admin { command } => handle_admin(command, json).await,
        Commands::Companion { command } => handle_companion(command, json).await,
        Commands::Telemetry { command } => handle_telemetry(command, json).await,
        Commands::Audio { command } => match command {
            AudioCommands::Generate {
                prompt,
                duration,
                bpm,
            } => handle_audio_generate(&prompt, duration, bpm, json)
                .await
                .map(|_| ExitCode::SUCCESS),
        },
        Commands::Visual { command } => match command {
            VisualCommands::Generate {
                prompt,
                model,
                lora,
                size,
            } => handle_visual_generate(
                &prompt,
                model.as_deref(),
                lora.as_deref(),
                size.as_deref(),
                json,
            )
            .await
            .map(|_| ExitCode::SUCCESS),
        },
        Commands::Video { command } => match command {
            VideoCommands::Generate { prompt, frames, fps } => {
                handle_video_generate(&prompt, frames, fps, json)
                    .await
                    .map(|_| ExitCode::SUCCESS)
            }
        },
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
