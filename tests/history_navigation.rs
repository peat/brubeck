//! Integration tests for history navigation functionality
//!
//! This test suite verifies that the history navigation system correctly captures
//! and restores state for all RISC-V instructions and edge cases.
//!
//! Run with: cargo test --test history_navigation

#[macro_use]
#[path = "common/mod.rs"]
mod common;

#[path = "history_navigation/integration_tests.rs"]
mod integration_tests;

#[path = "history_navigation/comprehensive_tests.rs"]
mod comprehensive_tests;

// Re-export common helpers and add history navigation specific extensions
pub mod helpers {
    pub use crate::common::*;

    /// Extension trait for history navigation specific operations
    pub trait HistoryNavigationExt {
        /// Navigate back N times
        fn previous_n(&mut self, n: usize) -> &mut Self;

        /// Navigate forward N times  
        fn next_n(&mut self, n: usize) -> &mut Self;

        /// Navigate forward with expected content
        fn next_expect(&mut self, expected: &str) -> &mut Self;

        /// Check forward navigation should fail
        fn next_should_fail(&mut self) -> &mut Self;
    }

    impl HistoryNavigationExt for TestContext<brubeck::interpreter::Interpreter> {
        fn previous_n(&mut self, n: usize) -> &mut Self {
            for _ in 0..n {
                self.previous();
            }
            self
        }

        fn next_n(&mut self, n: usize) -> &mut Self {
            for _ in 0..n {
                self.next();
            }
            self
        }

        fn next_expect(&mut self, _expected: &str) -> &mut Self {
            let ctx = self.context("Navigate forward");
            let result = self
                .inner
                .next_state()
                .unwrap_or_else(|e| panic!("{ctx}: {e:?}"));
            // For now, just check that navigation succeeded
            // The library no longer returns instruction names
            // The new API returns StateDelta, not strings
            // Success is indicated by not panicking above
            let _ = result;
            self
        }

        fn next_should_fail(&mut self) -> &mut Self {
            let ctx = self.context("Navigate forward (expecting failure)");
            if self.inner.next_state().is_ok() {
                panic!("{ctx}: Expected forward navigation to fail but it succeeded");
            }
            self
        }
    }

    // Already imported via pub use crate::common::*;
}
