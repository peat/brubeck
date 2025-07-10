//! Implementation of the RISC-V 32bit Integer Base (RV32I) ISA
//!
//! This module provides a complete implementation of the RV32I base integer
//! instruction set, including:
//!
//! - The [CPU] emulator with registers and memory
//! - All 47 RV32I [instructions](Instruction)
//! - Instruction encoding [formats](formats) (R, I, S, B, U, J types)
//! - [Register](Register) definitions with ABI names
//! - Common [pseudo-instructions](pseudo_instructions) that expand to RV32I instructions

pub mod cpu;
pub mod formats;
pub mod instructions;
pub mod pseudo_instructions;
pub mod registers;

pub use cpu::*;
pub use formats::*;
pub use instructions::*;
pub use pseudo_instructions::*;
pub use registers::*;

