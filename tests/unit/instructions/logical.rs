//! Unit tests for logical instructions (AND, ANDI, OR, ORI, XOR, XORI)
//!
//! These tests verify bitwise logical operations, including immediate
//! sign extension and common bit manipulation patterns.
//!
//! Reference: RISC-V ISA Manual, Volume I: Unprivileged ISA, Version 20191213
//! Section 2.4 - Integer Computational Instructions
//!
//! Key concepts:
//! - AND, OR, XOR perform bitwise operations
//! - Immediate versions sign-extend the 12-bit immediate to 32 bits
//! - Common patterns: masking, setting bits, toggling bits
//! - XORI rd, rs1, -1 is the NOT operation (pseudo-instruction)
//!
//! Truth tables:
//! ```
//! AND: 0&0=0, 0&1=0, 1&0=0, 1&1=1
//! OR:  0|0=0, 0|1=1, 1|0=1, 1|1=1  
//! XOR: 0^0=0, 0^1=1, 1^0=1, 1^1=0
//! ```

use brubeck::rv32_i::{formats::IType, instructions::Instruction, registers::Register};

// Import test helpers
use crate::unit::test_helpers::{values, CpuAssertions, CpuBuilder, ExecuteWithContext};

#[test]
fn test_andi_ori_xori_all_ones() {
    // Test logical operations with all bits set
    // Note: 12-bit immediate -1 sign-extends to 0xFFFFFFFF
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X2, values::NEG_ONE) // All 1s
        .build();

    let mut inst = IType::default();
    inst.rd = Register::X1;
    inst.rs1 = Register::X2;
    inst.imm.set_signed(-1).unwrap(); // Sign-extends to all 1s

    // ANDI: all 1s AND all 1s = all 1s
    let andi = Instruction::ANDI(inst);
    cpu.execute_expect(andi, "ANDI with all ones");
    cpu.assert_register(Register::X1, values::NEG_ONE, "1s AND 1s = 1s (identity)");

    // ORI: all 1s OR all 1s = all 1s
    let ori = Instruction::ORI(inst);
    cpu.execute_expect(ori, "ORI with all ones");
    cpu.assert_register(Register::X1, values::NEG_ONE, "1s OR 1s = 1s (idempotent)");

    // XORI: all 1s XOR all 1s = all 0s
    let xori = Instruction::XORI(inst);
    cpu.execute_expect(xori, "XORI with all ones");
    cpu.assert_register(Register::X1, 0, "1s XOR 1s = 0s (self-inverse)");
}

#[test]
fn test_andi_ori_xori_all_zeros() {
    // Test logical operations with zero (identity/annihilator)
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X2, values::NEG_ONE) // Test value
        .build();

    let mut inst = IType::default();
    inst.rd = Register::X1;
    inst.rs1 = Register::X2;
    inst.imm.set_unsigned(0).unwrap();

    // ANDI: value AND 0 = 0 (zero is annihilator)
    let andi = Instruction::ANDI(inst);
    cpu.execute_expect(andi, "ANDI with zero");
    cpu.assert_register(Register::X1, 0, "x AND 0 = 0 (annihilator)");

    // ORI: value OR 0 = value (zero is identity)
    let ori = Instruction::ORI(inst);
    cpu.execute_expect(ori, "ORI with zero");
    cpu.assert_register(Register::X1, values::NEG_ONE, "x OR 0 = x (identity)");

    // XORI: value XOR 0 = value (zero is identity)
    let xori = Instruction::XORI(inst);
    cpu.execute_expect(xori, "XORI with zero");
    cpu.assert_register(Register::X1, values::NEG_ONE, "x XOR 0 = x (identity)");
}

#[test]
fn test_logical_sign_extension() {
    // Critical: Logical immediates are sign-extended!
    // This affects how masks work with negative immediates
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X2, 0xFFFF_0000) // Upper half set
        .build();

    let mut inst = IType::default();
    inst.rd = Register::X1;
    inst.rs1 = Register::X2;
    inst.imm.set_signed(-1).unwrap(); // 12-bit -1 becomes 32-bit 0xFFFFFFFF

    // ANDI: Preserves only bits set in both operands
    let andi = Instruction::ANDI(inst);
    cpu.execute_expect(andi, "ANDI with sign-extended -1");
    cpu.assert_register(
        Register::X1,
        0xFFFF_0000,
        "0xFFFF0000 AND 0xFFFFFFFF = 0xFFFF0000",
    );

    // ORI: Sets all bits that are set in either operand
    cpu.set_register(Register::X2, 0xFFFF_0000);
    let ori = Instruction::ORI(inst);
    cpu.execute_expect(ori, "ORI with sign-extended -1");
    cpu.assert_register(
        Register::X1,
        0xFFFF_FFFF,
        "0xFFFF0000 OR 0xFFFFFFFF = 0xFFFFFFFF",
    );

    // XORI: Inverts bits where operands differ
    cpu.set_register(Register::X2, 0xFFFF_0000);
    let xori = Instruction::XORI(inst);
    cpu.execute_expect(xori, "XORI with sign-extended -1");
    cpu.assert_register(
        Register::X1,
        0x0000_FFFF,
        "0xFFFF0000 XOR 0xFFFFFFFF = 0x0000FFFF (complement lower half)",
    );
}

#[test]
fn test_xori_bitwise_not() {
    // RISC-V idiom: XORI rd, rs1, -1 implements NOT
    // This is a pseudo-instruction: NOT rd, rs
    let mut cpu = CpuBuilder::new().build();

    let mut inst = IType::default();
    inst.rd = Register::X1;
    inst.rs1 = Register::X2;
    inst.imm.set_signed(-1).unwrap(); // The magic immediate for NOT
    let xori = Instruction::XORI(inst);

    // Test NOT of various bit patterns
    let test_patterns = [
        (
            values::ALTERNATING_BITS,
            values::ALTERNATING_BITS_INV,
            "NOT alternating bits",
        ),
        (0x0F0F_0F0F, 0xF0F0_F0F0, "NOT nibble pattern"),
        (0x00FF_00FF, 0xFF00_FF00, "NOT byte pattern"),
        (
            values::ONE,
            values::NEG_ONE - 1,
            "NOT 1 = -2 (two's complement)",
        ),
    ];

    for (input, expected, desc) in test_patterns {
        cpu.set_register(Register::X2, input);
        cpu.execute_expect(xori, desc);
        cpu.assert_register(Register::X1, expected, desc);
    }
}

#[test]
fn test_logical_common_patterns() {
    // Common bit manipulation patterns in RISC-V
    let mut cpu = CpuBuilder::new().build();

    let mut inst = IType::default();
    inst.rd = Register::X1;
    inst.rs1 = Register::X2;

    // Pattern 1: Extract lower byte with ANDI
    cpu.set_register(Register::X2, 0x1234_5678);
    inst.imm.set_unsigned(0xFF).unwrap();
    let andi = Instruction::ANDI(inst);
    cpu.execute_expect(andi, "Extract lower byte");
    cpu.assert_register(Register::X1, 0x78, "ANDI x, 0xFF extracts lowest byte");

    // Pattern 2: Set bits with ORI (combine values)
    cpu.set_register(Register::X2, 0x1234_5600);
    inst.imm.set_unsigned(0x78).unwrap();
    let ori = Instruction::ORI(inst);
    cpu.execute_expect(ori, "Set specific bits");
    cpu.assert_register(
        Register::X1,
        0x1234_5678,
        "ORI sets bits without affecting others",
    );

    // Pattern 3: Toggle bits with XORI (flip specific bits)
    cpu.set_register(Register::X2, 0x1234_5678);
    inst.imm.set_unsigned(0xF0).unwrap();
    let xori = Instruction::XORI(inst);
    cpu.execute_expect(xori, "Toggle bit pattern");
    cpu.assert_register(
        Register::X1,
        0x1234_5688,
        "XORI toggles specific bits: 0x78 XOR 0xF0 = 0x88",
    );
}
