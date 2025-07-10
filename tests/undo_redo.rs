//! Integration tests for undo/redo functionality
//!
//! This test suite verifies that the undo/redo system correctly captures
//! and restores state for all RISC-V instructions and edge cases.
//!
//! Run with: cargo test --test undo_redo --features repl

#[macro_use]
#[path = "common/mod.rs"]
mod common;

#[cfg(feature = "repl")]
#[path = "undo_redo/integration_tests.rs"]
mod integration_tests;

#[cfg(feature = "repl")]
#[path = "undo_redo/comprehensive_tests.rs"]
mod comprehensive_tests;

// Re-export common helpers and add undo/redo specific extensions
#[cfg(feature = "repl")]
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
        
        fn redo_expect(&mut self, expected: &str) -> &mut Self {
            let ctx = self.context("Redo");
            let result = self.inner.interpret("/redo")
                .unwrap_or_else(|e| panic!("{}: {:?}", ctx, e));
            assert_contains_with_context(&result, expected, &ctx);
            self
        }
        
        fn redo_should_fail(&mut self) -> &mut Self {
            let ctx = self.context("Redo (expecting failure)");
            if self.inner.interpret("/redo").is_ok() {
                panic!("{}: Expected redo to fail but it succeeded", ctx);
            }
            self
        }
    }
    
    // Already imported via pub use crate::common::*;
}