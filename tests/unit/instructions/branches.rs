//! Unit tests for branch instructions (BEQ, BNE, BLT, BGE, BLTU, BGEU)
//!
//! These tests verify conditional branching behavior, PC updates,
//! and the proper handling of signed vs unsigned comparisons.

use brubeck::rv32_i::{
    cpu::CPU,
    formats::BType,
    instructions::Instruction,
    registers::Register,
};

#[test]
fn test_beq_equal() {
    let mut cpu = CPU::default();
    let mut inst = BType::default();
    
    cpu.x1 = 24;
    cpu.x2 = 24; // Equal values
    cpu.pc = 0;
    
    inst.rs1 = Register::X1;
    inst.rs2 = Register::X2;
    inst.imm.set_signed(64).unwrap(); // Branch offset (will be doubled)
    
    let beq = Instruction::BEQ(inst);
    let result = cpu.execute(beq);
    assert!(result.is_ok());
    assert_eq!(cpu.pc, 128, "BEQ: Should branch when equal (0 + 64*2 = 128)");
}

#[test]
fn test_beq_not_equal() {
    let mut cpu = CPU::default();
    let mut inst = BType::default();
    
    cpu.x1 = 24;
    cpu.x2 = 25; // Not equal
    cpu.pc = 0;
    
    inst.rs1 = Register::X1;
    inst.rs2 = Register::X2;
    inst.imm.set_signed(64).unwrap();
    
    let beq = Instruction::BEQ(inst);
    let result = cpu.execute(beq);
    assert!(result.is_ok());
    assert_eq!(cpu.pc, Instruction::LENGTH, 
        "BEQ: Should not branch when not equal (PC advances by 4)");
}

#[test]
fn test_beq_backward_branch() {
    let mut cpu = CPU::default();
    let mut inst = BType::default();
    
    cpu.x1 = 100;
    cpu.x2 = 100;
    cpu.pc = 256;
    
    inst.rs1 = Register::X1;
    inst.rs2 = Register::X2;
    inst.imm.set_signed(-64).unwrap(); // Negative offset
    
    let beq = Instruction::BEQ(inst);
    cpu.execute(beq).unwrap();
    assert_eq!(cpu.pc, 128, "BEQ: Should branch backward (256 + (-64*2) = 128)");
}

#[test]
fn test_bne_not_equal() {
    let mut cpu = CPU::default();
    let mut inst = BType::default();
    
    cpu.x1 = 23;
    cpu.x2 = 24; // Not equal
    cpu.pc = 0;
    
    inst.rs1 = Register::X1;
    inst.rs2 = Register::X2;
    inst.imm.set_signed(64).unwrap();
    
    let bne = Instruction::BNE(inst);
    cpu.execute(bne).unwrap();
    assert_eq!(cpu.pc, 128, "BNE: Should branch when not equal");
}

#[test]
fn test_bne_equal() {
    let mut cpu = CPU::default();
    let mut inst = BType::default();
    
    cpu.x1 = 24;
    cpu.x2 = 24; // Equal
    cpu.pc = 0;
    
    inst.rs1 = Register::X1;
    inst.rs2 = Register::X2;
    inst.imm.set_signed(64).unwrap();
    
    let bne = Instruction::BNE(inst);
    cpu.execute(bne).unwrap();
    assert_eq!(cpu.pc, Instruction::LENGTH, 
        "BNE: Should not branch when equal");
}

#[test]
fn test_blt_signed_comparison() {
    let mut cpu = CPU::default();
    let mut inst = BType::default();
    
    inst.rs1 = Register::X1;
    inst.rs2 = Register::X2;
    inst.imm.set_signed(64).unwrap();
    
    // Test positive less than
    cpu.x1 = 23;
    cpu.x2 = 24;
    cpu.pc = 0;
    
    let blt = Instruction::BLT(inst);
    cpu.execute(blt).unwrap();
    assert_eq!(cpu.pc, 128, "BLT: 23 < 24 should branch");
    
    // Test negative less than positive
    cpu.x1 = -1i32 as u32;
    cpu.x2 = 1;
    cpu.pc = 0;
    
    cpu.execute(blt).unwrap();
    assert_eq!(cpu.pc, 128, "BLT: -1 < 1 should branch (signed)");
    
    // Test equal values
    cpu.x1 = 24;
    cpu.x2 = 24;
    cpu.pc = 0;
    
    cpu.execute(blt).unwrap();
    assert_eq!(cpu.pc, Instruction::LENGTH, "BLT: 24 < 24 should not branch");
}

#[test]
fn test_bltu_unsigned_comparison() {
    let mut cpu = CPU::default();
    let mut inst = BType::default();
    
    inst.rs1 = Register::X1;
    inst.rs2 = Register::X2;
    inst.imm.set_unsigned(64).unwrap();
    
    // Test simple unsigned comparison
    cpu.x1 = 23;
    cpu.x2 = 24;
    cpu.pc = 0;
    
    let bltu = Instruction::BLTU(inst);
    cpu.execute(bltu).unwrap();
    assert_eq!(cpu.pc, 128, "BLTU: 23 < 24 should branch");
    
    // Test unsigned comparison with "negative" number
    cpu.x1 = 1;
    cpu.x2 = -1i32 as u32; // 0xFFFFFFFF in unsigned
    cpu.pc = 0;
    
    cpu.execute(bltu).unwrap();
    assert_eq!(cpu.pc, 128, "BLTU: 1 < 0xFFFFFFFF should branch (unsigned)");
}

#[test]
fn test_bge_signed_comparison() {
    let mut cpu = CPU::default();
    let mut inst = BType::default();
    
    inst.rs1 = Register::X1;
    inst.rs2 = Register::X2;
    inst.imm.set_signed(64).unwrap();
    
    // Test greater than
    cpu.x1 = 24;
    cpu.x2 = 23;
    cpu.pc = 0;
    
    let bge = Instruction::BGE(inst);
    cpu.execute(bge).unwrap();
    assert_eq!(cpu.pc, 128, "BGE: 24 >= 23 should branch");
    
    // Test equal
    cpu.x1 = 24;
    cpu.x2 = 24;
    cpu.pc = 0;
    
    cpu.execute(bge).unwrap();
    assert_eq!(cpu.pc, 128, "BGE: 24 >= 24 should branch (equal)");
    
    // Test less than
    cpu.x1 = 23;
    cpu.x2 = 24;
    cpu.pc = 0;
    
    cpu.execute(bge).unwrap();
    assert_eq!(cpu.pc, Instruction::LENGTH, "BGE: 23 >= 24 should not branch");
}

#[test]
fn test_bgeu_unsigned_comparison() {
    let mut cpu = CPU::default();
    let mut inst = BType::default();
    
    inst.rs1 = Register::X1;
    inst.rs2 = Register::X2;
    inst.imm.set_unsigned(64).unwrap();
    
    // Test simple unsigned comparison
    cpu.x1 = 24;
    cpu.x2 = 23;
    cpu.pc = 0;
    
    let bgeu = Instruction::BGEU(inst);
    cpu.execute(bgeu).unwrap();
    assert_eq!(cpu.pc, 128, "BGEU: 24 >= 23 should branch");
    
    // Test unsigned comparison with "negative" number
    cpu.x1 = -1i32 as u32; // 0xFFFFFFFF
    cpu.x2 = 1;
    cpu.pc = 0;
    
    cpu.execute(bgeu).unwrap();
    assert_eq!(cpu.pc, 128, "BGEU: 0xFFFFFFFF >= 1 should branch (unsigned)");
}

#[test]
fn test_branch_offset_encoding() {
    // Branch offsets are encoded in multiples of 2
    let mut cpu = CPU::default();
    let mut inst = BType::default();
    
    cpu.x1 = 100;
    cpu.x2 = 100; // Equal for BEQ
    
    inst.rs1 = Register::X1;
    inst.rs2 = Register::X2;
    
    // Test various offsets
    let test_cases = [
        (0, 0),       // No branch offset
        (2, 4),       // Minimum forward branch
        (4, 8),       // Small forward branch
        (-2, -4),     // Minimum backward branch
        (-4, -8),     // Small backward branch
        (2047, 4094), // Maximum positive offset (12-bit signed)
        (-2048, -4096), // Maximum negative offset
    ];
    
    for (imm, expected_offset) in test_cases {
        cpu.pc = 1000; // Start from non-zero PC
        inst.imm.set_signed(imm).unwrap();
        
        let beq = Instruction::BEQ(inst);
        cpu.execute(beq).unwrap();
        
        let expected_pc = (1000i32 + expected_offset) as u32;
        assert_eq!(cpu.pc, expected_pc, 
            "Branch offset {} should result in PC = {}", imm, expected_pc);
    }
}

#[test]
fn test_branch_with_x0() {
    // Common pattern: comparing against zero register
    let mut cpu = CPU::default();
    let mut inst = BType::default();
    
    inst.rs1 = Register::X1;
    inst.rs2 = Register::X0; // Always zero
    inst.imm.set_signed(64).unwrap();
    
    // BEQ x1, x0, offset - branch if x1 == 0
    cpu.x1 = 0;
    cpu.pc = 0;
    let beq = Instruction::BEQ(inst);
    cpu.execute(beq).unwrap();
    assert_eq!(cpu.pc, 128, "BEQ x1, x0: Should branch when x1 is zero");
    
    // BNE x1, x0, offset - branch if x1 != 0
    cpu.x1 = 42;
    cpu.pc = 0;
    let bne = Instruction::BNE(inst);
    cpu.execute(bne).unwrap();
    assert_eq!(cpu.pc, 128, "BNE x1, x0: Should branch when x1 is non-zero");
}