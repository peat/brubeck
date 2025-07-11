//! The interpreter takes input, parses it, and executes it in the [CPU](crate::rv32_i::CPU)
//!
//! The interpreter supports standard RISC-V assembly syntax for the RV32I instruction
//! set, including common pseudo-instructions. It can parse register names (both
//! x0-x31 and ABI names), immediate values in decimal/hex/binary formats, and
//! execute instructions or inspect register state.
//!
//! ## Examples
//!
//! ```
//! use brubeck::{Interpreter, parse};
//! use brubeck::rv32_i::Register;
//!
//! let mut i = Interpreter::new();
//!
//! // Parse and execute an ADDI instruction that sets register x1 to 3
//! let instructions = parse("ADDI x1, zero, 3").unwrap();
//! let delta = i.execute(instructions[0]).unwrap();
//! assert_eq!(delta.register_changes.len(), 1);
//!
//! // Check register x1 value using public cpu field
//! assert_eq!(i.cpu.get_register(Register::X1), 3);
//!
//! // Navigate to previous state
//! let undo_delta = i.previous_state().unwrap();
//! assert_eq!(i.cpu.get_register(Register::X1), 0);
//! ```

use crate::rv32_i::{Instruction, StateDelta, CPU};

// Internal interpreter modules
mod builder;
mod errors;
mod executor;
mod fuzzy;
mod parser;
mod state_history;
mod types;
mod validator;

use state_history::StateHistory;

// Re-export types for public API
pub use errors::{ExecutionError, HistoryError, ParseError};

// Internal use only - not part of public API
use types::{Command, Error};

/// Parse an assembly instruction string into a list of executable instructions
///
/// This is a static method that handles both regular instructions and pseudo-instructions,
/// expanding pseudo-instructions into their constituent real instructions.
///
/// # Examples
/// ```
/// use brubeck::interpreter::parse;
///
/// // Regular instruction
/// let instructions = parse("ADDI x1, x0, 42").unwrap();
/// assert_eq!(instructions.len(), 1);
///
/// // Pseudo-instruction that expands to one instruction
/// let instructions = parse("MV x1, x2").unwrap();
/// assert_eq!(instructions.len(), 1); // MV -> ADDI x1, x2, 0
///
/// // Pseudo-instruction that may expand to multiple instructions
/// let instructions = parse("LI x1, 0x12345").unwrap();
/// // LI can expand to LUI + ADDI for large values
/// ```
pub fn parse(input: &str) -> Result<Vec<crate::rv32_i::Instruction>, ParseError> {
    // Use the existing parser to get a Command
    let command = match parser::parse(input) {
        Ok(cmd) => cmd,
        Err(Error::UnknownInstruction {
            instruction,
            suggestion,
        }) => {
            return Err(ParseError::UnknownInstruction {
                instruction,
                suggestion,
            });
        }
        Err(Error::InvalidRegister { register, .. }) => {
            return Err(ParseError::InvalidRegister { register });
        }
        Err(Error::WrongArgumentCount {
            instruction,
            expected,
            found,
        }) => {
            // Parse expected count from string (e.g. "3 arguments" -> 3)
            let expected_count = expected
                .chars()
                .take_while(|c| c.is_numeric())
                .collect::<String>()
                .parse()
                .unwrap_or(0);
            return Err(ParseError::WrongArgumentCount {
                instruction,
                expected: expected_count,
                found,
            });
        }
        Err(Error::ImmediateOutOfRange {
            instruction,
            value,
            range,
        }) => {
            // Parse range string to get min/max values
            // Format is typically "-2048 to 2047"
            let parts: Vec<&str> = range.split(" to ").collect();
            let min = parts
                .first()
                .and_then(|s| s.parse().ok())
                .unwrap_or(i32::MIN);
            let max = parts
                .get(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(i32::MAX);
            return Err(ParseError::ImmediateOutOfRange {
                instruction,
                value,
                min,
                max,
            });
        }
        Err(Error::Generic(msg)) | Err(Error::UnrecognizedToken(msg)) => {
            return Err(ParseError::SyntaxError { message: msg });
        }
        Err(Error::Cpu(_)) => {
            // CPU errors shouldn't happen during parsing
            return Err(ParseError::SyntaxError {
                message: "Unexpected CPU error during parsing".to_string(),
            });
        }
    };

    // Convert Command to Vec<Instruction>
    match command {
        Command::Exec(instruction) => Ok(vec![instruction]),
        Command::ExecPseudo(pseudo) => {
            // Expand pseudo-instruction
            pseudo.expand().map_err(|e| ParseError::SyntaxError {
                message: format!("Failed to expand pseudo-instruction: {e}"),
            })
        }
    }
}

pub struct Interpreter {
    pub cpu: CPU,
    history: StateHistory,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    /// Creates a new Interpreter with 1 mebibyte of memory.
    pub fn new() -> Self {
        Self {
            cpu: CPU::default(),              // initializes with 1 mebibyte of memory
            history: StateHistory::new(1000), // Default history limit of 1000
        }
    }

    /// Creates a new Interpreter with custom memory size and history limit.
    pub fn with_config(memory_size: usize, history_limit: usize) -> Self {
        Self {
            cpu: CPU::new(memory_size),
            history: StateHistory::new(history_limit),
        }
    }

    /// Interprets a single command (no semicolons)
    pub fn interpret(&mut self, input: &str) -> Result<String, Error> {
        let command = parser::parse(input)?;
        let delta = executor::run_command(command, self)?;

        // Format the state delta for display
        let mut changes = Vec::new();

        // Format register changes (show the most important ones)
        let mut has_pc_change = false;
        for (reg, old, new) in &delta.register_changes {
            if *reg == crate::rv32_i::Register::PC {
                changes.push(format!("PC: 0x{:08x} → 0x{:08x}", old, new));
                has_pc_change = true;
            } else {
                changes.push(format!("{:?}: {} → {}", reg, *old as i32, *new as i32));
            }
        }
        
        // Always show PC change from delta
        if !has_pc_change && delta.pc_change.0 != delta.pc_change.1 {
            changes.push(format!("PC: 0x{:08x} → 0x{:08x}", delta.pc_change.0, delta.pc_change.1));
        }

        // Show memory changes summary
        if !delta.memory_changes.is_empty() {
            changes.push(format!("{} memory bytes changed", delta.memory_changes.len()));
        }

        // Show CSR changes
        for (csr, old, new) in &delta.csr_changes {
            changes.push(format!("CSR[0x{:03x}]: 0x{:08x} → 0x{:08x}", csr, old, new));
        }

        if changes.is_empty() {
            Ok("No state changes".to_string())
        } else {
            Ok(changes.join(", "))
        }
    }

    /// Executes an [Instruction] directly, returning the state changes
    pub fn execute(
        &mut self,
        instruction: Instruction,
    ) -> Result<StateDelta, crate::rv32_i::CPUError> {
        // Execute the instruction and get the delta
        let delta = self.cpu.execute(instruction)?;

        // Record the delta in history
        self.history.record_delta(delta.clone());

        // Return the delta
        Ok(delta)
    }

    /// Navigate to the previous state in history, returning the applied delta
    pub fn previous_state(&mut self) -> Result<StateDelta, HistoryError> {
        if let Some(delta) = self.history.get_previous_delta() {
            // Create a reverse modify to undo the delta
            let undo_modify = delta.to_reverse_modify();

            // Apply the undo
            let undo_delta = self
                .cpu
                .apply(&undo_modify)
                .map_err(|_| HistoryError::AtBeginning)?; // Should never fail

            Ok(undo_delta)
        } else {
            Err(HistoryError::AtBeginning)
        }
    }

    /// Navigate to the next state in history, returning the applied delta
    pub fn next_state(&mut self) -> Result<StateDelta, HistoryError> {
        if let Some(delta) = self.history.get_next_delta() {
            // Create a forward modify to redo the delta
            let redo_modify = delta.to_forward_modify();

            // Apply the redo
            let redo_delta = self
                .cpu
                .apply(&redo_modify)
                .map_err(|_| HistoryError::AtEnd)?; // Should never fail

            Ok(redo_delta)
        } else {
            Err(HistoryError::AtEnd)
        }
    }

    /// Reset the interpreter to its initial state
    ///
    /// This clears the CPU state and history while maintaining configuration settings
    /// like memory size and history limit.
    pub fn reset(&mut self) {
        // Reset CPU to initial state
        self.cpu.reset();

        // Clear history
        self.history.clear();
    }
}
