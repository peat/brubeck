//! Common test infrastructure shared across unit and integration tests
//!
//! This module provides reusable test utilities, values, and patterns
//! that are useful for both CPU-level unit tests and interpreter-level
//! integration tests.

pub mod assertions;
pub mod context;
pub mod values;

// Re-export commonly used types for active tests
// Currently these are not used directly in tests but needed by extension traits
#[allow(unused_imports)]
pub use context::{interpreter_context, TestContext};
