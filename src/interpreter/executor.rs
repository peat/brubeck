//! Execution module for running commands and instructions
//!
//! This module handles the execution of parsed commands, including
//! hardware instructions, pseudo-instructions, and REPL commands.

use super::formatter;
use super::types::{Command, Error};
use crate::rv32_i::{Instruction, PseudoInstruction};

#[cfg(feature = "repl")]
use crate::history::StateSnapshot;

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
        Command::Undo => handle_undo(interpreter),
        #[cfg(feature = "repl")]
        Command::Redo => handle_redo(interpreter),
    }
}

/// Executes an instruction with state tracking for undo/redo
///
/// # State Tracking
///
/// When the REPL feature is enabled, this function captures:
/// - Register state before and after execution
/// - Program counter changes
/// - Memory modifications
/// - CSR (Control and Status Register) changes
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
    // Capture state before execution (only if REPL feature is enabled)
    #[cfg(feature = "repl")]
    let (old_registers, old_pc, instruction_text) = {
        interpreter.cpu_mut().clear_tracking();
        let regs = interpreter.cpu().get_all_registers();
        let pc = interpreter.cpu().pc;
        // Use provided display name or generate one
        let text = display_name.unwrap_or_else(|| {
            // Use the mnemonic for the instruction
            instruction.mnemonic().to_string()
        });
        (regs, pc, text)
    };

    // Execute the instruction
    match interpreter.cpu_mut().execute(instruction) {
        Ok(()) => {
            // Capture state after successful execution
            #[cfg(feature = "repl")]
            {
                let new_registers = interpreter.cpu().get_all_registers();
                let new_pc = interpreter.cpu().pc;
                let snapshot = StateSnapshot {
                    instruction: instruction_text,
                    registers: old_registers,
                    pc: old_pc,
                    registers_after: new_registers,
                    pc_after: new_pc,
                    csr_changes: interpreter.cpu().csr_changes.clone(),
                    memory_changes: interpreter.cpu().memory_changes.clone(),
                };
                interpreter.history_mut().push(snapshot);
            }

            Ok(formatter::format_instruction_result(
                &instruction,
                interpreter.cpu(),
            ))
        }
        e => Err(Error::Generic(format!("{e:?}"))),
    }
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
    for inst in instructions {
        // Execute with the pseudo-instruction name for history
        match execute_with_tracking(inst, Some(pseudo_name.clone()), interpreter) {
            Ok(result) => results.push(result),
            Err(e) => return Err(e),
        }
    }

    if results.len() == 1 {
        Ok(format!("Pseudo-instruction: {}", results[0]))
    } else {
        Ok(format!(
            "Pseudo-instruction expanded to: {}",
            results.join("; ")
        ))
    }
}

/// Handles the /undo command
///
/// # Undo Operation
///
/// Restores the CPU state to before the last executed instruction:
/// - Restores all register values
/// - Restores program counter
/// - Reverts memory modifications
/// - Reverts CSR changes
#[cfg(feature = "repl")]
fn handle_undo(interpreter: &mut crate::interpreter::Interpreter) -> Result<String, Error> {
    // Clone the snapshot to avoid borrowing conflicts
    let snapshot = match interpreter.history_mut().undo() {
        Some(s) => s.clone(),
        None => return Err(Error::Generic("Nothing to undo".to_string())),
    };

    // Now we can mutably borrow the CPU without conflicts
    let cpu = interpreter.cpu_mut();

    // Restore CPU state
    cpu.set_all_registers(&snapshot.registers);
    cpu.pc = snapshot.pc;

    // Restore memory changes
    cpu.restore_memory(&snapshot.memory_changes);

    // Restore CSR changes
    cpu.restore_csrs(&snapshot.csr_changes);

    Ok(format!("Undid: {}", snapshot.instruction))
}

/// Handles the /redo command
///
/// # Redo Operation
///
/// Re-applies a previously undone instruction:
/// - Restores registers to the state after the instruction
/// - Updates program counter
/// - Re-applies memory modifications
/// - Re-applies CSR changes
#[cfg(feature = "repl")]
fn handle_redo(interpreter: &mut crate::interpreter::Interpreter) -> Result<String, Error> {
    // Clone the snapshot to avoid borrowing conflicts
    let snapshot = match interpreter.history_mut().redo() {
        Some(s) => s.clone(),
        None => return Err(Error::Generic("Nothing to redo".to_string())),
    };

    // Now we can mutably borrow the CPU without conflicts
    let cpu = interpreter.cpu_mut();

    // Restore to the state AFTER the instruction
    cpu.set_all_registers(&snapshot.registers_after);
    cpu.pc = snapshot.pc_after;

    // Apply the memory changes
    for delta in &snapshot.memory_changes {
        if (delta.address as usize) < cpu.memory.len() {
            cpu.memory[delta.address as usize] = delta.new_value;
        }
    }

    // Apply the CSR changes
    for &(addr, _old_val, new_val) in &snapshot.csr_changes {
        if cpu.csr_exists[addr as usize] && !cpu.csr_readonly[addr as usize] {
            cpu.csrs[addr as usize] = new_val;
        }
    }

    Ok(format!("Redid: {}", snapshot.instruction))
}
