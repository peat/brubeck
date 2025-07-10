//! Integration tests for command-line argument parsing
//!
//! These tests verify that command-line arguments are correctly parsed
//! and applied to the interpreter configuration.

#[path = "common/mod.rs"]
mod common;

use common::*;

// ==================== MEMORY SIZE PARSER TESTS ====================

#[test]
fn test_parse_memory_size_bytes() {
    use brubeck::cli::parse_memory_size;
    
    // Plain numbers are bytes
    assert_eq!(parse_memory_size("0").unwrap(), 0);
    assert_eq!(parse_memory_size("1024").unwrap(), 1024);
    assert_eq!(parse_memory_size("999999").unwrap(), 999999);
}

#[test]
fn test_parse_memory_size_kilobytes() {
    use brubeck::cli::parse_memory_size;
    
    // Lowercase k
    assert_eq!(parse_memory_size("1k").unwrap(), 1024);
    assert_eq!(parse_memory_size("256k").unwrap(), 256 * 1024);
    
    // Uppercase K
    assert_eq!(parse_memory_size("1K").unwrap(), 1024);
    assert_eq!(parse_memory_size("1024K").unwrap(), 1024 * 1024);
}

#[test]
fn test_parse_memory_size_megabytes() {
    use brubeck::cli::parse_memory_size;
    
    // Lowercase m
    assert_eq!(parse_memory_size("1m").unwrap(), 1024 * 1024);
    assert_eq!(parse_memory_size("5m").unwrap(), 5 * 1024 * 1024);
    
    // Uppercase M
    assert_eq!(parse_memory_size("1M").unwrap(), 1024 * 1024);
    assert_eq!(parse_memory_size("10M").unwrap(), 10 * 1024 * 1024);
}

#[test]
fn test_parse_memory_size_invalid() {
    use brubeck::cli::parse_memory_size;
    
    // Invalid formats
    assert!(parse_memory_size("").is_err());
    assert!(parse_memory_size("xyz").is_err());
    assert!(parse_memory_size("1.5M").is_err());
    assert!(parse_memory_size("-5").is_err());
    assert!(parse_memory_size("1gb").is_err());
    assert!(parse_memory_size("1G").is_err());
    
    // Invalid suffixes
    assert!(parse_memory_size("1x").is_err());
    assert!(parse_memory_size("1kB").is_err());
    assert!(parse_memory_size("1MB").is_err());
}

#[test]
fn test_parse_memory_size_edge_cases() {
    use brubeck::cli::parse_memory_size;
    
    // Zero with suffix
    assert_eq!(parse_memory_size("0k").unwrap(), 0);
    assert_eq!(parse_memory_size("0M").unwrap(), 0);
    
    // Large values that might overflow
    let result = parse_memory_size("4096M");
    if result.is_ok() {
        // Should handle up to 4GB if usize allows
        assert_eq!(result.unwrap(), 4096 * 1024 * 1024);
    } else {
        // Or gracefully fail on 32-bit systems
        assert!(result.is_err());
    }
}

// ==================== CLI STRUCTURE TESTS ====================

#[test]
fn test_cli_defaults() {
    use brubeck::cli::Cli;
    use clap::Parser;
    
    let cli = Cli::parse_from(&["brubeck"]);
    
    assert_eq!(cli.memory, "1M");
    assert_eq!(cli.undo_limit, 1000);
    assert!(!cli.no_undo);
    assert!(cli.execute.is_none());
    assert!(cli.script.is_none());
}

#[test]
fn test_cli_memory_flag() {
    use brubeck::cli::Cli;
    use clap::Parser;
    
    // Short form
    let cli = Cli::parse_from(&["brubeck", "-m", "256k"]);
    assert_eq!(cli.memory, "256k");
    
    // Long form
    let cli = Cli::parse_from(&["brubeck", "--memory", "2M"]);
    assert_eq!(cli.memory, "2M");
}

#[test]
fn test_cli_undo_flags() {
    use brubeck::cli::Cli;
    use clap::Parser;
    
    // Custom undo limit
    let cli = Cli::parse_from(&["brubeck", "--undo-limit", "500"]);
    assert_eq!(cli.undo_limit, 500);
    assert!(!cli.no_undo);
    
    // No undo flag
    let cli = Cli::parse_from(&["brubeck", "--no-undo"]);
    assert!(cli.no_undo);
    
    // Both flags (no-undo should take precedence)
    let cli = Cli::parse_from(&["brubeck", "--undo-limit", "100", "--no-undo"]);
    assert!(cli.no_undo);
}

#[test]
fn test_cli_execute_flag() {
    use brubeck::cli::Cli;
    use clap::Parser;
    
    // Short form
    let cli = Cli::parse_from(&["brubeck", "-e", "ADDI x1, x0, 10"]);
    assert_eq!(cli.execute, Some("ADDI x1, x0, 10".to_string()));
    
    // Long form with semicolons
    let cli = Cli::parse_from(&["brubeck", "--execute", "ADDI x1, x0, 10; x1"]);
    assert_eq!(cli.execute, Some("ADDI x1, x0, 10; x1".to_string()));
}

#[test]
fn test_cli_script_flag() {
    use brubeck::cli::Cli;
    use clap::Parser;
    
    // Short form
    let cli = Cli::parse_from(&["brubeck", "-s", "test.bru"]);
    assert_eq!(cli.script, Some("test.bru".to_string()));
    
    // Long form
    let cli = Cli::parse_from(&["brubeck", "--script", "/path/to/script.bru"]);
    assert_eq!(cli.script, Some("/path/to/script.bru".to_string()));
}

#[test]
fn test_cli_conflicting_modes() {
    use brubeck::cli::Cli;
    use clap::Parser;
    
    // Can't use both execute and script
    let result = Cli::try_parse_from(&["brubeck", "-e", "x1", "-s", "script.bru"]);
    assert!(result.is_err());
}

// ==================== INTEGRATION TESTS ====================

#[test]
fn test_memory_size_applied() {
    use brubeck::interpreter::Interpreter;
    use brubeck::cli::{Config, parse_memory_size};
    
    let config = Config {
        memory_size: parse_memory_size("64k").unwrap(),
        undo_limit: 1000,
    };
    
    let interpreter = Interpreter::with_config(config);
    
    // Verify memory size was applied
    // This assumes we add a method to check memory size
    assert_eq!(interpreter.memory_size(), 64 * 1024);
}

#[test]
fn test_undo_limit_applied() {
    use brubeck::interpreter::Interpreter;
    use brubeck::cli::Config;
    
    // Zero undo limit (disabled)
    let config = Config {
        memory_size: 1024 * 1024,
        undo_limit: 0,
    };
    
    let mut interpreter = Interpreter::with_config(config);
    
    // Execute something
    interpreter.interpret("ADDI x1, x0, 10").unwrap();
    
    // Undo should fail
    let result = interpreter.interpret("/undo");
    assert!(result.is_err() || result.unwrap().contains("No history"));
}

#[test]
fn test_execute_mode() {
    use brubeck::cli::run_execute_mode;
    
    // Single command
    let output = run_execute_mode("ADDI x1, x0, 42; x1");
    assert!(output.contains("42"));
    
    // Multiple commands
    let output = run_execute_mode("ADDI x1, x0, 10; ADDI x2, x0, 20; ADD x3, x1, x2; x3");
    assert!(output.contains("30"));
}

#[test]
fn test_script_mode() {
    use brubeck::cli::run_script_mode;
    use std::fs;
    
    // Create a test script
    let script_content = "ADDI x1, x0, 100\nSLLI x2, x1, 2\nx2\n";
    fs::write("test_script.bru", script_content).unwrap();
    
    // Run it
    let output = run_script_mode("test_script.bru").unwrap();
    assert!(output.contains("400")); // 100 << 2 = 400
    
    // Clean up
    fs::remove_file("test_script.bru").unwrap();
}

#[test]
fn test_script_mode_missing_file() {
    use brubeck::cli::run_script_mode;
    
    let result = run_script_mode("nonexistent.bru");
    assert!(result.is_err());
}

// ==================== SEMICOLON PARSING TESTS ====================

interpreter_test!(test_semicolon_basic, |ctx| {
        // Single line with multiple commands
        ctx.exec("ADDI x1, x0, 10; ADDI x2, x0, 20; ADD x3, x1, x2")
           .check_reg("x1", "10")
           .check_reg("x2", "20")
           .check_reg("x3", "30");
    });
}

interpreter_test!(test_semicolon_spaces, |ctx| {
        // Various spacing around semicolons
        ctx.exec("ADDI x1, x0, 5;ADDI x2, x0, 3")    // No spaces
           .exec("ADDI x3, x0, 7 ; ADDI x4, x0, 9")  // Spaces around
           .exec("ADDI x5, x0, 11;  ADDI x6, x0, 13") // Space after
           .check_reg("x1", "5")
           .check_reg("x2", "3")
           .check_reg("x3", "7")
           .check_reg("x4", "9")
           .check_reg("x5", "11")
           .check_reg("x6", "13");
    });
}

interpreter_test!(test_semicolon_inspection, |ctx| {
        // Mix instructions and inspections
        let output = ctx.exec_with_output("ADDI x1, x0, 42; x1; ADDI x2, x1, 8; x2");
        assert!(output.contains("42"));
        assert!(output.contains("50"));
    });
}

interpreter_test!(test_semicolon_empty, |ctx| {
        // Empty commands should be ignored
        ctx.exec("ADDI x1, x0, 1;;ADDI x2, x0, 2")   // Double semicolon
           .exec(";ADDI x3, x0, 3")                   // Leading semicolon
           .exec("ADDI x4, x0, 4;")                   // Trailing semicolon
           .check_reg("x1", "1")
           .check_reg("x2", "2")
           .check_reg("x3", "3")
           .check_reg("x4", "4");
    });
}

interpreter_test!(test_semicolon_errors, |ctx| {
        // Error in middle shouldn't affect other commands
        let result = ctx.exec_may_fail("ADDI x1, x0, 10; INVALID x2; ADDI x3, x0, 30");
        
        // First command should have succeeded
        ctx.check_reg("x1", "10");
        
        // Third command should not have executed
        ctx.check_reg("x3", "0");
        
        // Error message should be about INVALID
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("INVALID"));
    });
}

// ==================== BANNER SUPPRESSION TESTS ====================

#[test]
fn test_banner_suppression() {
    use brubeck::cli::{should_show_banner, ExecutionMode};
    
    // Interactive mode shows banner
    assert!(should_show_banner(ExecutionMode::Interactive));
    
    // Non-interactive modes suppress banner
    assert!(!should_show_banner(ExecutionMode::Execute));
    assert!(!should_show_banner(ExecutionMode::Script));
}

// ==================== HELPER METHODS ====================

// Add extension methods to TestContext for testing multi-command output
#[cfg(feature = "repl")]
impl crate::common::TestContext<brubeck::interpreter::Interpreter> {
    /// Execute and return the full output (for testing semicolon-separated commands)
    pub fn exec_with_output(&mut self, commands: &str) -> String {
        let ctx = self.context(&format!("Execute '{}'", commands));
        self.inner.interpret(commands)
            .unwrap_or_else(|e| panic!("{}: {:?}", ctx, e))
    }
    
    /// Execute and allow failure
    pub fn exec_may_fail(&mut self, commands: &str) -> Result<String, brubeck::interpreter::Error> {
        self.inner.interpret(commands)
    }
}