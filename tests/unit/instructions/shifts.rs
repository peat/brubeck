//! Unit tests for shift instructions (SLL, SLLI, SRL, SRLI, SRA, SRAI)
//!
//! These tests verify logical and arithmetic shift operations,
//! including the 5-bit shift amount limitation and sign extension behavior.
//!
//! Reference: RISC-V ISA Manual, Volume I: Unprivileged ISA, Version 20191213
//! Section 2.4 - Integer Computational Instructions
//!
//! Key RISC-V shift behaviors:
//! - Shift amounts are masked to 5 bits (& 0x1F), limiting shifts to 0-31
//! - SLL/SRL: Logical shifts (fill with zeros)
//! - SRA: Arithmetic shift (sign-extend from MSB)
//! - Immediate shifts use 5-bit immediate field directly

use brubeck::rv32_i::{
    formats::{IType, RType},
    instructions::Instruction,
    registers::Register,
};

// Import test helpers
use crate::unit::test_helpers::{values, CpuAssertions, CpuBuilder, ExecuteWithContext};

#[test]
fn test_sll_basic() {
    // SLL: Shift Left Logical
    // rd = rs1 << (rs2 & 0x1F)
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, 0b1010) // Value: 10
        .with_register(Register::X2, 2) // Shift amount
        .build();

    let inst = RType {
        rd: Register::X3,
        rs1: Register::X1,
        rs2: Register::X2,
        ..Default::default()
    };

    let sll = Instruction::SLL(inst);
    cpu.execute_expect(sll, "SLL basic shift");

    cpu.assert_register(Register::X3, 0b101000, "10 << 2 = 40");
}

#[test]
fn test_sll_5bit_mask() {
    // RISC-V key behavior: shift amounts use only lower 5 bits
    // This prevents undefined behavior for shifts >= 32
    let mut cpu = CpuBuilder::new().with_register(Register::X1, 1).build();

    let inst = RType {
        rd: Register::X3,
        rs1: Register::X1,
        rs2: Register::X2,
        ..Default::default()
    };
    let sll = Instruction::SLL(inst);

    // Test cases demonstrating 5-bit masking
    let test_cases = [
        (32, 0, "32 & 0x1F = 0: no shift"),
        (33, 1, "33 & 0x1F = 1: shift by 1"),
        (63, 31, "63 & 0x1F = 31: max shift"),
        (64, 0, "64 & 0x1F = 0: wraps to 0"),
    ];

    for (shift_amt, expected_shift, desc) in test_cases {
        cpu.set_register(Register::X2, shift_amt);
        cpu.execute_expect(sll, desc);
        cpu.assert_register(Register::X3, 1 << expected_shift, desc);
    }
}

#[test]
fn test_slli_basic() {
    // SLLI: Shift Left Logical Immediate
    // rd = rs1 << shamt[4:0]
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, 0b1111) // Value: 15
        .build();

    let mut inst = IType {
        rd: Register::X2,
        rs1: Register::X1,
        ..Default::default()
    };
    inst.imm.set_unsigned(3).unwrap(); // Shift by 3

    let slli = Instruction::SLLI(inst);
    cpu.execute_expect(slli, "SLLI by 3");

    cpu.assert_register(Register::X2, 0b1111000, "15 << 3 = 120");
}

#[test]
fn test_slli_max_shift() {
    // Test maximum valid shift amount (31)
    // Shifting 1 by 31 positions sets only the MSB
    let mut cpu = CpuBuilder::new().with_register(Register::X1, 1).build();

    let mut inst = IType {
        rd: Register::X2,
        rs1: Register::X1,
        ..Default::default()
    };
    inst.imm.set_unsigned(31).unwrap(); // Maximum valid shift

    let slli = Instruction::SLLI(inst);
    cpu.execute_expect(slli, "SLLI maximum shift");

    cpu.assert_register(
        Register::X2,
        values::MSB_SET,
        "1 << 31 sets only MSB (0x80000000)",
    );
}

#[test]
fn test_srl_basic() {
    // SRL: Shift Right Logical
    // rd = rs1 >> (rs2 & 0x1F)
    // Fills with zeros from the left
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, 0b111100) // Value: 60
        .with_register(Register::X2, 2) // Shift amount
        .build();

    let inst = RType {
        rd: Register::X3,
        rs1: Register::X1,
        rs2: Register::X2,
        ..Default::default()
    };

    let srl = Instruction::SRL(inst);
    cpu.execute_expect(srl, "SRL basic shift");

    cpu.assert_register(Register::X3, 0b1111, "60 >> 2 = 15");
}

#[test]
fn test_srl_no_sign_extension() {
    // SRL is logical shift - always fills with zeros
    // This differs from SRA (arithmetic shift)
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, values::MSB_SET) // 0x80000000
        .with_register(Register::X2, 1)
        .build();

    let inst = RType {
        rd: Register::X3,
        rs1: Register::X1,
        rs2: Register::X2,
        ..Default::default()
    };

    let srl = Instruction::SRL(inst);
    cpu.execute_expect(srl, "SRL with MSB set");

    cpu.assert_register(
        Register::X3,
        0x40000000,
        "SRL fills with zeros, not sign bit",
    );
}

#[test]
fn test_srli_basic() {
    // SRLI: Shift Right Logical Immediate
    // Like SRL but with immediate shift amount
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, 0xFF000000)
        .build();

    let mut inst = IType {
        rd: Register::X2,
        rs1: Register::X1,
        ..Default::default()
    };
    inst.imm.set_unsigned(8).unwrap();

    let srli = Instruction::SRLI(inst);
    cpu.execute_expect(srli, "SRLI by 8");

    cpu.assert_register(Register::X2, 0x00FF0000, "SRLI shifts right with zero fill");
}

#[test]
fn test_sra_positive_number() {
    // SRA: Shift Right Arithmetic
    // For positive numbers (MSB=0), behaves like SRL
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, values::I32_MAX) // 0x7FFFFFFF
        .with_register(Register::X2, 4)
        .build();

    let inst = RType {
        rd: Register::X3,
        rs1: Register::X1,
        rs2: Register::X2,
        ..Default::default()
    };

    let sra = Instruction::SRA(inst);
    cpu.execute_expect(sra, "SRA positive number");

    cpu.assert_register(
        Register::X3,
        0x07FFFFFF,
        "SRA on positive number fills with zeros",
    );
}

#[test]
fn test_sra_negative_number() {
    // SRA preserves sign by filling with sign bit (MSB)
    // This is the key difference from SRL
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, 0xFFFFFF00) // -256 in two's complement
        .with_register(Register::X2, 4)
        .build();

    let inst = RType {
        rd: Register::X3,
        rs1: Register::X1,
        rs2: Register::X2,
        ..Default::default()
    };

    let sra = Instruction::SRA(inst);
    cpu.execute_expect(sra, "SRA negative number");

    // -256 >> 4 = -16 (0xFFFFFFF0)
    cpu.assert_register(
        Register::X3,
        0xFFFFFFF0,
        "SRA extends sign bit (arithmetic shift right)",
    );
}

#[test]
fn test_sra_sign_extension_pattern() {
    // Demonstrate how SRA propagates the sign bit
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, 0xFFFF0000) // Negative with pattern
        .with_register(Register::X2, 8)
        .build();

    let inst = RType {
        rd: Register::X3,
        rs1: Register::X1,
        rs2: Register::X2,
        ..Default::default()
    };

    let sra = Instruction::SRA(inst);
    cpu.execute_expect(sra, "SRA with pattern");

    cpu.assert_register(
        Register::X3,
        0xFFFFFF00,
        "SRA fills shifted bits with sign bit (1s)",
    );
}

#[test]
fn test_srai_basic() {
    // SRAI: Shift Right Arithmetic Immediate
    // Sign-preserving division by powers of 2
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, (-1024_i32) as u32)
        .build();

    let mut inst = IType {
        rd: Register::X2,
        rs1: Register::X1,
        ..Default::default()
    };
    inst.imm.set_unsigned(2).unwrap();

    let srai = Instruction::SRAI(inst);
    cpu.execute_expect(srai, "SRAI negative value");

    cpu.assert_register(
        Register::X2,
        (-256_i32) as u32,
        "SRAI: -1024 >> 2 = -256 (arithmetic shift)",
    );
}

#[test]
fn test_shift_by_zero() {
    // Shifting by 0 should preserve the original value
    // This is a common edge case in shift operations
    let test_value = 0x12345678;
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, test_value)
        .with_register(Register::X2, 0) // Shift amount = 0
        .build();

    let inst = RType {
        rd: Register::X3,
        rs1: Register::X1,
        rs2: Register::X2,
        ..Default::default()
    };

    // Test all shift operations with zero
    let shifts = [
        (Instruction::SLL(inst), "SLL by 0"),
        (Instruction::SRL(inst), "SRL by 0"),
        (Instruction::SRA(inst), "SRA by 0"),
    ];

    for (shift_inst, desc) in shifts {
        cpu.execute_expect(shift_inst, desc);
        cpu.assert_register(
            Register::X3,
            test_value,
            "Shift by 0 preserves original value",
        );
    }
}

#[test]
fn test_shift_edge_cases() {
    // Test boundary conditions for shift operations
    let mut cpu = CpuBuilder::new().build();

    let mut inst = IType {
        rd: Register::X2,
        rs1: Register::X1,
        ..Default::default()
    };
    inst.imm.set_unsigned(31).unwrap(); // Maximum shift amount

    // Case 1: Shift 1 left by 31 (sets only MSB)
    cpu.set_register(Register::X1, 1);
    let slli = Instruction::SLLI(inst);
    cpu.execute_expect(slli, "SLLI 1 by 31");
    cpu.assert_register(Register::X2, values::MSB_SET, "1 << 31 = 0x80000000");

    // Case 2: Shift MSB right logically by 31 (isolates MSB)
    cpu.set_register(Register::X1, values::MSB_SET);
    let srli = Instruction::SRLI(inst);
    cpu.execute_expect(srli, "SRLI MSB by 31");
    cpu.assert_register(Register::X2, 1, "0x80000000 >> 31 = 1 (logical)");

    // Case 3: Arithmetic shift of -1 (all ones)
    cpu.set_register(Register::X1, values::NEG_ONE);
    let srai = Instruction::SRAI(inst);
    cpu.execute_expect(srai, "SRAI -1 by 31");
    cpu.assert_register(
        Register::X2,
        values::NEG_ONE,
        "-1 >> 31 = -1 (arithmetic preserves sign)",
    );
}

#[test]
fn test_shift_chain() {
    // Test a sequence of shifts to verify they compose correctly
    // Common pattern in bit manipulation algorithms
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, 0x0F) // Starting value: 15
        .build();

    // Step 1: Shift left by 4 (multiply by 16)
    let mut inst = IType {
        rd: Register::X2,
        rs1: Register::X1,
        ..Default::default()
    };
    inst.imm.set_unsigned(4).unwrap();

    let slli = Instruction::SLLI(inst);
    cpu.execute_expect(slli, "SLLI by 4");
    cpu.assert_register(Register::X2, 0xF0, "15 << 4 = 240");

    // Step 2: Shift right logically by 2 (divide by 4)
    inst.rd = Register::X3;
    inst.rs1 = Register::X2;
    inst.imm.set_unsigned(2).unwrap();

    let srli = Instruction::SRLI(inst);
    cpu.execute_expect(srli, "SRLI by 2");
    cpu.assert_register(Register::X3, 0x3C, "240 >> 2 = 60");

    // Result: (15 << 4) >> 2 = 60
}
