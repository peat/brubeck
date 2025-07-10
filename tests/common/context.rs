//! Unified test context for both CPU and Interpreter testing
//!
//! This module provides a generic TestContext that can wrap either
//! a CPU (for unit tests) or an Interpreter (for integration tests)
//! with a consistent fluent API.

use super::assertions::*;
use brubeck::interpreter::Interpreter;

/// Generic test context that provides a fluent API for testing
pub struct TestContext<T> {
    pub inner: T,
    pub test_name: String,
}

// ==================== COMMON IMPLEMENTATION ====================

#[allow(dead_code)] // Methods are used but some only with specific features
impl<T> TestContext<T> {
    /// Set the test name for better error messages
    pub fn with_name(mut self, name: &str) -> Self {
        self.test_name = name.to_string();
        self
    }

    /// Get a context string for error messages
    pub fn context(&self, operation: &str) -> String {
        if self.test_name.is_empty() {
            operation.to_string()
        } else {
            format!("{} - {}", self.test_name, operation)
        }
    }
}

// ==================== CPU-SPECIFIC IMPLEMENTATION ====================
// Note: Currently unused - CPU-specific test context functionality has been removed
// as all current tests use the Interpreter context instead.

// ==================== INTERPRETER-SPECIFIC IMPLEMENTATION ====================

impl Default for TestContext<Interpreter> {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)] // Some methods only used with 'repl' feature
impl TestContext<Interpreter> {
    /// Create a new test context for Interpreter testing
    pub fn new() -> Self {
        Self {
            inner: Interpreter::new(),
            test_name: String::new(),
        }
    }

    /// Execute an instruction string
    pub fn exec(&mut self, instruction: &str) -> &mut Self {
        let ctx = self.context(&format!("Execute '{instruction}'"));
        self.inner
            .interpret(instruction)
            .unwrap_or_else(|e| panic!("{ctx}: {e:?}"));
        self
    }

    /// Execute and expect failure
    pub fn exec_fail(&mut self, instruction: &str) -> String {
        let ctx = self.context(&format!("Execute '{instruction}' (expecting failure)"));
        self.inner
            .interpret(instruction)
            .err()
            .unwrap_or_else(|| panic!("{ctx}: Expected failure but succeeded"))
            .to_string()
    }

    /// Check register contains value
    pub fn check_reg(&mut self, reg: &str, expected: &str) -> &mut Self {
        let ctx = self.context(&format!("Check {reg}"));
        let result = self
            .inner
            .interpret(&format!("/regs {reg}"))
            .unwrap_or_else(|e| panic!("{ctx}: Failed to read register: {e:?}"));
        assert_contains_with_context(&result, expected, &ctx);
        self
    }

    /// Check multiple registers are zero
    pub fn check_regs_zero(&mut self, start: u32, end: u32) -> &mut Self {
        for i in start..=end {
            self.check_reg(&format!("x{i}"), "0");
        }
        self
    }

    /// Get the current PC value
    pub fn get_pc(&self) -> u32 {
        self.inner.get_pc()
    }

    /// Undo last operation
    #[cfg(feature = "repl")]
    pub fn undo(&mut self) -> &mut Self {
        let ctx = self.context("Previous");
        self.inner
            .interpret("/previous")
            .unwrap_or_else(|e| panic!("{ctx}: {e:?}"));
        self
    }

    /// Undo with expected content
    #[cfg(feature = "repl")]
    pub fn undo_expect(&mut self, expected: &str) -> &mut Self {
        let ctx = self.context("Undo");
        let result = self
            .inner
            .interpret("/previous")
            .unwrap_or_else(|e| panic!("{ctx}: {e:?}"));
        assert_contains_with_context(&result, expected, &ctx);
        self
    }

    /// Redo last undone operation
    #[cfg(feature = "repl")]
    pub fn redo(&mut self) -> &mut Self {
        let ctx = self.context("Next");
        self.inner
            .interpret("/next")
            .unwrap_or_else(|e| panic!("{ctx}: {e:?}"));
        self
    }

    /// Check undo should fail
    #[cfg(feature = "repl")]
    pub fn undo_should_fail(&mut self) -> &mut Self {
        let ctx = self.context("Undo (expecting failure)");
        if self.inner.interpret("/previous").is_ok() {
            panic!("{ctx}: Expected undo to fail but it succeeded");
        }
        self
    }
}

// ==================== CONVERSION HELPERS ====================

/// Create an Interpreter test context
pub fn interpreter_context() -> TestContext<Interpreter> {
    TestContext::new()
}

// ==================== TEST MACROS ====================

// Test macros have been removed as they were not being used
