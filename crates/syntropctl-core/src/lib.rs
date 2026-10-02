//! Core library for the syntropctl unified administration CLI.
//!
//! Provides pure Rust Varlink client communication, daemon discovery,
//! and operations targeting the syntropd AI subsystem daemons.

pub mod admin;
pub mod companion;
pub mod daemon;
pub mod error;
pub mod ops;
pub mod sensory;
pub mod telemetry;
pub mod varlink;

pub use companion::{
    ask_screen, click_mouse, execute_instruction, listen_session, listen_session_with_callback,
    move_mouse_abs, send_key, type_text, ActuatorAction, CompanionAskResult,
    CompanionExecuteResult, CompanionListenEvent, CompanionListenOptions, UiAction,
};

pub use admin::{
    execute_remediation, execute_rollback, log_admin_audit, query_admin_audit, query_admin_status,
    reset_lockout, AdminAuditEntry, AdminStatusReport, CircuitBreakerInfo, LockoutResetOutcome,
    RecipeStep, RemediationOutcome, RemediationRecipe, RemediationRecipeKind, RollbackOutcome,
};
pub use daemon::{DaemonEndpoint, DaemonKind, DAEMONS};
pub use error::SyntropctlError;
pub use sensory::capture_audio;
pub use sensory::capture_frame;
pub use sensory::capture_screen;
pub use sensory::get_operator_presence;
pub use sensory::sensory_socket_path;
pub use sensory::AudioCaptureResult;
pub use sensory::ImageCaptureResult;
pub use sensory::OperatorPresenceResult;
pub use telemetry::{
    apply_tuning_policy, load_active_tuning_config, query_telemetry_status, PressureMetrics,
    TelemetryStatusReport, TuningConfig, TuningOutcome, TuningPolicy,
};
pub use varlink::VarlinkClient;

