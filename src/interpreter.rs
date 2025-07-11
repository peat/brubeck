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
//! use brubeck::interpreter::Interpreter;
//! use brubeck::rv32_i::Register;
//!
//! let mut i = Interpreter::new();
//!
//! // Execute an ADDI instruction that sets register x1 to 3
//! let output = i.interpret("ADDI x1, zero, 3");
//! assert!(output.is_ok());
//!
//! // Check register x1 value
//! assert_eq!(i.cpu().get_register(Register::X1), 3);
//!
//! // Navigate to previous state
//! let result = i.previous_state();
//! assert!(result.is_ok());
//! assert_eq!(i.cpu().get_register(Register::X1), 0);
//! ```

use crate::rv32_i::{Instruction, StateDelta, CPU};

/// Simple state history for navigation
pub(crate) struct StateHistory {
    deltas: Vec<StateDelta>,
    current_position: usize,
    limit: usize,
}

impl StateHistory {
    fn new(limit: usize) -> Self {
        Self {
            deltas: Vec::new(),
            current_position: 0,
            limit,
        }
    }

    pub(crate) fn record_delta(&mut self, delta: StateDelta) {
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

    fn get_previous_delta(&mut self) -> Option<&StateDelta> {
        if self.current_position > 0 {
            self.current_position -= 1;
            self.deltas.get(self.current_position)
        } else {
            None
        }
    }

    fn get_next_delta(&mut self) -> Option<&StateDelta> {
        if self.current_position < self.deltas.len() {
            let delta = self.deltas.get(self.current_position);
            self.current_position += 1;
            delta
        } else {
            None
        }
    }

    pub(crate) fn clear(&mut self) {
        self.deltas.clear();
        self.current_position = 0;
    }
}

// Internal interpreter modules
#[path = "interpreter/builder.rs"]
mod builder;
#[path = "interpreter/executor.rs"]
mod executor;
#[path = "interpreter/formatter.rs"]
mod formatter;
#[path = "interpreter/parser.rs"]
mod parser;
#[path = "interpreter/types.rs"]
mod types;
#[path = "interpreter/validator.rs"]
mod validator;

// Re-export types for public API
pub use types::{Command, Error, Token};

pub struct Interpreter {
    cpu: CPU,
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

    /// Interprets a single command, which could be an instruction (eg: `ADDI x1, zero, 3`) or an
    /// inspection for registers (eg: `PC` or `X1`). Returns a String representation of the
    /// result or an Error.
    ///
    /// Supports semicolon-separated commands for batch execution.
    pub fn interpret(&mut self, input: &str) -> Result<String, Error> {
        // Library no longer supports semicolon-separated commands
        // This functionality has been moved to the binary
        if input.contains(';') {
            return Err(Error::Generic(
                "Semicolon-separated commands are not supported in the library. Use the binary for batch execution.".to_string()
            ));
        }
        self.interpret_single(input)
    }

    /// Interprets a single command (no semicolons)
    fn interpret_single(&mut self, input: &str) -> Result<String, Error> {
        let command = parser::parse(input)?;
        executor::run_command(command, self)
    }

    /// Executes an [Instruction] directly, skipping the parsing steps.
    pub fn execute(&mut self, instruction: Instruction) -> Result<String, Error> {
        executor::execute_with_tracking(instruction, None, self)
    }

    /// Gets the current program counter value
    pub fn get_pc(&self) -> u32 {
        self.cpu.pc
    }

    /// Gets a reference to the CPU for inspection
    pub fn cpu(&self) -> &CPU {
        &self.cpu
    }

    /// Gets a mutable reference to the CPU
    pub fn cpu_mut(&mut self) -> &mut CPU {
        &mut self.cpu
    }

    /// Gets a mutable reference to the history (internal use)
    pub(crate) fn history_mut(&mut self) -> &mut StateHistory {
        &mut self.history
    }

    /// Navigate to the previous state in history
    pub fn previous_state(&mut self) -> Result<String, Error> {
        if let Some(delta) = self.history.get_previous_delta() {
            // Create a reverse modify to undo the delta
            let undo_modify = delta.to_reverse_modify();

            // Apply the undo
            let _undo_delta = self.cpu.apply(&undo_modify)?;

            Ok("Undid previous instruction".to_string())
        } else {
            Err(Error::Generic("No previous state in history".to_string()))
        }
    }

    /// Navigate to the next state in history
    pub fn next_state(&mut self) -> Result<String, Error> {
        if let Some(delta) = self.history.get_next_delta() {
            // Create a forward modify to redo the delta
            let redo_modify = delta.to_forward_modify();

            // Apply the redo
            let _redo_delta = self.cpu.apply(&redo_modify)?;

            Ok("Redid next instruction".to_string())
        } else {
            Err(Error::Generic("No next state in history".to_string()))
        }
    }

    /// Clear the state history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

// Tests have been moved to tests/parser.rs and tests/unit/instructions/
