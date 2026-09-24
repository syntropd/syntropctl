//! Varlink protocol framing and client interface module.

pub mod client;

pub use client::{ServiceInfo, VarlinkCall, VarlinkClient, VarlinkReply, DEFAULT_RPC_TIMEOUT};
