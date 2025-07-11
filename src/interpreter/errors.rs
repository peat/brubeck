//! Error types for the interpreter
//!
//! This module defines structured error types for different layers of the interpreter:
//! - `ParseError` for parsing failures
//! - `HistoryError` for navigation failures
//! - Re-exports `CPUError` for execution failures

use std::fmt;

/// Errors that can occur during parsing
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// Unknown instruction mnemonic
    UnknownInstruction {
        instruction: String,
        suggestion: Option<String>,
    },
    /// Invalid register name
    InvalidRegister { register: String },
    /// Wrong number of arguments for an instruction
    WrongArgumentCount {
        instruction: String,
        expected: usize,
        found: usize,
    },
    /// Immediate value out of valid range
    ImmediateOutOfRange {
        instruction: String,
        value: i32,
        min: i32,
        max: i32,
    },
    /// General syntax error
    SyntaxError { message: String },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnknownInstruction {
                instruction,
                suggestion,
            } => {
                write!(f, "Unknown instruction '{instruction}'")?;
                if let Some(s) = suggestion {
                    write!(f, ". Did you mean '{s}'?")?;
                }
                Ok(())
            }
            ParseError::InvalidRegister { register } => {
                write!(f, "Invalid register '{register}'")
            }
            ParseError::WrongArgumentCount {
                instruction,
                expected,
                found,
            } => {
                write!(
                    f,
                    "{} expects {} arguments, but {} {} provided",
                    instruction,
                    expected,
                    found,
                    if *found == 1 { "was" } else { "were" }
                )
            }
            ParseError::ImmediateOutOfRange {
                instruction,
                value,
                min,
                max,
            } => {
                write!(
                    f,
                    "Immediate value {value} out of range for {instruction} (valid range: {min} to {max})"
                )
            }
            ParseError::SyntaxError { message } => {
                write!(f, "{message}")
            }
        }
    }
}

impl std::error::Error for ParseError {}

/// Errors that can occur during history navigation
#[derive(Debug, Clone, PartialEq)]
pub enum HistoryError {
    /// Cannot navigate backward - already at the beginning
    AtBeginning,
    /// Cannot navigate forward - already at the end
    AtEnd,
}

impl fmt::Display for HistoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HistoryError::AtBeginning => {
                write!(f, "Already at the beginning of history")
            }
            HistoryError::AtEnd => {
                write!(f, "Already at the most recent state")
            }
        }
    }
}

impl std::error::Error for HistoryError {}

/// Errors that can occur during execution
///
/// This is a re-export of the CPU error type for convenience
pub use crate::rv32_i::CPUError;

/// Combined error type for the interpret() convenience method
///
/// This will be used by the binary, not the refactored library API
#[derive(Debug)]
pub enum ExecutionError {
    ParseError(ParseError),
    CPUError(CPUError),
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionError::ParseError(e) => write!(f, "Parse error: {e}"),
            ExecutionError::CPUError(e) => write!(f, "Execution error: {e}"),
        }
    }
}

impl std::error::Error for ExecutionError {}

impl From<ParseError> for ExecutionError {
    fn from(err: ParseError) -> Self {
        ExecutionError::ParseError(err)
    }
}

impl From<CPUError> for ExecutionError {
    fn from(err: CPUError) -> Self {
        ExecutionError::CPUError(err)
    }
}
