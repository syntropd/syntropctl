//! Cognitive Desktop Companion operations coordinating sensory ingest, multimodal streaming, and actuation.

pub mod actuator;
pub mod ask;
pub mod execute;
pub mod grounding;
pub mod listen;
pub mod router_stream;
pub mod safety;
pub mod talk;

pub use actuator::{
    actuator_socket_path, click_mouse, dispatch_action, move_mouse_abs, send_key, type_text,
    ActuatorAction,
};
pub use ask::{ask_screen, ground_screen_ocr, CompanionAskResult};
pub use execute::{
    discover_ui_element_fast, execute_instruction, execute_instruction_with_grounding,
    parse_direct_instruction, plan_ui_actions, plan_ui_actions_with_grounding,
    CompanionExecuteResult, UiAction,
};
pub use grounding::{ground_ui_element, parse_grounding_response, GroundedElement};
pub use listen::{
    listen_session, listen_session_with_callback, CompanionListenEvent, CompanionListenOptions,
};
pub use router_stream::{
    parse_http_completion_response, query_router_multimodal, router_socket_path,
    MULTIMODAL_ROUTER_TIMEOUT,
};
pub use safety::{check_elevated_auth_focus, check_physical_user_input};
pub use talk::{run_talk_session, TalkOptions, TalkTranscriptEvent};
