//! Unit tests for RISC-V instructions
//! 
//! Tests are organized by instruction category to match the RISC-V ISA manual structure.
//! Each test file focuses on related instructions and their edge cases.

mod arithmetic;
mod comparison;
mod logical;
mod shifts;
mod loads_stores;
mod branches;
mod jumps;
mod upper_immediate;
mod misc;