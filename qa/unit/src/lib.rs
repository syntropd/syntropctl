//! Root module for unit QA tests.

#[cfg(test)]
pub static ENV_MUTEX: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[cfg(test)]
pub mod admin_tests;
#[cfg(test)]
pub mod companion_tests;
#[cfg(test)]
pub mod endpoint_tests;
#[cfg(test)]
pub mod format_tests;
#[cfg(test)]
pub mod ops_tests;
#[cfg(test)]
pub mod varlink_tests;

