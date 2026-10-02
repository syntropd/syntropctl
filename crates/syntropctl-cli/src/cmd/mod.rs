//! Command implementation modules for syntropctl subcommands.

pub mod admin;
pub mod companion;
pub mod execute;
pub mod inspect;
pub mod sensory;
pub mod telemetry;
pub mod triage;

pub use admin::handle_admin;
pub use companion::handle_companion;
pub use execute::handle_completions;
pub use execute::handle_embed;
pub use execute::handle_generate;
pub use execute::handle_run;
pub use inspect::handle_devices;
pub use inspect::handle_drift;
pub use inspect::handle_explain;
pub use inspect::handle_info;
pub use inspect::handle_models;
pub use inspect::handle_status;
pub use sensory::handle_audio;
pub use sensory::handle_frame;
pub use sensory::handle_presence;
pub use sensory::handle_screen;
pub use telemetry::handle_telemetry;
pub use triage::handle_audit;
pub use triage::handle_decide;
pub use triage::handle_prompt;

