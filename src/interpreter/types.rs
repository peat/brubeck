//! Common types used across the interpreter modules
//!
//! This module contains the shared types used by the parser, builder,
//! validator, formatter, and executor modules.

use crate::rv32_i::{Instruction, PseudoInstruction, Register};
use std::fmt::Display;

/// Represents a command that can be executed by the interpreter
#[derive(Debug, PartialEq)]
pub enum Command {
    /// Execute a hardware instruction
    Exec(Instruction),
    /// Execute a pseudo-instruction
    ExecPseudo(PseudoInstruction),
    /// Show all registers
    ShowRegs,
    /// Show specific registers
    ShowSpecificRegs(Vec<Register>),
    /// Show help information
    ShowHelp,
    /// Navigate to previous state in history (REPL feature)
    #[cfg(feature = "repl")]
    Previous,
    /// Navigate to next state in history (REPL feature)
    #[cfg(feature = "repl")]
    Next,
}

/// Represents a token in the parsed assembly
#[derive(Debug, PartialEq)]
pub enum Token {
    /// A register reference
    Register(Register),
    /// A hardware instruction
    Instruction(Instruction),
    /// A pseudo-instruction
    PseudoInstruction(PseudoInstruction),
    /// A 32-bit immediate value
    Value32(i32),
    /// An offset(register) notation for loads/stores
    OffsetRegister { offset: i32, register: Register },
}

/// Error types for the interpreter
#[derive(Debug)]
pub enum Error {
    /// Generic error with a message
    Generic(String),
    /// Unrecognized token during parsing
    UnrecognizedToken(String),
    /// Unknown instruction mnemonic
    UnknownInstruction {
        instruction: String,
        suggestion: Option<String>,
    },
    /// Invalid register name
    InvalidRegister { register: String, help: String },
    /// Wrong number of arguments for an instruction
    WrongArgumentCount {
        instruction: String,
        expected: String,
        found: usize,
    },
    /// Immediate value out of valid range
    ImmediateOutOfRange {
        instruction: String,
        value: i32,
        range: String,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err_string = match self {
            Self::Generic(s) => s.to_owned(),

            Self::UnrecognizedToken(s) => {
                format!("Unrecognized token: '{s}'\n💡 Tip: Check for typos in instruction names, register names, or number formats")
            },

            Self::UnknownInstruction { instruction, suggestion } => {
                match suggestion {
                    Some(s) => format!("Unknown instruction '{instruction}'. Did you mean '{s}'?\n💡 Tip: RISC-V instructions are case-insensitive. Use 'help' for a list of supported instructions"),
                    None => format!("Unknown instruction '{instruction}'\n💡 Tip: Check the RISC-V ISA manual or use 'help' for supported instructions"),
                }
            },

            Self::InvalidRegister { register, help } => {
                format!("Invalid register '{register}'. {help}\n💡 Tip: Valid registers are x0-x31, or ABI names like zero, ra, sp, gp, tp, t0-t6, s0-s11, a0-a7")
            },

            Self::WrongArgumentCount { instruction, expected, found } => {
                let tip = match instruction.as_str() {
                    "ADD" | "SUB" | "AND" | "OR" | "XOR" | "SLL" | "SLT" | "SLTU" | "SRA" | "SRL" => 
                        "💡 Tip: R-type instructions need 3 registers: rd, rs1, rs2 (e.g., ADD x1, x2, x3)",
                    "ADDI" | "ANDI" | "ORI" | "XORI" | "SLTI" | "SLTIU" | "SLLI" | "SRAI" | "SRLI" => 
                        "💡 Tip: I-type instructions need 2 registers + immediate: rd, rs1, imm (e.g., ADDI x1, x2, 100)",
                    "LW" | "LH" | "LB" | "LHU" | "LBU" => 
                        "💡 Tip: Load instructions: LW x1, offset(base) or LW x1, base, offset",
                    "SW" | "SH" | "SB" => 
                        "💡 Tip: Store instructions: SW rs2, offset(base) or SW rs2, base, offset",
                    "BEQ" | "BNE" | "BLT" | "BGE" | "BLTU" | "BGEU" => 
                        "💡 Tip: Branch instructions need 2 registers + offset: rs1, rs2, offset",
                    "LUI" | "AUIPC" => 
                        "💡 Tip: Upper immediate instructions need register + immediate: rd, imm",
                    "JAL" => 
                        "💡 Tip: JAL needs link register + offset: rd, offset",
                    "JALR" => 
                        "💡 Tip: JALR needs link register, base register + offset: rd, rs1, offset",
                    _ => "💡 Tip: Check the RISC-V ISA manual for the correct instruction format",
                };
                format!("{instruction} expects {expected}, but {found} {} provided\n{tip}",
                    if *found == 1 { "was" } else { "were" })
            },

            Self::ImmediateOutOfRange { instruction, value, range } => {
                let tip = match instruction.as_str() {
                    "ADDI" | "ANDI" | "ORI" | "XORI" | "SLTI" | "SLTIU" => 
                        "💡 Tip: I-type immediates are 12-bit signed values. For larger values, use LUI + ADDI pattern",
                    "LUI" | "AUIPC" => 
                        "💡 Tip: Upper immediate instructions use 20-bit values that become the upper 20 bits of the result",
                    "SLLI" | "SRAI" | "SRLI" => 
                        "💡 Tip: Shift amounts must be 0-31 since RISC-V registers are 32 bits",
                    "BEQ" | "BNE" | "BLT" | "BGE" | "BLTU" | "BGEU" => 
                        "💡 Tip: Branch offsets are 12-bit signed values and must be even (word-aligned)",
                    "JAL" => 
                        "💡 Tip: JAL offsets are 20-bit signed values and must be even (word-aligned)",
                    _ => "💡 Tip: Different instruction types have different immediate ranges - check the RISC-V ISA manual",
                };
                format!("Immediate value {value} out of range for {instruction} (valid range: {range})\n{tip}")
            },
        };

        write!(f, "{err_string}")
    }
}
