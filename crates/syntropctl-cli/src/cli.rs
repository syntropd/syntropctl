//! CLI arguments and command definitions for syntropctl.

use clap::{Parser, Subcommand};
use clap_complete::Shell;

/// Unified administration and diagnostic CLI for the syntropd subsystem.
#[derive(Parser, Debug)]
#[command(
    name = "syntropctl",
    version,
    about = "Unified administration and diagnostic CLI for the syntropd subsystem",
    long_about = "Manage, inspect, and execute operations across sentry, inferenced, modeld, contextd, toold, and runtimed."
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
}
