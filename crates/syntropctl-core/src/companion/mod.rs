//! Cognitive Desktop Companion operations coordinating sensory ingest, multimodal streaming, and actuation.

pub mod actuator;
pub mod ask;
pub mod execute;
pub mod listen;
pub mod router_stream;
pub mod safety;

pub use actuator::{
    actuator_socket_path, click_mouse, dispatch_action, move_mouse_abs, send_key, type_text,
    ActuatorAction,
};
pub use ask::{ask_screen, CompanionAskResult};
pub use execute::{
    execute_instruction, parse_direct_instruction, plan_ui_actions, CompanionExecuteResult,
    UiAction,
};
pub use listen::{
    listen_session, CompanionListenEvent, CompanionListenOptions,
};
pub use router_stream::{
    parse_http_completion_response, query_router_multimodal, router_socket_path,
    MULTIMODAL_ROUTER_TIMEOUT,
};
pub use safety::check_physical_user_input;
