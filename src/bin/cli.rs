//! Command-line interface for Brubeck
//!
//! This module handles parsing command-line arguments and provides
//! utilities for different execution modes.

use std::fmt;

use clap::Parser;

/// Command-line arguments for Brubeck
#[derive(Parser, Debug)]
#[command(name = "brubeck")]
#[command(about = "A RISC-V assembly REPL and emulator", long_about = None)]
#[command(version)]
pub struct Cli {
    /// Memory size (e.g., 1M, 256k, 1024)
    #[arg(short = 'm', long = "memory", default_value = "1M")]
    pub memory: String,

    /// Maximum history navigation depth
    #[arg(long = "history-limit", default_value_t = 1000)]
    pub history_limit: usize,

    /// Disable state history navigation functionality
    #[arg(long = "no-state-history", conflicts_with = "history_limit")]
    pub no_state_history: bool,

    /// Maximum command history size
    #[arg(long = "history-size", default_value_t = 1000)]
    pub history_size: usize,

    /// Disable command history
    #[arg(long = "no-history", conflicts_with = "history_size")]
    pub no_history: bool,

    /// Execute commands and exit (semicolon-separated)
    #[arg(short = 'e', long = "execute", conflicts_with = "script")]
    pub execute: Option<String>,

    /// Execute script file and exit
    #[arg(short = 's', long = "script", conflicts_with = "execute")]
    pub script: Option<String>,

    /// Suppress banner and instruction descriptions (REPL only)
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Show instruction trace with PC and descriptions (script/execute only)
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Disable color output
    #[arg(long = "no-color")]
    pub no_color: bool,

    /// Enable educational tips in error messages
    #[arg(long = "tips")]
    pub tips: bool,
}

/// Configuration derived from CLI arguments
#[derive(Debug, Clone)]
pub struct Config {
    pub memory_size: usize,
    pub history_limit: usize,
}

impl Config {
    /// Creates a new configuration with validation
    pub fn new(memory_size: usize, history_limit: usize) -> Result<Self, String> {
        // For now, we accept any memory size that fits in usize
        // Could add validation here if needed
        Ok(Self {
            memory_size,
            history_limit,
        })
    }
}

/// Error type for memory size parsing
#[derive(Debug, Clone, PartialEq)]
pub struct ParseMemoryError {
    message: String,
}

impl fmt::Display for ParseMemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid memory size: {}", self.message)
    }
}

impl std::error::Error for ParseMemoryError {}

/// Maximum allowed memory size (1GB)
const MAX_MEMORY_SIZE: usize = 1024 * 1024 * 1024;

/// Parses a human-readable memory size string into bytes
///
/// Maximum allowed size is 1GB.
///
/// # Examples
/// ```
/// use brubeck::cli::parse_memory_size;
///
/// assert_eq!(parse_memory_size("1024").unwrap(), 1024);
/// assert_eq!(parse_memory_size("1k").unwrap(), 1024);
/// assert_eq!(parse_memory_size("1K").unwrap(), 1024);
/// assert_eq!(parse_memory_size("1m").unwrap(), 1048576);
/// assert_eq!(parse_memory_size("1M").unwrap(), 1048576);
/// ```
pub fn parse_memory_size(s: &str) -> Result<usize, ParseMemoryError> {
    let s = s.trim();

    if s.is_empty() {
        return Err(ParseMemoryError {
            message: "empty string".to_string(),
        });
    }

    // Check if last character is a unit suffix
    let last_char = s.chars().last().unwrap();

    let (number_part, multiplier) = match last_char {
        'k' | 'K' => {
            let num_str = &s[..s.len() - 1];
            (num_str, 1024)
        }
        'm' | 'M' => {
            let num_str = &s[..s.len() - 1];
            (num_str, 1024 * 1024)
        }
        _ if last_char.is_ascii_digit() => {
            // No suffix, parse as bytes
            (s, 1)
        }
        _ => {
            return Err(ParseMemoryError {
                message: format!("invalid suffix '{last_char}'"),
            });
        }
    };

    // Parse the numeric part
    let number: u64 = number_part.parse().map_err(|_| ParseMemoryError {
        message: format!("invalid number '{number_part}'"),
    })?;

    // Check for overflow when multiplying
    let result = number
        .checked_mul(multiplier as u64)
        .ok_or_else(|| ParseMemoryError {
            message: "arithmetic overflow".to_string(),
        })?;

    // Check if it fits in usize
    if result > usize::MAX as u64 {
        return Err(ParseMemoryError {
            message: "size too large for platform".to_string(),
        });
    }

    // Check against our maximum limit (1GB)
    let size = result as usize;
    if size > MAX_MEMORY_SIZE {
        return Err(ParseMemoryError {
            message: format!("size exceeds maximum of 1GB (got {size} bytes)"),
        });
    }

    Ok(size)
}

/// Splits a command string by semicolons into individual commands
///
/// Handles trimming whitespace and filtering empty commands
pub fn split_commands(input: &str) -> Vec<&str> {
    input
        .split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Execution mode for the interpreter
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExecutionMode {
    Interactive,
    Execute,
    Script,
}

/// Determines whether to show the banner based on execution mode
pub fn should_show_banner(mode: ExecutionMode) -> bool {
    matches!(mode, ExecutionMode::Interactive)
}

impl Cli {
    /// Converts CLI arguments into a Config
    pub fn to_config(&self) -> Result<Config, ParseMemoryError> {
        let memory_size = parse_memory_size(&self.memory)?;

        let history_limit = if self.no_state_history {
            0
        } else {
            self.history_limit
        };

        Config::new(memory_size, history_limit).map_err(|e| ParseMemoryError { message: e })
    }

    /// Determines the execution mode from CLI arguments
    pub fn execution_mode(&self) -> ExecutionMode {
        if self.execute.is_some() {
            ExecutionMode::Execute
        } else if self.script.is_some() {
            ExecutionMode::Script
        } else {
            ExecutionMode::Interactive
        }
    }
}

// Execute and script mode functions are implemented in main.rs

// Tests have been moved to tests/unit/cli.rs
