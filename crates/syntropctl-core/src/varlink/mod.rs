//! Varlink protocol framing and client interface module.

pub mod client;

pub use client::{
    ServiceInfo, VarlinkCall, VarlinkClient, VarlinkReply, DEFAULT_RPC_TIMEOUT,
    GENERATE_RPC_TIMEOUT,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varlink_constants() {
        assert_eq!(DEFAULT_RPC_TIMEOUT.as_secs(), 10);
        assert_eq!(GENERATE_RPC_TIMEOUT.as_secs(), 600);
    }
}
