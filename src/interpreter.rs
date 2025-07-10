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
//!
//! let mut i = Interpreter::new();
//!
//! // Execute an ADDI instruction that sets register x1 to 3
//! let output = i.interpret("ADDI x1, zero, 3");
//! assert!(output.is_ok());
//!
//! // Inspect register x1 using the /regs command
//! let output = i.interpret("/regs x1");
//! assert!(output.unwrap().contains("x 1 (ra  ): 0x00000003"));
//!
//! // Show all registers
//! let output = i.interpret("/regs");
//! assert!(output.is_ok());
//! ```

use crate::rv32_i::{Instruction, CPU};

#[cfg(feature = "repl")]
use crate::history::HistoryManager;

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
    #[cfg(feature = "repl")]
    history: HistoryManager,
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
            cpu: CPU::default(), // initializes with 1 mebibyte of memory
            #[cfg(feature = "repl")]
            history: HistoryManager::new(1000), // Default history size
        }
    }

    /// Creates a new Interpreter with custom configuration
    #[cfg(feature = "repl")]
    pub fn with_config(config: crate::cli::Config) -> Self {
        Self {
            cpu: CPU::new(config.memory_size),
            history: HistoryManager::new(config.undo_limit),
        }
    }

    /// Returns the configured memory size
    #[cfg(feature = "repl")]
    pub fn memory_size(&self) -> usize {
        self.cpu.memory.len()
    }

    /// Interprets a single command, which could be an instruction (eg: `ADDI x1, zero, 3`) or an
    /// inspection for registers (eg: `PC` or `X1`). Returns a String representation of the
    /// result or an Error.
    ///
    /// Supports semicolon-separated commands for batch execution.
    pub fn interpret(&mut self, input: &str) -> Result<String, Error> {
        // Check if input contains semicolons
        if input.contains(';') {
            // Split by semicolons and execute each command
            let mut results = Vec::new();

            #[cfg(feature = "repl")]
            let commands = crate::cli::split_commands(input);
            #[cfg(not(feature = "repl"))]
            let commands: Vec<&str> = input
                .split(';')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();

            for command in commands {
                match self.interpret_single(command) {
                    Ok(result) => results.push(result),
                    Err(e) => return Err(e), // Stop on first error
                }
            }

            Ok(results.join("\n"))
        } else {
            self.interpret_single(input)
        }
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

    /// Gets a reference to the CPU for the executor
    pub(crate) fn cpu(&self) -> &CPU {
        &self.cpu
    }

    /// Gets a mutable reference to the CPU for the executor
    pub(crate) fn cpu_mut(&mut self) -> &mut CPU {
        &mut self.cpu
    }

    /// Gets a mutable reference to the history manager
    #[cfg(feature = "repl")]
    pub(crate) fn history_mut(&mut self) -> &mut HistoryManager {
        &mut self.history
    }
}

// Tests have been moved to tests/parser.rs and tests/unit/instructions/
