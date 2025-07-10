//! Unified test context for both CPU and Interpreter testing
//!
//! This module provides a generic TestContext that can wrap either
//! a CPU (for unit tests) or an Interpreter (for integration tests)
//! with a consistent fluent API.

use brubeck::rv32_i::{CPU, Register};
use brubeck::interpreter::Interpreter;
use super::assertions::*;

/// Generic test context that provides a fluent API for testing
pub struct TestContext<T> {
    pub inner: T,
    pub test_name: String,
}

// ==================== COMMON IMPLEMENTATION ====================

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

impl TestContext<CPU> {
    /// Create a new test context for CPU testing
    pub fn new_cpu() -> Self {
        Self {
            inner: CPU::default(),
            test_name: String::new(),
        }
    }
    
    /// Create with custom memory size
    pub fn new_cpu_with_memory(size: usize) -> Self {
        Self {
            inner: CPU::new(size),
            test_name: String::new(),
        }
    }
    
    /// Set a register value
    pub fn set_reg(&mut self, reg: Register, value: u32) -> &mut Self {
        self.inner.set_register(reg, value);
        self
    }
    
    /// Get a register value
    pub fn get_reg(&self, reg: Register) -> u32 {
        self.inner.get_register(reg)
    }
    
    /// Execute an instruction directly
    pub fn execute(&mut self, inst: brubeck::rv32_i::Instruction) -> &mut Self {
        let ctx = self.context("Execute instruction");
        self.inner.execute(inst)
            .unwrap_or_else(|e| panic!("{}: {:?}", ctx, e));
        self
    }
    
    /// Check register value
    pub fn check_reg_value(&self, reg: Register, expected: u32) -> &Self {
        let actual = self.get_reg(reg);
        let ctx = self.context(&format!("Check {:?}", reg));
        assert_with_context(actual, expected, &ctx, 
            &format!("Register {:?} mismatch", reg));
        self
    }
    
    /// Write to memory
    pub fn write_memory(&mut self, addr: u32, bytes: &[u8]) -> &mut Self {
        let start = addr as usize;
        self.inner.memory[start..start + bytes.len()].copy_from_slice(bytes);
        self
    }
}

// ==================== INTERPRETER-SPECIFIC IMPLEMENTATION ====================

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
        let ctx = self.context(&format!("Execute '{}'", instruction));
        self.inner.interpret(instruction)
            .unwrap_or_else(|e| panic!("{}: {:?}", ctx, e));
        self
    }
    
    /// Execute and expect failure
    pub fn exec_fail(&mut self, instruction: &str) -> String {
        let ctx = self.context(&format!("Execute '{}' (expecting failure)", instruction));
        self.inner.interpret(instruction)
            .err()
            .unwrap_or_else(|| panic!("{}: Expected failure but succeeded", ctx))
            .to_string()
    }
    
    /// Check register contains value
    pub fn check_reg(&mut self, reg: &str, expected: &str) -> &mut Self {
        let ctx = self.context(&format!("Check {}", reg));
        let result = self.inner.interpret(reg)
            .unwrap_or_else(|e| panic!("{}: Failed to read register: {:?}", ctx, e));
        assert_contains_with_context(&result, expected, &ctx);
        self
    }
    
    /// Check multiple registers are zero
    pub fn check_regs_zero(&mut self, start: u32, end: u32) -> &mut Self {
        for i in start..=end {
            self.check_reg(&format!("x{}", i), "0");
        }
        self
    }
    
    /// Get register value as string
    pub fn get_reg_str(&mut self, reg: &str) -> String {
        let ctx = self.context(&format!("Get {}", reg));
        self.inner.interpret(reg)
            .unwrap_or_else(|e| panic!("{}: Failed to read register: {:?}", ctx, e))
    }
    
    /// Get the current PC value
    pub fn get_pc(&self) -> u32 {
        self.inner.get_pc()
    }
    
    /// Load a value into a register efficiently
    pub fn load_reg(&mut self, reg: &str, value: i32) -> &mut Self {
        use super::values::fits_in_imm12;
        
        if fits_in_imm12(value) {
            self.exec(&format!("ADDI {}, x0, {}", reg, value))
        } else {
            self.exec(&format!("LI {}, {}", reg, value))
        }
    }
    
    /// Undo last operation
    #[cfg(feature = "repl")]
    pub fn undo(&mut self) -> &mut Self {
        let ctx = self.context("Undo");
        self.inner.interpret("/undo")
            .unwrap_or_else(|e| panic!("{}: {:?}", ctx, e));
        self
    }
    
    /// Undo with expected content
    #[cfg(feature = "repl")]
    pub fn undo_expect(&mut self, expected: &str) -> &mut Self {
        let ctx = self.context("Undo");
        let result = self.inner.interpret("/undo")
            .unwrap_or_else(|e| panic!("{}: {:?}", ctx, e));
        assert_contains_with_context(&result, expected, &ctx);
        self
    }
    
    /// Redo last undone operation
    #[cfg(feature = "repl")]
    pub fn redo(&mut self) -> &mut Self {
        let ctx = self.context("Redo");
        self.inner.interpret("/redo")
            .unwrap_or_else(|e| panic!("{}: {:?}", ctx, e));
        self
    }
    
    /// Check undo should fail
    #[cfg(feature = "repl")]
    pub fn undo_should_fail(&mut self) -> &mut Self {
        let ctx = self.context("Undo (expecting failure)");
        if self.inner.interpret("/undo").is_ok() {
            panic!("{}: Expected undo to fail but it succeeded", ctx);
        }
        self
    }
}

// ==================== CONVERSION HELPERS ====================

/// Create a CPU test context
pub fn cpu_context() -> TestContext<CPU> {
    TestContext::new_cpu()
}

/// Create an Interpreter test context
pub fn interpreter_context() -> TestContext<Interpreter> {
    TestContext::new()
}

// ==================== TEST MACROS ====================

/// Macro to create a test with a TestContext
#[macro_export]
macro_rules! context_test {
    ($name:ident, $context_type:ident, $body:expr) => {
        #[test]
        fn $name() {
            let mut ctx = super::$context_type();
            ctx.test_name = stringify!($name).to_string();
            $body(&mut ctx);
        }
    };
}

/// Macro for CPU unit tests
#[macro_export]
macro_rules! cpu_test {
    ($name:ident, $body:expr) => {
        context_test!($name, cpu_context, $body);
    };
}

/// Macro for interpreter integration tests
#[macro_export]
macro_rules! interpreter_test {
    ($name:ident, $body:expr) => {
        context_test!($name, interpreter_context, $body);
    };
}