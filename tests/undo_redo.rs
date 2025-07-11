//! Integration tests for undo/redo functionality
//!
//! This test suite verifies that the undo/redo system correctly captures
//! and restores state for all RISC-V instructions and edge cases.
//!
//! Run with: cargo test --test undo_redo --features repl

#[macro_use]
#[path = "common/mod.rs"]
mod common;

#[path = "undo_redo/integration_tests.rs"]
mod integration_tests;

#[path = "undo_redo/comprehensive_tests.rs"]
mod comprehensive_tests;

// Re-export common helpers and add undo/redo specific extensions
pub mod helpers {
    pub use crate::common::*;

    /// Extension trait for undo/redo specific operations
    pub trait UndoRedoExt {
        /// Undo N times
        fn undo_n(&mut self, n: usize) -> &mut Self;

        /// Redo N times  
        fn redo_n(&mut self, n: usize) -> &mut Self;

        /// Redo with expected content
        fn redo_expect(&mut self, expected: &str) -> &mut Self;

        /// Check redo should fail
        fn redo_should_fail(&mut self) -> &mut Self;
    }

    impl UndoRedoExt for TestContext<brubeck::interpreter::Interpreter> {
        fn undo_n(&mut self, n: usize) -> &mut Self {
            for _ in 0..n {
                self.undo();
            }
            self
        }

        fn redo_n(&mut self, n: usize) -> &mut Self {
            for _ in 0..n {
                self.redo();
            }
            self
        }

        fn redo_expect(&mut self, _expected: &str) -> &mut Self {
            let ctx = self.context("Redo");
            let result = self
                .inner
                .next_state()
                .unwrap_or_else(|e| panic!("{ctx}: {e:?}"));
            // For now, just check that redo succeeded
            // The library no longer returns instruction names
            assert!(
                result.contains("Redid"),
                "{ctx}: Expected redo message, got: {result}"
            );
            self
        }

        fn redo_should_fail(&mut self) -> &mut Self {
            let ctx = self.context("Redo (expecting failure)");
            if self.inner.next_state().is_ok() {
                panic!("{ctx}: Expected redo to fail but it succeeded");
            }
            self
        }
    }

    // Already imported via pub use crate::common::*;
}
