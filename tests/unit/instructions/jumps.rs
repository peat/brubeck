//! Unit tests for jump instructions (JAL, JALR)
//!
//! These tests verify unconditional jumps, link register behavior,
//! and alignment requirements as specified in the RISC-V ISA.
//!
//! Reference: RISC-V ISA Manual, Volume I: Unprivileged ISA, Version 20191213
//! Section 2.5 - Control Transfer Instructions
//!
//! Key concepts:
//! - JAL: Jump And Link - PC-relative jump with return address
//! - JALR: Jump And Link Register - register-indirect jump
//! - Return address = PC + 4 (next instruction)
//! - JAL offset is in multiples of 2 (like branches)
//! - JALR clears LSB of target address for alignment
//!
//! Common patterns:
//! ```
//! JAL ra, function    # Call function (ra = x1)
//! ...
//! JALR x0, 0(ra)     # Return from function
//! ```

use brubeck::rv32_i::{
    formats::{IType, JType},
    instructions::Instruction,
    registers::Register,
};

// Import test helpers
use crate::unit::test_helpers::{CpuAssertions, CpuBuilder, ExecuteWithContext};

#[test]
fn test_jal_basic() {
    // JAL: Jump And Link
    // rd = PC + 4; PC = PC + sign_extend(immediate)
    // Used for function calls when offset is known at compile time
    let mut cpu = CpuBuilder::new().with_pc(0).build();

    let mut inst = JType {
        rd: Register::X1, // Standard link register (ra)
        ..Default::default()
    };
    inst.imm.set_unsigned(4).unwrap(); // Encoded offset (actual: 4*2 = 8)

    let jal = Instruction::JAL(inst);
    cpu.execute_expect(jal, "JAL forward jump");

    // JAL performs two operations:
    cpu.assert_pc(8, "PC = 0 + 8 (forward jump)");
    cpu.assert_register(Register::X1, 4, "ra = PC + 4 (return address)");
}

#[test]
fn test_jal_misalignment() {
    // RV32I requires 4-byte alignment for instruction fetch
    // JAL to misaligned address should fail
    let mut cpu = CpuBuilder::new().with_pc(0).build();

    let mut inst = JType {
        rd: Register::X1,
        ..Default::default()
    };
    inst.imm.set_unsigned(1).unwrap(); // Becomes offset 2 (not 4-byte aligned)

    let jal = Instruction::JAL(inst);
    let result = cpu.execute(jal);

    assert!(
        result.is_err(),
        "JAL to address 2 should fail (requires 4-byte alignment)"
    );
}

#[test]
fn test_jal_negative_offset() {
    // Backward jumps are used for loops and backward branches
    let mut cpu = CpuBuilder::new().with_pc(1000).build();

    let mut inst = JType {
        rd: Register::X1,
        ..Default::default()
    };
    inst.imm.set_signed(-100).unwrap(); // Encoded: -100, Actual: -200

    let jal = Instruction::JAL(inst);
    cpu.execute_expect(jal, "JAL backward jump");

    cpu.assert_pc(800, "PC = 1000 + (-200) = 800");
    cpu.assert_register(Register::X1, 1004, "ra saves next instruction");
}

#[test]
fn test_jal_x0_destination() {
    // JAL x0, offset is effectively an unconditional jump (no return)
    // Common pattern: infinite loops, goto statements
    let mut cpu = CpuBuilder::new().with_pc(0).build();

    let mut inst = JType {
        rd: Register::X0, // Discard return address
        ..Default::default()
    };
    inst.imm.set_unsigned(10).unwrap();

    let jal = Instruction::JAL(inst);
    cpu.execute_expect(jal, "JAL with x0 (goto)");

    cpu.assert_pc(20, "Jump executed: PC = 0 + 20");
    cpu.assert_register(Register::X0, 0, "x0 remains zero (write ignored)");
}

#[test]
fn test_jalr_basic() {
    // JALR: Jump And Link Register
    // rd = PC + 4; PC = (rs1 + sign_extend(immediate)) & ~1
    // Used for indirect jumps (function pointers, returns)
    let mut cpu = CpuBuilder::new()
        .with_pc(0)
        .with_register(Register::X2, 100) // Base address
        .build();

    let mut inst = IType {
        rs1: Register::X2,
        rd: Register::X1, // Save return address
        ..Default::default()
    };
    inst.imm.set_signed(12).unwrap();

    let jalr = Instruction::JALR(inst);
    cpu.execute_expect(jalr, "JALR basic jump");

    cpu.assert_pc(112, "PC = (100 + 12) & ~1 = 112");
    cpu.assert_register(Register::X1, 4, "Return address saved");
}

#[test]
fn test_jalr_with_base() {
    // JALR with negative offset - common for computed jumps
    let mut cpu = CpuBuilder::new()
        .with_pc(0)
        .with_register(Register::X2, 24) // Base address
        .build();

    let mut inst = IType {
        rs1: Register::X2,
        rd: Register::X1,
        ..Default::default()
    };
    inst.imm.set_signed(-12).unwrap();

    let jalr = Instruction::JALR(inst);
    cpu.execute_expect(jalr, "JALR with negative offset");

    cpu.assert_pc(12, "PC = 24 + (-12) = 12");
    cpu.assert_register(Register::X1, 4, "Return address = 0 + 4");
}

#[test]
fn test_jalr_least_significant_bit() {
    // JALR clears LSB to ensure even address alignment
    // This supports future compressed instructions (2-byte aligned)
    let mut cpu = CpuBuilder::new()
        .with_pc(0)
        .with_register(Register::X2, 13) // Odd address
        .build();

    let mut inst = IType {
        rs1: Register::X2,
        rd: Register::X1,
        ..Default::default()
    };
    inst.imm.set_unsigned(0).unwrap();

    let jalr = Instruction::JALR(inst);
    cpu.execute_expect(jalr, "JALR with odd address");

    cpu.assert_pc(12, "PC = 13 & ~1 = 12 (LSB cleared)");
}

#[test]
fn test_jalr_return_pattern() {
    // Standard function return: JALR x0, 0(ra)
    // This is the RET pseudo-instruction
    let mut cpu = CpuBuilder::new()
        .with_pc(0x2000) // Current function location
        .with_register(Register::X1, 0x1000) // Return address (ra)
        .build();

    let mut inst = IType {
        rd: Register::X0,  // Discard "return" address
        rs1: Register::X1, // Jump to address in ra
        ..Default::default()
    };
    inst.imm.set_unsigned(0).unwrap();

    let jalr = Instruction::JALR(inst);
    cpu.execute_expect(jalr, "Function return (RET)");

    cpu.assert_pc(0x1000, "Returned to caller");
    cpu.assert_register(Register::X0, 0, "x0 unchanged");
}

#[test]
fn test_jal_jalr_call_return() {
    // Complete function call/return sequence
    // This demonstrates the standard RISC-V calling convention
    let mut cpu = CpuBuilder::new()
        .with_pc(0x1000) // Main program
        .build();

    // Step 1: Call function using JAL
    let mut jal_inst = JType {
        rd: Register::X1, // Save return address in ra
        ..Default::default()
    };
    jal_inst.imm.set_unsigned(100).unwrap(); // Jump forward 200 bytes

    let jal = Instruction::JAL(jal_inst);
    cpu.execute_expect(jal, "JAL to function");

    // Verify function call
    cpu.assert_pc(0x1000 + 200, "Jumped to function at 0x10C8");
    cpu.assert_register(Register::X1, 0x1004, "Return address saved");

    // Step 2: Simulate some work in the function
    cpu.pc = 0x2000; // Function does some work...

    // Step 3: Return from function using JALR (RET pseudo-instruction)
    let mut jalr_inst = IType {
        rd: Register::X0,  // RET doesn't save new return address
        rs1: Register::X1, // Jump to saved return address
        ..Default::default()
    };
    jalr_inst.imm.set_unsigned(0).unwrap();

    let jalr = Instruction::JALR(jalr_inst);
    cpu.execute_expect(jalr, "JALR return from function");

    cpu.assert_pc(0x1004, "Returned to instruction after JAL");
}
