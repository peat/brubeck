//! Unit tests for RISC-V instructions
//!
//! Tests are organized by instruction category to match the RISC-V ISA manual structure.
//! Each test file focuses on related instructions and their edge cases.

mod arithmetic;
mod branches;
mod comparison;
mod csr;
mod jumps;
mod loads_stores;
mod logical;
mod misc;
mod shifts;
mod system;
mod upper_immediate;
