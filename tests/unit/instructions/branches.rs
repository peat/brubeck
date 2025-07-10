//! Unit tests for branch instructions (BEQ, BNE, BLT, BGE, BLTU, BGEU)
//!
//! These tests verify conditional branching behavior, PC updates,
//! and the proper handling of signed vs unsigned comparisons.
//!
//! Reference: RISC-V ISA Manual, Volume I: Unprivileged ISA, Version 20191213
//! Section 2.5 - Control Transfer Instructions
//!
//! Key concepts:
//! - PC-relative addressing: target = PC + sign_extend(immediate)
//! - Branch immediates encode multiples of 2 (bit 0 always 0)
//! - The immediate in the instruction is already shifted left by 1
//! - Branches do NOT have a delay slot
//!
//! Branch offset encoding visualization:
//! ```
//! Immediate value: -64 to +63 (in instruction encoding)
//! Actual offset:   -128 to +126 (after left shift by 1)
//! Target address:  PC + actual_offset
//! ```

use brubeck::rv32_i::{formats::BType, instructions::Instruction, registers::Register};

// Import test helpers
use crate::unit::test_helpers::{values, CpuAssertions, CpuBuilder, ExecuteWithContext};

#[test]
fn test_beq_equal() {
    // BEQ: Branch if Equal
    // if (rs1 == rs2) PC = PC + sign_extend(immediate)
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, 24)
        .with_register(Register::X2, 24) // Equal values
        .with_pc(0)
        .build();

    let mut inst = BType {
        rs1: Register::X1,
        rs2: Register::X2,
        ..Default::default()
    };
    inst.imm.set_signed(64).unwrap(); // Encoded offset (actual: 64*2 = 128)

    let beq = Instruction::BEQ(inst);
    cpu.execute_expect(beq, "BEQ with equal values");

    cpu.assert_pc(128, "BEQ taken: PC = 0 + 128");
}

#[test]
fn test_beq_not_equal() {
    // BEQ not taken: PC advances to next instruction
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, 24)
        .with_register(Register::X2, 25) // Not equal
        .with_pc(0)
        .build();

    let mut inst = BType {
        rs1: Register::X1,
        rs2: Register::X2,
        ..Default::default()
    };
    inst.imm.set_signed(64).unwrap();

    let beq = Instruction::BEQ(inst);
    cpu.execute_expect(beq, "BEQ with unequal values");

    cpu.assert_pc(
        Instruction::LENGTH,
        "BEQ not taken: PC advances by 4 (instruction size)",
    );
}

#[test]
fn test_beq_backward_branch() {
    // Backward branches are common in loops
    // Example: for(i=0; i<10; i++) { ... } jumps back to loop start
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, 100)
        .with_register(Register::X2, 100) // Equal values
        .with_pc(256)
        .build();

    let mut inst = BType {
        rs1: Register::X1,
        rs2: Register::X2,
        ..Default::default()
    };
    inst.imm.set_signed(-64).unwrap(); // Negative offset for backward branch

    let beq = Instruction::BEQ(inst);
    cpu.execute_expect(beq, "BEQ backward branch");

    cpu.assert_pc(128, "Backward branch: 256 + (-64*2) = 128");
}

#[test]
fn test_bne_not_equal() {
    // BNE: Branch if Not Equal
    // if (rs1 != rs2) PC = PC + sign_extend(immediate)
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, 23)
        .with_register(Register::X2, 24) // Not equal
        .with_pc(0)
        .build();

    let mut inst = BType {
        rs1: Register::X1,
        rs2: Register::X2,
        ..Default::default()
    };
    inst.imm.set_signed(64).unwrap();

    let bne = Instruction::BNE(inst);
    cpu.execute_expect(bne, "BNE with unequal values");

    cpu.assert_pc(128, "BNE taken: values differ");
}

#[test]
fn test_bne_equal() {
    // BNE not taken when values are equal
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, 24)
        .with_register(Register::X2, 24) // Equal
        .with_pc(0)
        .build();

    let mut inst = BType {
        rs1: Register::X1,
        rs2: Register::X2,
        ..Default::default()
    };
    inst.imm.set_signed(64).unwrap();

    let bne = Instruction::BNE(inst);
    cpu.execute_expect(bne, "BNE with equal values");

    cpu.assert_pc(Instruction::LENGTH, "BNE not taken: PC advances by 4");
}

#[test]
fn test_blt_signed_comparison() {
    // BLT: Branch if Less Than (signed)
    // Uses two's complement signed comparison
    let mut cpu = CpuBuilder::new().with_pc(0).build();

    let mut inst = BType {
        rs1: Register::X1,
        rs2: Register::X2,
        ..Default::default()
    };
    inst.imm.set_signed(64).unwrap();
    let blt = Instruction::BLT(inst);

    // Test cases for signed comparison
    let test_cases = [
        (23, 24, true, "positive < positive"),
        (values::NEG_ONE, 1, true, "-1 < 1 (signed)"),
        (24, 24, false, "equal values"),
        (24, 23, false, "greater than"),
    ];

    for (val1, val2, should_branch, desc) in test_cases {
        cpu.set_register(Register::X1, val1);
        cpu.set_register(Register::X2, val2);
        cpu.pc = 0;

        cpu.execute_expect(blt, desc);

        let expected_pc = if should_branch {
            128
        } else {
            Instruction::LENGTH
        };
        cpu.assert_pc(expected_pc, desc);
    }
}

#[test]
fn test_bltu_unsigned_comparison() {
    // BLTU: Branch if Less Than Unsigned
    // All values treated as unsigned integers
    let mut cpu = CpuBuilder::new().with_pc(0).build();

    let mut inst = BType {
        rs1: Register::X1,
        rs2: Register::X2,
        ..Default::default()
    };
    inst.imm.set_unsigned(64).unwrap();
    let bltu = Instruction::BLTU(inst);

    // Test cases highlighting signed vs unsigned difference
    let test_cases = [
        (23, 24, true, "simple unsigned comparison"),
        (1, values::NEG_ONE, true, "1 < 0xFFFFFFFF (unsigned)"),
        (values::NEG_ONE, 1, false, "0xFFFFFFFF > 1 (unsigned)"),
    ];

    for (val1, val2, should_branch, desc) in test_cases {
        cpu.set_register(Register::X1, val1);
        cpu.set_register(Register::X2, val2);
        cpu.pc = 0;

        cpu.execute_expect(bltu, desc);

        let expected_pc = if should_branch {
            128
        } else {
            Instruction::LENGTH
        };
        cpu.assert_pc(expected_pc, desc);
    }
}

#[test]
fn test_bge_signed_comparison() {
    // BGE: Branch if Greater or Equal (signed)
    // Branches when rs1 >= rs2 using signed comparison
    let mut cpu = CpuBuilder::new().with_pc(0).build();

    let mut inst = BType {
        rs1: Register::X1,
        rs2: Register::X2,
        ..Default::default()
    };
    inst.imm.set_signed(64).unwrap();
    let bge = Instruction::BGE(inst);

    // Test cases for >= comparison
    let test_cases = [
        (24, 23, true, "greater than"),
        (24, 24, true, "equal (boundary case)"),
        (23, 24, false, "less than"),
        (values::NEG_ONE, values::NEG_TWO, true, "-1 >= -2 (signed)"),
    ];

    for (val1, val2, should_branch, desc) in test_cases {
        cpu.set_register(Register::X1, val1);
        cpu.set_register(Register::X2, val2);
        cpu.pc = 0;

        cpu.execute_expect(bge, desc);

        let expected_pc = if should_branch {
            128
        } else {
            Instruction::LENGTH
        };
        cpu.assert_pc(expected_pc, desc);
    }
}

#[test]
fn test_bgeu_unsigned_comparison() {
    // BGEU: Branch if Greater or Equal Unsigned
    // All values treated as unsigned integers
    let mut cpu = CpuBuilder::new().with_pc(0).build();

    let mut inst = BType {
        rs1: Register::X1,
        rs2: Register::X2,
        ..Default::default()
    };
    inst.imm.set_unsigned(64).unwrap();
    let bgeu = Instruction::BGEU(inst);

    // Test cases for unsigned >= comparison
    let test_cases = [
        (24, 23, true, "simple unsigned >="),
        (values::NEG_ONE, 1, true, "0xFFFFFFFF >= 1 (unsigned)"),
        (1, values::NEG_ONE, false, "1 < 0xFFFFFFFF (unsigned)"),
    ];

    for (val1, val2, should_branch, desc) in test_cases {
        cpu.set_register(Register::X1, val1);
        cpu.set_register(Register::X2, val2);
        cpu.pc = 0;

        cpu.execute_expect(bgeu, desc);

        let expected_pc = if should_branch {
            128
        } else {
            Instruction::LENGTH
        };
        cpu.assert_pc(expected_pc, desc);
    }
}

#[test]
fn test_branch_offset_encoding() {
    // RISC-V branch encoding: immediate represents multiples of 2
    // This allows branches to any halfword-aligned address
    // Actual offset = encoded_immediate * 2
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, 100)
        .with_register(Register::X2, 100) // Equal for BEQ
        .build();

    let mut inst = BType {
        rs1: Register::X1,
        rs2: Register::X2,
        ..Default::default()
    };

    // Test various offsets (encoded_imm, actual_offset)
    let test_cases = [
        (0, 0),         // No branch
        (2, 4),         // Forward by 4 bytes (1 instruction)
        (4, 8),         // Forward by 8 bytes (2 instructions)
        (-2, -4),       // Backward by 4 bytes
        (-4, -8),       // Backward by 8 bytes
        (2047, 4094),   // Max positive (12-bit signed)
        (-2048, -4096), // Max negative
    ];

    for (encoded_imm, actual_offset, desc) in
        test_cases.map(|(i, o)| (i, o, format!("offset {i} -> PC+{o}")))
    {
        cpu.pc = 1000; // Start from non-zero PC
        inst.imm.set_signed(encoded_imm).unwrap();

        let beq = Instruction::BEQ(inst);
        cpu.execute_expect(beq, &desc);

        let expected_pc = (1000i32 + actual_offset) as u32;
        cpu.assert_pc(expected_pc, &desc);
    }
}

#[test]
fn test_branch_with_x0() {
    // Common RISC-V patterns using x0 (always zero):
    // - BEQ rs, x0: branch if rs == 0
    // - BNE rs, x0: branch if rs != 0
    // - BLT x0, rs: branch if rs > 0 (signed)
    // - BGE rs, x0: branch if rs >= 0 (signed)
    let mut cpu = CpuBuilder::new().with_pc(0).build();

    let mut inst = BType {
        rs1: Register::X1,
        rs2: Register::X0, // x0 always reads as 0
        ..Default::default()
    };
    inst.imm.set_signed(64).unwrap();

    // Pattern 1: BEQ x1, x0 - branch if x1 == 0
    cpu.set_register(Register::X1, 0);
    cpu.pc = 0;
    let beq = Instruction::BEQ(inst);
    cpu.execute_expect(beq, "BEQ with x0");
    cpu.assert_pc(128, "BEQ x1, x0: branches when x1 is zero");

    // Pattern 2: BNE x1, x0 - branch if x1 != 0
    cpu.set_register(Register::X1, 42);
    cpu.pc = 0;
    let bne = Instruction::BNE(inst);
    cpu.execute_expect(bne, "BNE with x0");
    cpu.assert_pc(128, "BNE x1, x0: branches when x1 is non-zero");
}
