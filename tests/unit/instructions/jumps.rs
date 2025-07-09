//! Unit tests for jump instructions (JAL, JALR)
//!
//! Tests unconditional jumps and link register behavior.

use brubeck::rv32_i::cpu::CPU;

// TODO: Migrate jump tests from src/rv32_i/mod.rs

#[test]
fn test_jumps_placeholder() {
    let cpu = CPU::default();
    assert_eq!(cpu.pc, 0);
}