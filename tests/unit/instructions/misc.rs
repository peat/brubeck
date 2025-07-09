//! Unit tests for miscellaneous instructions (NOP, FENCE, EBREAK, ECALL)
//!
//! These tests verify special-purpose instructions including
//! the NOP instruction and system instructions.
//!
//! Reference: RISC-V ISA Manual, Volume I: Unprivileged ISA, Version 20191213
//! Section 2.4 - Integer Computational Instructions (NOP)
//! Section 2.7 - Memory Model (FENCE)
//! Section 2.8 - Control and Status Register Instructions (EBREAK, ECALL)
//!
//! Key concepts:
//! - NOP: No Operation - advances PC without changing state
//! - NOP is a pseudo-instruction: ADDI x0, x0, 0
//! - Used for alignment, timing, and placeholder instructions
//!
//! Common uses of NOP:
//! - Instruction alignment in branch delay slots (other architectures)
//! - Code patching placeholders
//! - Timing adjustments in tight loops
//! - Debugging breakpoints (replaced at runtime)

use brubeck::rv32_i::{instructions::Instruction, registers::Register};

// Import test helpers
use crate::unit::test_helpers::{CpuAssertions, CpuBuilder, ExecuteWithContext};

#[test]
fn test_nop() {
    // NOP: No Operation
    // Only effect is PC += 4
    // Encoded as ADDI x0, x0, 0 (write to x0 is ignored)
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, 0x12345678) // Set some state
        .with_register(Register::X2, 0xABCDEF00) // to verify it's preserved
        .build();

    let nop = Instruction::NOP;

    // Save initial register state
    let x1_before = cpu.get_register(Register::X1);
    let x2_before = cpu.get_register(Register::X2);

    // Execute first NOP
    cpu.execute_expect(nop, "First NOP");
    cpu.assert_pc(4, "NOP advances PC by 4");

    // Verify registers unchanged
    cpu.assert_register(Register::X1, x1_before, "X1 unchanged by NOP");
    cpu.assert_register(Register::X2, x2_before, "X2 unchanged by NOP");
    cpu.assert_register(Register::X0, 0, "X0 remains zero");

    // Execute second NOP to verify PC continues advancing
    cpu.execute_expect(nop, "Second NOP");
    cpu.assert_pc(8, "Second NOP advances PC to 8");

    // Registers still unchanged
    cpu.assert_register(Register::X1, x1_before, "X1 still unchanged");
    cpu.assert_register(Register::X2, x2_before, "X2 still unchanged");
}

#[test]
fn test_nop_use_cases() {
    // Demonstrate practical uses of NOP instructions
    let mut cpu = CpuBuilder::new()
        .with_pc(0x1000) // Start at a specific address
        .build();

    let nop = Instruction::NOP;

    // Use case 1: Alignment padding
    // Sometimes NOPs are used to align branch targets to cache lines
    // or to ensure specific instruction addresses
    cpu.execute_expect(nop, "Alignment NOP 1");
    cpu.assert_pc(0x1004, "First NOP for alignment");

    cpu.execute_expect(nop, "Alignment NOP 2");
    cpu.assert_pc(0x1008, "Second NOP for alignment");

    // Now we're at a nicely aligned address (0x1008)
    // which might be beneficial for branch targets

    // Use case 2: Multiple NOPs in sequence
    // Used in delay slots or for timing
    let start_pc = cpu.pc;
    for i in 0..4 {
        cpu.execute_expect(nop, &format!("Timing NOP {}", i));
    }
    cpu.assert_pc(start_pc + 16, "Four NOPs advance PC by 16");

    // Use case 3: NOP as placeholder
    // During development, NOPs can be placeholders for future instructions
    // They can be patched at runtime without changing code size

    // Verify NOP is truly "no operation"
    match nop {
        Instruction::NOP => {
            // In RISC-V, NOP is a specific instruction variant
            // It's guaranteed to have no architectural effect except PC += 4
        }
        _ => panic!("NOP should be NOP variant"),
    }
}

// Note: FENCE, EBREAK, and ECALL are not yet implemented in this emulator
// - FENCE: Memory ordering instruction
// - EBREAK: Breakpoint/debugger trap
// - ECALL: System call/environment call
