//! Execution module for running commands and instructions
//!
//! This module handles the execution of parsed commands, including
//! hardware instructions and pseudo-instructions.

use super::formatter;
use super::types::{Command, Error};
use crate::rv32_i::{Instruction, PseudoInstruction};

/// Executes a command and returns the result as a string
///
/// # Command Types
///
/// - **Exec**: Execute a hardware instruction
/// - **ExecPseudo**: Execute a pseudo-instruction
pub fn run_command(
    input: Command,
    interpreter: &mut crate::interpreter::Interpreter,
) -> Result<String, Error> {
    match input {
        Command::Exec(instruction) => execute_with_tracking(instruction, None, interpreter),
        Command::ExecPseudo(pseudo) => execute_pseudo(pseudo, interpreter),
    }
}

/// Executes an instruction with delta tracking for undo/redo
///
/// # State Tracking
///
/// This function captures the StateDelta returned by CPU execution,
/// which includes all state changes and records it in history.
///
/// # Arguments
///
/// - `instruction`: The instruction to execute
/// - `display_name`: Optional name for history display (used for pseudo-instructions)
/// - `interpreter`: The interpreter context
pub fn execute_with_tracking(
    instruction: Instruction,
    display_name: Option<String>,
    interpreter: &mut crate::interpreter::Interpreter,
) -> Result<String, Error> {
    // Execute the instruction and get the delta
    let delta = interpreter.cpu_mut().execute(instruction)?;

    // Record the delta in history
    interpreter.history_mut().record_delta(delta.clone());

    // Format the result
    let instruction_text = display_name.unwrap_or_else(|| instruction.mnemonic().to_string());
    Ok(formatter::format_instruction_result(
        &instruction_text,
        &delta,
    ))
}

/// Executes a pseudo-instruction by expanding it and running the real instructions
///
/// # Pseudo-instruction Expansion
///
/// Pseudo-instructions are assembly conveniences that expand to one or more
/// real instructions. For example:
/// - `MV x1, x2` expands to `ADDI x1, x2, 0`
/// - `LI x1, 0x12345` may expand to `LUI` + `ADDI` for large values
///
/// # State Tracking
///
/// Each expanded instruction is executed with the pseudo-instruction's name
/// for better history display in the REPL.
pub fn execute_pseudo(
    pseudo: PseudoInstruction,
    interpreter: &mut crate::interpreter::Interpreter,
) -> Result<String, Error> {
    // Get a nice display name for the pseudo-instruction
    let pseudo_name = format!("{pseudo:?}"); // We'll improve this later

    let instructions = pseudo
        .expand()
        .map_err(|e| Error::Generic(format!("Failed to expand pseudo-instruction: {e}")))?;

    let mut results = Vec::new();
    let mut instruction_names = Vec::new();
    for inst in instructions {
        instruction_names.push(inst.mnemonic().to_string());
        // Execute with the pseudo-instruction name for history
        match execute_with_tracking(inst, Some(pseudo_name.clone()), interpreter) {
            Ok(result) => results.push(result),
            Err(e) => return Err(e),
        }
    }

    if results.len() == 1 {
        Ok(format!(
            "Pseudo-instruction {} expanded to {}: {}",
            pseudo_name, instruction_names[0], results[0]
        ))
    } else {
        Ok(format!(
            "Pseudo-instruction {} expanded to [{}]: {}",
            pseudo_name,
            instruction_names.join(", "),
            results.join("; ")
        ))
    }
}
