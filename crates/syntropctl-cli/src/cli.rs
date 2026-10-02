//! CLI arguments and command definitions for syntropctl.

use crate::cmd::admin::AdminCommands;
use crate::cmd::companion::CompanionCommands;
use crate::cmd::telemetry::TelemetryCommands;
use clap::{Parser, Subcommand};
use clap_complete::Shell;
use std::path::PathBuf;

/// Unified administration and diagnostic CLI for the syntropd subsystem.
#[derive(Parser, Debug)]
#[command(
    name = "syntropctl",
    version,
    about = "Unified administration and diagnostic CLI for the syntropd subsystem",
    long_about = "Manage, inspect, and execute operations across sentry, inferenced, modeld, contextd, toold, runtimed, and routerd."
)]
pub struct Cli {
    /// Format output as JSON instead of tabular plain text.
    #[arg(long = "json", global = true)]
    pub json: bool,

    /// Increase logging verbosity.
    #[arg(short = 'v', long = "verbose", global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

/// Available subcommands for syntropctl.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Display fleet health matrix across all subsystem daemons.
    Status {
        /// Optional specific daemon name to inspect.
        daemon: Option<String>,
    },

    /// Diagnose and explain failure or state of a systemd unit.
    Explain {
        /// Name of the target systemd unit (e.g., nginx.service, sentry.service).
        unit: String,
    },

    /// Query unified catalog of cached and loaded neural models.
    Models,

    /// Inspect hardware accelerators, memory allocations, and PSI pressure.
    Devices,

    /// Query system chronology and configuration drift from contextd.
    Drift {
        /// Optional unit or service name filter.
        unit: Option<String>,
    },

    /// Execute a sandboxed command or tool via toold.
    Run {
        /// Tool or command binary name.
        tool: String,

        /// Profile name for sandbox policies.
        #[arg(short = 'p', long = "profile")]
        profile: Option<String>,

        /// Arguments passed to the target tool.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Generate text completions from a prompt via runtimed.
    Generate {
        /// Input prompt text.
        prompt: String,

        /// Model identifier to invoke.
        #[arg(short = 'm', long = "model", default_value = "qwen2.5-coder-7b")]
        model: String,

        /// Maximum token budget for generation.
        #[arg(short = 'n', long = "max-tokens", default_value = "256")]
        max_tokens: usize,

        /// Decoding temperature (0.0 for deterministic output).
        #[arg(short = 't', long = "temperature", default_value = "0.0")]
        temperature: f32,

        /// Reasoning effort tier (none, low, medium, high, max; defaults to 0 tokens on CPU / tight memory, 1,024 on GPU with healthy VRAM).
        #[arg(short = 'e', long = "effort", alias = "reasoning-effort")]
        effort: Option<String>,
    },

    /// Compute semantic vector embedding for input text via runtimed.
    Embed {
        /// Input text string.
        text: String,

        /// Model identifier for computing embeddings.
        #[arg(short = 'm', long = "model", default_value = "qwen2.5-coder-7b")]
        model: String,
    },

    /// Introspect Varlink interface specifications and vendor metadata.
    Info {
        /// Target daemon name (e.g., runtimed, toold).
        daemon: Option<String>,
    },

    /// Generate shell auto-completion scripts.
    Completions {
        /// Target shell syntax.
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Review and approve or reject pending System One triage decisions.
    Decide {
        /// Approve specific incident ID without interactive prompt.
        #[arg(long = "approve")]
        approve: Option<String>,

        /// Reject specific incident ID without interactive prompt.
        #[arg(long = "reject")]
        reject: Option<String>,
    },

    /// Sub-millisecond shell prompt hook returning pending triage count.
    Prompt {
        /// Print raw count integer without prompt decoration.
        #[arg(long = "raw")]
        raw: bool,
    },

    /// Query structured systemd-journald audit trail.
    Audit {
        /// Filter audit records by service unit name.
        #[arg(short = 'u', long = "unit")]
        unit: Option<String>,

        /// Maximum number of audit records to return.
        #[arg(short = 'n', long = "limit", default_value = "50")]
        limit: usize,
    },

    /// Environmental sensing and operator presence queries.
    Sensory {
        #[command(subcommand)]
        command: SensoryCommands,
    },

    /// Autonomous OS self-healing, remediation recipes, and circuit breaker management.
    Admin {
        #[command(subcommand)]
        command: AdminCommands,
    },

    /// Linux Cognitive Desktop Companion multimodal assistance and desktop automation.
    Companion {
        #[command(subcommand)]
        command: CompanionCommands,
    },

    /// Non-blocking kernel telemetry, PSI pressure, and closed-loop tuning governor.
    Telemetry {
        #[command(subcommand)]
        command: TelemetryCommands,
    },
}

/// Subcommands for environmental sensing operations.
#[derive(Subcommand, Debug)]
pub enum SensoryCommands {
    /// Record ambient audio PCM from default input source.
    Audio {
        /// Recording duration in milliseconds.
        #[arg(long = "duration-ms")]
        duration_ms: Option<u32>,

        /// Sampling frequency in Hz (e.g. 16000).
        #[arg(long = "sample-rate")]
        sample_rate: Option<u32>,

        /// Optional output file path to write captured PCM bytes.
        #[arg(long = "out")]
        out: Option<PathBuf>,
    },

    /// Capture video frame via V4L2 device.
    Frame {
        /// Video device path (e.g., /dev/video0).
        #[arg(long = "device")]
        device: Option<String>,

        /// Frame width in pixels.
        #[arg(long = "width")]
        width: Option<u32>,

        /// Frame height in pixels.
        #[arg(long = "height")]
        height: Option<u32>,

        /// Optional output file path to write captured frame image.
        #[arg(long = "out")]
        out: Option<PathBuf>,
    },

    /// Capture desktop screen image buffer.
    Screen {
        /// Target display identifier (e.g. :0, wayland-0).
        #[arg(long = "display")]
        display: Option<String>,

        /// Optional output file path to write captured screen image.
        #[arg(long = "out")]
        out: Option<PathBuf>,
    },

    /// Query composite operator presence estimation.
    Presence,
}
