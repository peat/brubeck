//! Unit tests for arithmetic instructions (ADD, ADDI, SUB)
//!
//! These tests verify the behavior of basic arithmetic operations as specified
//! in the RISC-V ISA manual, including overflow behavior and sign extension.
//!
//! Reference: RISC-V ISA Manual, Volume I: Unprivileged ISA, Version 20191213
//! Section 2.4 - Integer Computational Instructions
//!
//! Key concepts tested:
//! - Modular arithmetic (overflow wraps at 2^32)
//! - Sign extension of 12-bit immediates
//! - x0 register special behavior (hardwired to zero)

use brubeck::rv32_i::{
    formats::{IType, RType},
    instructions::Instruction,
    registers::Register,
};

// Import test helpers
use crate::unit::test_helpers::{values, CpuAssertions, CpuBuilder, ExecuteWithContext};

#[test]
fn test_add_sub_basic() {
    // ADD: rd = rs1 + rs2
    // SUB: rd = rs1 - rs2
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X2, 8)
        .with_register(Register::X3, 4)
        .build();

    let rtype = RType {
        rd: Register::X1,
        rs1: Register::X2,
        rs2: Register::X3,
        ..Default::default()
    };

    let add = Instruction::ADD(rtype);
    let sub = Instruction::SUB(rtype);

    // Test ADD: 8 + 4 = 12
    cpu.execute_expect(add, "ADD: 8 + 4");
    cpu.assert_register(Register::X1, 12, "ADD result");

    // Test SUB: 8 - 4 = 4
    cpu.execute_expect(sub, "SUB: 8 - 4");
    cpu.assert_register(Register::X1, 4, "SUB result");
}

#[test]
fn test_add_sub_overflow() {
    // RISC-V spec: "Arithmetic overflow is ignored and the result is simply
    // the low XLEN bits of the result" (XLEN=32 for RV32I)
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X2, 3)
        .with_register(Register::X3, values::U32_MAX - 1)
        .build();

    let rtype = RType {
        rd: Register::X1,
        rs1: Register::X2,
        rs2: Register::X3,
        ..Default::default()
    };

    let add = Instruction::ADD(rtype);
    let sub = Instruction::SUB(rtype);

    // Test unsigned overflow in addition: 3 + (2^32-2) = 2^32+1 = 1 (mod 2^32)
    cpu.execute_expect(add, "ADD with overflow");
    cpu.assert_register(Register::X1, 1, "ADD overflow wraps to 1");

    // Test subtraction with overflow: 3 - (2^32-2) = 5 (mod 2^32)
    cpu.execute_expect(sub, "SUB with underflow");
    cpu.assert_register(Register::X1, 5, "SUB underflow wraps to 5");
}

#[test]
fn test_addi_basic() {
    // ADDI: rd = rs1 + sign_extend(immediate[11:0])
    // Common uses: increment/decrement, stack pointer adjustment, load immediate
    let mut cpu = CpuBuilder::new().build();

    // Test 1: Add zero (NOP-like behavior when rd=rs1)
    let mut inst = IType {
        rd: Register::X1,
        rs1: Register::X1,
        ..Default::default()
    };
    inst.imm.set_unsigned(0).unwrap();
    let addi = Instruction::ADDI(inst);
    cpu.execute_expect(addi, "ADDI with zero immediate");
    cpu.assert_register(Register::X1, 0, "0 + 0 = 0");

    // Test 2: Positive immediate (common increment pattern)
    inst.imm.set_unsigned(5).unwrap();
    let addi = Instruction::ADDI(inst);
    cpu.execute_expect(addi, "ADDI positive immediate");
    cpu.assert_register(Register::X1, 5, "0 + 5 = 5");

    // Test 3: Negative immediate (tests sign extension)
    inst.imm.set_signed(-3).unwrap();
    let addi = Instruction::ADDI(inst);
    cpu.execute_expect(addi, "ADDI negative immediate");
    cpu.assert_register(Register::X1, 2, "5 + (-3) = 2");
}

#[test]
fn test_addi_sign_extension() {
    // RISC-V spec: ADDI sign-extends the 12-bit immediate to 32 bits
    // This is critical for negative numbers and address calculations
    let mut cpu = CpuBuilder::new().build();

    // Test boundary values for 12-bit signed immediate
    let test_cases = [
        (2047, 2047_u32, "max positive immediate (+2047)"),
        (-2048, (-2048_i32) as u32, "max negative immediate (-2048)"),
        (
            -1,
            values::NEG_ONE,
            "negative one sign-extends to 0xFFFFFFFF",
        ),
        (1, 1, "positive values unchanged"),
    ];

    for (imm_val, expected, desc) in test_cases {
        let mut inst = IType {
            rd: Register::X1,
            rs1: Register::X0, // x0 = 0, so result = 0 + immediate
            ..Default::default()
        };
        inst.imm.set_signed(imm_val).unwrap();
        let addi = Instruction::ADDI(inst);

        cpu.execute_expect(addi, desc);
        cpu.assert_register(Register::X1, expected, desc);
    }
}

#[test]
fn test_x0_destination() {
    // RISC-V spec: x0 is hardwired to zero
    // - Reads always return 0
    // - Writes are ignored
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, 100)
        .with_register(Register::X2, 200)
        .build();

    let rtype = RType {
        rd: Register::X0, // Attempt to write to x0
        rs1: Register::X1,
        rs2: Register::X2,
        ..Default::default()
    };

    let add = Instruction::ADD(rtype);
    cpu.execute_expect(add, "ADD to x0 register");

    cpu.assert_register(Register::X0, 0, "x0 remains zero after write attempt");
}
