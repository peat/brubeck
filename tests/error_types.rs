//! Tests for the new error types

use brubeck::{ExecutionError, HistoryError, ParseError};

#[test]
fn test_parse_error_display() {
    let err = ParseError::UnknownInstruction {
        instruction: "ADDDD".to_string(),
        suggestion: Some("ADD".to_string()),
    };
    assert_eq!(
        err.to_string(),
        "Unknown instruction 'ADDDD'. Did you mean 'ADD'?"
    );

    let err = ParseError::InvalidRegister {
        register: "x99".to_string(),
    };
    assert_eq!(err.to_string(), "Invalid register 'x99'");

    let err = ParseError::WrongArgumentCount {
        instruction: "ADD".to_string(),
        expected: 3,
        found: 2,
    };
    assert_eq!(
        err.to_string(),
        "ADD expects 3 arguments, but 2 were provided"
    );

    let err = ParseError::ImmediateOutOfRange {
        instruction: "ADDI".to_string(),
        value: 5000,
        min: -2048,
        max: 2047,
    };
    assert_eq!(
        err.to_string(),
        "Immediate value 5000 out of range for ADDI (valid range: -2048 to 2047)"
    );
}

#[test]
fn test_history_error_display() {
    let err = HistoryError::AtBeginning;
    assert_eq!(err.to_string(), "Already at the beginning of history");

    let err = HistoryError::AtEnd;
    assert_eq!(err.to_string(), "Already at the most recent state");
}

#[test]
fn test_execution_error_conversion() {
    let parse_err = ParseError::SyntaxError {
        message: "Unexpected token".to_string(),
    };
    let exec_err: ExecutionError = parse_err.into();
    assert_eq!(exec_err.to_string(), "Parse error: Unexpected token");
}

#[test]
fn test_error_equality() {
    let err1 = ParseError::InvalidRegister {
        register: "x99".to_string(),
    };
    let err2 = ParseError::InvalidRegister {
        register: "x99".to_string(),
    };
    assert_eq!(err1, err2);

    let err1 = HistoryError::AtBeginning;
    let err2 = HistoryError::AtBeginning;
    assert_eq!(err1, err2);
}
