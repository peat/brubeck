//! Common assertion traits for both unit and integration tests
//!
//! These traits provide consistent error messages and assertion patterns
//! across different test types.

use std::fmt::Display;

/// Trait for asserting register values
pub trait RegisterAssertions {
    /// Assert register contains a specific string value
    fn assert_register_contains(&self, reg: &str, expected: &str, context: &str);
    
    /// Assert register has exact numeric value
    fn assert_register_equals(&self, reg: &str, expected: u32, context: &str);
    
    /// Assert multiple registers are zero
    fn assert_registers_zero(&self, start: u32, end: u32, context: &str);
}

/// Trait for asserting memory state
pub trait MemoryAssertions {
    /// Assert memory word at address (little-endian)
    fn assert_memory_word(&self, addr: u32, expected: u32, context: &str);
    
    /// Assert memory bytes match expected
    fn assert_memory_bytes(&self, addr: u32, expected: &[u8], context: &str);
}

/// Trait for asserting execution results
pub trait ExecutionAssertions {
    /// Execute and expect success
    fn assert_execute_success(&mut self, instruction: &str, context: &str);
    
    /// Execute and expect specific error
    fn assert_execute_fails(&mut self, instruction: &str, expected_error: &str, context: &str);
    
    /// Execute and check result contains string
    fn assert_execute_result(&mut self, instruction: &str, expected: &str, context: &str);
}

/// Helper for creating detailed assertion messages
pub fn assert_with_context<T: PartialEq + Display>(
    actual: T,
    expected: T,
    context: &str,
    detail: &str,
) {
    if actual != expected {
        panic!(
            "{}: {}\n  Expected: {}\n  Actual:   {}\n",
            context, detail, expected, actual
        );
    }
}

/// Helper for asserting string contains substring with context
pub fn assert_contains_with_context(
    haystack: &str,
    needle: &str,
    context: &str,
) {
    if !haystack.contains(needle) {
        panic!(
            "{}: Expected to find '{}' in output\n  Actual output: {}",
            context, needle, haystack
        );
    }
}

/// Trait for common test patterns
pub trait TestPatterns {
    /// Test that an operation can be undone and redone
    fn test_undo_redo(&mut self, instruction: &str, check_reg: &str, 
                      expected_after: &str, expected_before: &str);
    
    /// Test a sequence of operations
    fn test_sequence(&mut self, instructions: &[&str], 
                     expected_results: &[(&str, &str)]);
}

/// Format a value in multiple representations for debugging
pub fn format_value(val: u32) -> String {
    format!(
        "{} (0x{:08X}, {})",
        val,
        val,
        val as i32
    )
}

/// Format a register name consistently
pub fn format_register(reg: &str) -> String {
    if reg.starts_with("x") || reg.starts_with("X") {
        reg.to_lowercase()
    } else {
        // Handle ABI names if needed
        reg.to_string()
    }
}

/// Create a descriptive test failure message
#[macro_export]
macro_rules! test_assert {
    ($cond:expr, $context:expr, $($arg:tt)*) => {
        if !$cond {
            panic!("{}: {}", $context, format!($($arg)*));
        }
    };
}

/// Create a descriptive equality assertion
#[macro_export]
macro_rules! test_assert_eq {
    ($left:expr, $right:expr, $context:expr) => {
        if $left != $right {
            panic!(
                "{}: Assertion failed\n  Left:  {:?}\n  Right: {:?}",
                $context, $left, $right
            );
        }
    };
    ($left:expr, $right:expr, $context:expr, $($arg:tt)*) => {
        if $left != $right {
            panic!(
                "{}: {}\n  Left:  {:?}\n  Right: {:?}",
                $context, format!($($arg)*), $left, $right
            );
        }
    };
}