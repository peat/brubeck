//! Unit tests for load and store instructions
//!
//! Tests memory access patterns, alignment, and different widths.

use brubeck::rv32_i::cpu::CPU;

// TODO: Migrate load/store tests from src/rv32_i/mod.rs
// Including: LW, LH, LHU, LB, LBU, SW, SH, SB

#[test]
fn test_loads_stores_placeholder() {
    let cpu = CPU::default();
    assert_eq!(cpu.pc, 0);
}