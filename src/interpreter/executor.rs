//! Execution module for running commands and instructions
//!
//! This module handles the execution of parsed commands, including
//! hardware instructions, pseudo-instructions, and REPL commands.

use super::formatter;
use super::types::{Command, Error};
use crate::rv32_i::{Instruction, PseudoInstruction, StateDelta};

#[cfg(feature = "repl")]
/// Simple history manager for collecting StateDelta records
pub struct HistoryManager {
    deltas: Vec<StateDelta>,
    current_position: usize,
    limit: usize,
}

#[cfg(feature = "repl")]
impl HistoryManager {
    pub fn new() -> Self {
        Self {
            deltas: Vec::new(),
            current_position: 0,
            limit: 1000, // Default limit
        }
    }

    pub fn with_limit(limit: usize) -> Self {
        Self {
            deltas: Vec::new(),
            current_position: 0,
            limit,
        }
    }

    pub fn record_delta(&mut self, delta: StateDelta) {
        // Don't record anything if limit is 0
        if self.limit == 0 {
            return;
        }

        // If we're in the middle of history, truncate everything after current position
        self.deltas.truncate(self.current_position);

        // Add the new delta
        self.deltas.push(delta);
        self.current_position = self.deltas.len();

        // Enforce the limit by removing oldest entries
        while self.deltas.len() > self.limit {
            self.deltas.remove(0);
            self.current_position = self.current_position.saturating_sub(1);
        }
    }

    pub fn get_previous_delta(&mut self) -> Option<&StateDelta> {
        if self.current_position > 0 {
            self.current_position -= 1;
            self.deltas.get(self.current_position)
        } else {
            None
        }
    }

    pub fn get_next_delta(&mut self) -> Option<&StateDelta> {
        if self.current_position < self.deltas.len() {
            let delta = self.deltas.get(self.current_position);
            self.current_position += 1;
            delta
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.deltas.clear();
        self.current_position = 0;
    }
}

/// Executes a command and returns the result as a string
///
/// # Command Types
///
/// - **Exec**: Execute a hardware instruction
/// - **ExecPseudo**: Execute a pseudo-instruction
/// - **Inspect**: Show a register's value
/// - **ShowRegs**: Display all registers
/// - **ShowSpecificRegs**: Display specific registers
/// - **ShowHelp**: Display help information
/// - **Undo/Redo**: Navigate command history (REPL only)
pub fn run_command(
    input: Command,
    interpreter: &mut crate::interpreter::Interpreter,
) -> Result<String, Error> {
    match input {
        Command::Exec(instruction) => execute_with_tracking(instruction, None, interpreter),
        Command::ExecPseudo(pseudo) => execute_pseudo(pseudo, interpreter),
        Command::ShowRegs => Ok(formatter::format_all_registers(interpreter.cpu())),
        Command::ShowSpecificRegs(regs) => Ok(formatter::format_specific_registers(
            interpreter.cpu(),
            regs,
        )),
        Command::ShowHelp => Ok(formatter::format_help()),
        #[cfg(feature = "repl")]
        Command::Previous => handle_previous(interpreter),
        #[cfg(feature = "repl")]
        Command::Next => handle_next(interpreter),
        #[cfg(feature = "repl")]
        Command::Reset => handle_reset(interpreter),
        #[cfg(feature = "repl")]
        Command::ShowMemory { start, end } => {
            Ok(formatter::format_memory(interpreter.cpu(), start, end))
        }
    }
}

/// Executes an instruction with delta tracking for undo/redo
///
/// # State Tracking
///
/// When the REPL feature is enabled, this function captures the StateDelta
/// returned by CPU execution, which includes all state changes.
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

    // Record the delta in history if REPL is enabled
    #[cfg(feature = "repl")]
    if let Some(history) = interpreter.history_mut() {
        history.record_delta(delta.clone());
    }

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
        Ok(format!("Pseudo-instruction {} expanded to {}: {}", 
                   pseudo_name, instruction_names[0], results[0]))
    } else {
        Ok(format!(
            "Pseudo-instruction {} expanded to [{}]: {}",
            pseudo_name,
            instruction_names.join(", "),
            results.join("; ")
        ))
    }
}

/// Handles the /previous command
///
/// # Previous Operation
///
/// Navigates to the previous state in the execution history:
/// - Restores all register values
/// - Restores program counter
/// - Reverts memory modifications
/// - Reverts CSR changes
#[cfg(feature = "repl")]
fn handle_previous(interpreter: &mut crate::interpreter::Interpreter) -> Result<String, Error> {
    if let Some(history) = interpreter.history_mut() {
        if let Some(delta) = history.get_previous_delta() {
            // Create a reverse modify to undo the delta
            let undo_modify = delta.to_reverse_modify();

            // Apply the undo
            let _undo_delta = interpreter.cpu_mut().apply(&undo_modify)?;

            return Ok("Undid previous instruction".to_string());
        }
    }

    Err(Error::Generic("No previous state in history".to_string()))
}

/// Handles the /next command
///
/// # Next Operation
///
/// Navigates to the next state in the execution history by re-applying
/// a previously undone delta.
#[cfg(feature = "repl")]
fn handle_next(interpreter: &mut crate::interpreter::Interpreter) -> Result<String, Error> {
    if let Some(history) = interpreter.history_mut() {
        if let Some(delta) = history.get_next_delta() {
            // Create a forward modify to redo the delta
            let redo_modify = delta.to_forward_modify();

            // Apply the redo
            let _redo_delta = interpreter.cpu_mut().apply(&redo_modify)?;

            return Ok("Redid next instruction".to_string());
        }
    }

    Err(Error::Generic("No next state in history".to_string()))
}

/// Handles the /reset command
///
/// # Reset Operation
///
/// Prompts for confirmation then resets the entire CPU state:
/// - All registers to 0
/// - Program counter to 0
/// - Memory cleared
/// - History cleared
#[cfg(feature = "repl")]
fn handle_reset(interpreter: &mut crate::interpreter::Interpreter) -> Result<String, Error> {
    use std::io::{self, Write};

    // Print confirmation prompt
    print!("Reset CPU? This will clear all registers, memory, and history. (y/N): ");
    io::stdout()
        .flush()
        .map_err(|e| Error::Generic(format!("Failed to flush stdout: {e}")))?;

    // Read user input
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| Error::Generic(format!("Failed to read input: {e}")))?;

    // Check if user confirmed
    let confirmed = input.trim().eq_ignore_ascii_case("y");

    if confirmed {
        // Reset CPU state
        interpreter.cpu_mut().reset();

        // Clear history
        if let Some(history) = interpreter.history_mut() {
            history.clear();
        }

        Ok("CPU state reset".to_string())
    } else {
        Ok("Reset cancelled".to_string())
    }
}
