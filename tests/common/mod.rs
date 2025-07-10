//! Common test infrastructure shared across unit and integration tests
//!
//! This module provides reusable test utilities, values, and patterns
//! that are useful for both CPU-level unit tests and interpreter-level
//! integration tests.

pub mod values;
pub mod assertions;
pub mod context;

// Re-export commonly used items
pub use values::*;
pub use assertions::*;
pub use context::*;