//! Command implementation modules for syntropctl subcommands.

pub mod execute;
pub mod inspect;

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
