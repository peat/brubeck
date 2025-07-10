//! Unit tests for comparison instructions (SLT, SLTI, SLTU, SLTIU)
//!
//! These tests verify signed and unsigned comparison behavior, including
//! edge cases around sign extension and boundary values.
//!
//! Reference: RISC-V ISA Manual, Volume I: Unprivileged ISA, Version 20191213
//! Section 2.4 - Integer Computational Instructions
//!
//! Key concepts:
//! - SLT/SLTI: Set Less Than (signed) - uses two's complement comparison
//! - SLTU/SLTIU: Set Less Than Unsigned - treats all values as unsigned
//! - Result is always 1 (true) or 0 (false)
//! - Immediate versions sign-extend the 12-bit immediate to 32 bits
//!
//! Critical difference - sign interpretation:
//! ```
//! Value     | Signed    | Unsigned
//! ----------|-----------|----------
//! 0xFFFFFFFF| -1        | 4294967295
//! 0x80000000| -2147483648| 2147483648
//! 0x7FFFFFFF| 2147483647| 2147483647
//! 0x00000001| 1         | 1
//! ```
//!
//! Common patterns:
//! - SLTI rd, rs1, 1: Check if rs1 < 1 (i.e., rs1 <= 0)
//! - SLTIU rd, rs1, 1: Check if rs1 == 0 (only 0 < 1 unsigned)

use brubeck::rv32_i::{formats::IType, instructions::Instruction, registers::Register};

// Import test helpers
use crate::unit::test_helpers::{values, CpuAssertions, CpuBuilder, ExecuteWithContext};

#[test]
fn test_slti_signed_comparison() {
    // SLTI: Set Less Than Immediate (signed)
    // rd = (rs1_signed < immediate_signed) ? 1 : 0
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X2, 0) // Start with zero
        .build();

    let mut inst = IType {
        rd: Register::X1,
        rs1: Register::X2,
        ..Default::default()
    };
    inst.imm.set_unsigned(0).unwrap();

    let slti = Instruction::SLTI(inst);

    // Test equal values (0 == 0)
    cpu.execute_expect(slti, "SLTI: 0 < 0");
    cpu.assert_register(Register::X1, 0, "Equal values return false");

    // Test rs1 < immediate (0 < 1)
    inst.imm.set_signed(1).unwrap();
    let slti = Instruction::SLTI(inst);
    cpu.execute_expect(slti, "SLTI: 0 < 1");
    cpu.assert_register(Register::X1, 1, "0 is less than 1");

    // Test rs1 > immediate (0 > -1)
    inst.imm.set_signed(-1).unwrap();
    let slti = Instruction::SLTI(inst);
    cpu.execute_expect(slti, "SLTI: 0 < -1");
    cpu.assert_register(Register::X1, 0, "0 is greater than -1 (signed)");
}

#[test]
fn test_slti_edge_cases() {
    // Test signed comparison at boundaries
    // Two's complement representation is critical here
    let mut cpu = CpuBuilder::new().build();

    let mut inst = IType {
        rd: Register::X1,
        rs1: Register::X2,
        ..Default::default()
    };

    // Test with maximum positive value in rs1
    // 0x7FFFFFFF (2147483647) is the largest positive signed 32-bit
    cpu.set_register(Register::X2, values::I32_MAX);
    inst.imm.set_signed(-1).unwrap();
    let slti = Instruction::SLTI(inst);
    cpu.execute_expect(slti, "SLTI: MAX_INT < -1");
    cpu.assert_register(Register::X1, 0, "2147483647 > -1 in signed comparison");

    // Test with minimum negative value in rs1
    // 0x80000000 (-2147483648) is the most negative signed 32-bit
    cpu.set_register(Register::X2, values::I32_MIN);
    inst.imm.set_signed(0).unwrap();
    let slti = Instruction::SLTI(inst);
    cpu.execute_expect(slti, "SLTI: MIN_INT < 0");
    cpu.assert_register(Register::X1, 1, "-2147483648 < 0 in signed comparison");
}

#[test]
fn test_sltiu_unsigned_comparison() {
    // SLTIU: Set Less Than Immediate Unsigned
    // Note: Immediate is sign-extended first, then both values
    // are compared as unsigned integers
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X2, 255) // Test value
        .build();

    let mut inst = IType {
        rd: Register::X1,
        rs1: Register::X2,
        ..Default::default()
    };

    // Test equal values (255 == 255)
    inst.imm.set_unsigned(255).unwrap();
    let sltiu = Instruction::SLTIU(inst);
    cpu.execute_expect(sltiu, "SLTIU: 255 < 255");
    cpu.assert_register(Register::X1, 0, "Equal values return false");

    // Test rs1 < immediate (255 < 256)
    inst.imm.set_unsigned(256).unwrap();
    let sltiu = Instruction::SLTIU(inst);
    cpu.execute_expect(sltiu, "SLTIU: 255 < 256");
    cpu.assert_register(Register::X1, 1, "255 is less than 256");

    // Test rs1 > immediate (255 > 254)
    inst.imm.set_unsigned(254).unwrap();
    let sltiu = Instruction::SLTIU(inst);
    cpu.execute_expect(sltiu, "SLTIU: 255 < 254");
    cpu.assert_register(Register::X1, 0, "255 is greater than 254");
}

#[test]
fn test_sltiu_sign_extension() {
    // Critical test: SLTIU sign-extends immediate but compares unsigned
    // This creates interesting behavior with negative immediates
    let mut cpu = CpuBuilder::new().with_register(Register::X2, 1).build();

    let mut inst = IType {
        rd: Register::X1,
        rs1: Register::X2,
        ..Default::default()
    };

    // Set immediate to -1, which sign-extends to 0xFFFFFFFF
    // As unsigned: 0xFFFFFFFF = 4,294,967,295 (max u32)
    inst.imm.set_signed(-1).unwrap();
    let sltiu = Instruction::SLTIU(inst);
    cpu.execute_expect(sltiu, "SLTIU with sign-extended -1");

    cpu.assert_register(
        Register::X1,
        1,
        "1 < 0xFFFFFFFF (4,294,967,295 unsigned) is true",
    );

    // Special case: SLTIU rd, rs1, 1 checks if rs1 == 0
    // This works because only 0 < 1 in unsigned comparison
    cpu.set_register(Register::X2, 0);
    inst.imm.set_unsigned(1).unwrap();
    let sltiu = Instruction::SLTIU(inst);
    cpu.execute_expect(sltiu, "SLTIU zero check pattern");
    cpu.assert_register(Register::X1, 1, "0 < 1 is true (zero check)");

    cpu.set_register(Register::X2, 5);
    cpu.execute_expect(sltiu, "SLTIU non-zero check");
    cpu.assert_register(Register::X1, 0, "5 < 1 is false (non-zero)");
}

#[test]
fn test_comparison_patterns() {
    // Common comparison patterns and their differences
    // This test demonstrates key signed vs unsigned behaviors
    let mut cpu = CpuBuilder::new().build();

    // Pattern comparison table:
    // Value A   | Value B   | Signed  | Unsigned
    // ----------|-----------|---------|----------
    // -1        | 1         | A < B   | A > B
    // 0x80000000| 0x7FFFFFFF| A < B   | A > B

    let mut inst = IType {
        rd: Register::X1,
        rs1: Register::X2,
        ..Default::default()
    };

    // Case 1: -1 vs 1
    cpu.set_register(Register::X2, values::NEG_ONE); // -1 as signed
    inst.imm.set_signed(1).unwrap();

    // Signed comparison: -1 < 1
    let slti = Instruction::SLTI(inst);
    cpu.execute_expect(slti, "SLTI: -1 < 1");
    cpu.assert_register(Register::X1, 1, "-1 < 1 (signed)");

    // Unsigned comparison: 0xFFFFFFFF > 1
    let sltiu = Instruction::SLTIU(inst);
    cpu.execute_expect(sltiu, "SLTIU: 0xFFFFFFFF < 1");
    cpu.assert_register(Register::X1, 0, "0xFFFFFFFF > 1 (unsigned)");

    // Case 2: Most negative vs most positive
    cpu.set_register(Register::X2, values::I32_MIN); // 0x80000000

    // Use a positive immediate that fits in 12 bits
    inst.imm.set_signed(100).unwrap();

    // Signed: -2147483648 < 100
    let slti = Instruction::SLTI(inst);
    cpu.execute_expect(slti, "SLTI: MIN_INT < 100");
    cpu.assert_register(Register::X1, 1, "Most negative < positive (signed)");

    // Unsigned: 0x80000000 > 100
    let sltiu = Instruction::SLTIU(inst);
    cpu.execute_expect(sltiu, "SLTIU: 0x80000000 < 100");
    cpu.assert_register(Register::X1, 0, "0x80000000 > 100 (unsigned)");
}
