//! Validation functions for instruction building
//!
//! This module contains validation logic used during instruction parsing
//! and building to ensure correct arguments, immediate ranges, and register usage.

use super::types::Error;
use crate::rv32_i::Register;

/// Validates that the number of arguments matches expected count
///
/// # Arguments
/// * `instruction` - Name of the instruction being validated
/// * `expected` - Expected number of arguments
/// * `found` - Actual number of arguments provided
pub fn validate_argument_count(
    instruction: &str,
    expected: usize,
    found: usize,
) -> Result<(), Error> {
    if found != expected {
        Err(Error::WrongArgumentCount {
            instruction: instruction.to_string(),
            expected: format!("{expected} arguments"),
            found,
        })
    } else {
        Ok(())
    }
}

/// Validates that an immediate value is within the allowed range
///
/// # Arguments
/// * `instruction` - Name of the instruction being validated
/// * `value` - The immediate value to validate
/// * `min` - Minimum allowed value (inclusive)
/// * `max` - Maximum allowed value (inclusive)
pub fn validate_immediate_range(
    instruction: &str,
    value: i32,
    min: i32,
    max: i32,
) -> Result<(), Error> {
    if value < min || value > max {
        Err(Error::ImmediateOutOfRange {
            instruction: instruction.to_string(),
            value,
            range: format!("{min} to {max}"),
        })
    } else {
        Ok(())
    }
}

/// Validates that the PC register is not used inappropriately
///
/// The PC register has special semantics and cannot be used as a general-purpose
/// register in most instructions. It's only accessible via AUIPC or as an
/// implicit operand in jump instructions.
///
/// # Arguments
/// * `reg` - The register to validate
/// * `position` - Description of where the register is being used (for error messages)
pub fn validate_not_pc(reg: Register, position: &str) -> Result<(), Error> {
    if reg == Register::PC {
        Err(Error::Generic(format!(
            "PC register cannot be used as {position} in this instruction. PC is only accessible via AUIPC or as an implicit operand in jumps."
        )))
    } else {
        Ok(())
    }
}
