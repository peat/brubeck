//! Unit tests for upper immediate instructions (LUI, AUIPC)
//!
//! These tests verify the loading of 20-bit immediates into the upper
//! portion of registers and PC-relative address calculations.
//!
//! Reference: RISC-V ISA Manual, Volume I: Unprivileged ISA, Version 20191213
//! Section 2.4 - Integer Computational Instructions
//!
//! Key concepts:
//! - LUI: Load Upper Immediate - loads 20-bit value into bits [31:12]
//! - AUIPC: Add Upper Immediate to PC - for PC-relative addressing
//! - Lower 12 bits are always zeroed by these instructions
//! - Used together with other instructions to build full 32-bit constants
//!
//! Common patterns:
//! ```
//! # Load 32-bit constant:
//! LUI x1, %hi(value)      # Upper 20 bits
//! ADDI x1, x1, %lo(value) # Lower 12 bits
//!
//! # PC-relative addressing:
//! AUIPC x1, %pcrel_hi(symbol)
//! ADDI x1, x1, %pcrel_lo(symbol)
//! ```

use brubeck::rv32_i::{formats::UType, instructions::Instruction, registers::Register};

// Import test helpers
use crate::unit::test_helpers::{values, CpuAssertions, CpuBuilder, ExecuteWithContext};

#[test]
fn test_lui_basic() {
    // LUI: rd = immediate << 12
    // Places 20-bit immediate in upper portion, zeros lower 12 bits
    let mut cpu = CpuBuilder::new().build();

    let mut inst = UType {
        rd: Register::X1,
        ..Default::default()
    };
    inst.imm.set_unsigned(1).unwrap();

    let lui = Instruction::LUI(inst);
    cpu.execute_expect(lui, "LUI with immediate=1");

    cpu.assert_register(
        Register::X1,
        0x1000,
        "LUI places immediate in bits [31:12], zeros [11:0]",
    );
}

#[test]
fn test_lui_edge_cases() {
    // Test boundary values for 20-bit immediate
    let mut cpu = CpuBuilder::new().build();

    let mut inst = UType {
        rd: Register::X1,
        ..Default::default()
    };

    // Case 1: Maximum 20-bit value (all ones)
    inst.imm.set_unsigned(values::IMM20_MAX).unwrap();
    let lui = Instruction::LUI(inst);
    cpu.execute_expect(lui, "LUI with max immediate");
    cpu.assert_register(
        Register::X1,
        0xFFFFF000,
        "Max immediate fills upper 20 bits, lower 12 are zero",
    );

    // Case 2: Zero value (common for clearing)
    inst.imm.set_unsigned(0).unwrap();
    let lui = Instruction::LUI(inst);
    cpu.execute_expect(lui, "LUI with zero");
    cpu.assert_register(Register::X1, 0, "LUI with 0 produces 0");

    // Case 3: Sign bit pattern (0x80000)
    inst.imm.set_unsigned(0x80000).unwrap();
    let lui = Instruction::LUI(inst);
    cpu.execute_expect(lui, "LUI setting sign bit");
    cpu.assert_register(Register::X1, 0x80000000, "LUI can set sign bit");
}

#[test]
fn test_auipc_basic() {
    // AUIPC: rd = PC + (immediate << 12)
    // Used for PC-relative addressing in position-independent code
    let mut cpu = CpuBuilder::new().with_pc(0).build();

    let mut inst = UType {
        rd: Register::X1,
        ..Default::default()
    };
    inst.imm.set_unsigned(1).unwrap();

    let auipc = Instruction::AUIPC(inst);
    cpu.execute_expect(auipc, "AUIPC from PC=0");

    cpu.assert_register(Register::X1, 0x1000, "AUIPC: PC(0) + (1 << 12) = 0x1000");
}

#[test]
fn test_auipc_with_pc() {
    // AUIPC with non-zero PC - common in real programs
    let mut cpu = CpuBuilder::new().with_pc(0x1000).build();

    let mut inst = UType {
        rd: Register::X1,
        ..Default::default()
    };
    inst.imm.set_unsigned(0x12345).unwrap();

    let auipc = Instruction::AUIPC(inst);
    cpu.execute_expect(auipc, "AUIPC with PC=0x1000");

    let expected = 0x1000 + (0x12345 << 12);
    cpu.assert_register(
        Register::X1,
        expected,
        "AUIPC adds shifted immediate to current PC",
    );
}

#[test]
fn test_auipc_pc_relative_addressing() {
    // Demonstrates AUIPC+ADDI pattern for full 32-bit PC-relative addressing
    // Example: Loading address of data that's 0x12345 bytes from current PC
    let mut cpu = CpuBuilder::new().with_pc(0x10000).build();

    // Step 1: AUIPC loads upper 20 bits of offset
    // For offset 0x12345:
    // - Upper 20 bits: 0x12 (but we need to adjust for sign extension)
    // - Lower 12 bits: 0x345
    // Since 0x345 is positive, we use 0x12 directly
    let mut inst = UType {
        rd: Register::X1,
        ..Default::default()
    };
    inst.imm.set_unsigned(0x12).unwrap();

    let auipc = Instruction::AUIPC(inst);
    cpu.execute_expect(auipc, "AUIPC for PC-relative addressing");

    cpu.assert_register(
        Register::X1,
        0x10000 + 0x12000,
        "AUIPC loads PC + upper bits of target offset",
    );

    // Step 2: ADDI would add the lower 12 bits (0x345) to complete the address
    // Final address would be: 0x10000 + 0x12000 + 0x345 = 0x22345
}

#[test]
fn test_lui_auipc_x0_destination() {
    // Writing to x0 is ignored, but instruction still executes
    // This can be used as a multi-byte NOP
    let mut cpu = CpuBuilder::new().with_pc(0).build();

    let mut inst = UType {
        rd: Register::X0,
        ..Default::default()
    };
    inst.imm.set_unsigned(0x12345).unwrap();

    // Test LUI to x0
    let lui = Instruction::LUI(inst);
    cpu.execute_expect(lui, "LUI to x0");
    cpu.assert_register(Register::X0, 0, "x0 remains zero");
    cpu.assert_pc(Instruction::LENGTH, "PC advances normally");

    // Test AUIPC to x0
    let auipc = Instruction::AUIPC(inst);
    cpu.execute_expect(auipc, "AUIPC to x0");
    cpu.assert_register(Register::X0, 0, "x0 remains zero");
    cpu.assert_pc(2 * Instruction::LENGTH, "PC advances normally");
}
