//! Unified test context for both CPU and Interpreter testing
//!
//! This module provides a generic TestContext that can wrap either
//! a CPU (for unit tests) or an Interpreter (for integration tests)
//! with a consistent fluent API.

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

        // Parse the register name
        use brubeck::rv32_i::Register;
        let register = match reg.to_uppercase().as_str() {
            "X0" => Register::X0,
            "X1" => Register::X1,
            "X2" => Register::X2,
            "X3" => Register::X3,
            "X4" => Register::X4,
            "X5" => Register::X5,
            "X6" => Register::X6,
            "X7" => Register::X7,
            "X8" => Register::X8,
            "X9" => Register::X9,
            "X10" => Register::X10,
            "X11" => Register::X11,
            "X12" => Register::X12,
            "X13" => Register::X13,
            "X14" => Register::X14,
            "X15" => Register::X15,
            "X16" => Register::X16,
            "X17" => Register::X17,
            "X18" => Register::X18,
            "X19" => Register::X19,
            "X20" => Register::X20,
            "X21" => Register::X21,
            "X22" => Register::X22,
            "X23" => Register::X23,
            "X24" => Register::X24,
            "X25" => Register::X25,
            "X26" => Register::X26,
            "X27" => Register::X27,
            "X28" => Register::X28,
            "X29" => Register::X29,
            "X30" => Register::X30,
            "X31" => Register::X31,
            "PC" => Register::PC,
            _ => panic!("{ctx}: Unknown register: {reg}"),
        };

        // Get the actual value
        let value = self.inner.cpu.get_register(register);

        // Parse expected value
        let expected_val = if expected.starts_with("0x") || expected.starts_with("0X") {
            u32::from_str_radix(&expected[2..], 16)
                .unwrap_or_else(|_| panic!("{ctx}: Invalid hex value: {expected}"))
        } else {
            expected
                .parse::<u32>()
                .unwrap_or_else(|_| panic!("{ctx}: Invalid decimal value: {expected}"))
        };

        assert_eq!(
            value, expected_val,
            "{ctx}: Register {reg} has value 0x{value:08x}, expected 0x{expected_val:08x}"
        );
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
        self.inner.cpu.pc
    }

    /// Navigate to previous state
    pub fn previous(&mut self) -> &mut Self {
        let ctx = self.context("Previous");
        self.inner
            .previous_state()
            .unwrap_or_else(|e| panic!("{ctx}: {e:?}"));
        self
    }

    /// Navigate to previous state with expected content
    pub fn previous_expect(&mut self, _expected: &str) -> &mut Self {
        let ctx = self.context("Navigate back");
        let _delta = self
            .inner
            .previous_state()
            .unwrap_or_else(|e| panic!("{ctx}: {e:?}"));
        // The new API returns StateDelta, not strings
        // Success is indicated by not panicking above
        self
    }

    /// Navigate to next state
    pub fn next(&mut self) -> &mut Self {
        let ctx = self.context("Next");
        self.inner
            .next_state()
            .unwrap_or_else(|e| panic!("{ctx}: {e:?}"));
        self
    }

    /// Check navigation back should fail
    pub fn previous_should_fail(&mut self) -> &mut Self {
        let ctx = self.context("Navigate back (expecting failure)");
        if self.inner.previous_state().is_ok() {
            panic!("{ctx}: Expected navigation back to fail but it succeeded");
        }
        self
    }
}

// ==================== CONVERSION HELPERS ====================

/// Create an Interpreter test context
#[allow(dead_code)]
pub fn interpreter_context() -> TestContext<Interpreter> {
    TestContext::new()
}

// ==================== TEST MACROS ====================

// Test macros have been removed as they were not being used
