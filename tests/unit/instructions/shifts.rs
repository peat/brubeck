//! Unit tests for shift instructions (SLL, SLLI, SRL, SRLI, SRA, SRAI)
//!
//! These tests verify logical and arithmetic shift operations,
//! including the 5-bit shift amount limitation and sign extension behavior.

use brubeck::rv32_i::{
    cpu::CPU,
    formats::{IType, RType},
    instructions::Instruction,
    registers::Register,
};

#[test]
fn test_sll_basic() {
    let mut cpu = CPU::default();
    let mut inst = RType::default();
    
    cpu.x1 = 0b1010; // 10
    cpu.x2 = 2;       // Shift amount
    
    inst.rd = Register::X3;
    inst.rs1 = Register::X1;
    inst.rs2 = Register::X2;
    
    let sll = Instruction::SLL(inst);
    cpu.execute(sll).unwrap();
    
    assert_eq!(cpu.x3, 0b101000, "SLL: 10 << 2 = 40");
}

#[test]
fn test_sll_5bit_mask() {
    let mut cpu = CPU::default();
    let mut inst = RType::default();
    
    cpu.x1 = 1;
    cpu.x2 = 33; // Should be masked to 1 (33 & 0x1F = 1)
    
    inst.rd = Register::X3;
    inst.rs1 = Register::X1;
    inst.rs2 = Register::X2;
    
    let sll = Instruction::SLL(inst);
    cpu.execute(sll).unwrap();
    
    assert_eq!(cpu.x3, 2, "SLL: Only lower 5 bits of shift amount used");
}

#[test]
fn test_slli_basic() {
    let mut cpu = CPU::default();
    let mut inst = IType::default();
    
    cpu.x1 = 0b1111;
    
    inst.rd = Register::X2;
    inst.rs1 = Register::X1;
    inst.imm.set_unsigned(3).unwrap(); // Shift by 3
    
    let slli = Instruction::SLLI(inst);
    cpu.execute(slli).unwrap();
    
    assert_eq!(cpu.x2, 0b1111000, "SLLI: 15 << 3 = 120");
}

#[test]
fn test_slli_max_shift() {
    let mut cpu = CPU::default();
    let mut inst = IType::default();
    
    cpu.x1 = 1;
    
    inst.rd = Register::X2;
    inst.rs1 = Register::X1;
    inst.imm.set_unsigned(31).unwrap(); // Maximum shift
    
    let slli = Instruction::SLLI(inst);
    cpu.execute(slli).unwrap();
    
    assert_eq!(cpu.x2, 0x80000000, "SLLI: 1 << 31 = 0x80000000");
}

#[test]
fn test_srl_basic() {
    let mut cpu = CPU::default();
    let mut inst = RType::default();
    
    cpu.x1 = 0b111100; // 60
    cpu.x2 = 2;
    
    inst.rd = Register::X3;
    inst.rs1 = Register::X1;
    inst.rs2 = Register::X2;
    
    let srl = Instruction::SRL(inst);
    cpu.execute(srl).unwrap();
    
    assert_eq!(cpu.x3, 0b1111, "SRL: 60 >> 2 = 15");
}

#[test]
fn test_srl_no_sign_extension() {
    let mut cpu = CPU::default();
    let mut inst = RType::default();
    
    cpu.x1 = 0x80000000; // MSB set
    cpu.x2 = 1;
    
    inst.rd = Register::X3;
    inst.rs1 = Register::X1;
    inst.rs2 = Register::X2;
    
    let srl = Instruction::SRL(inst);
    cpu.execute(srl).unwrap();
    
    assert_eq!(cpu.x3, 0x40000000, 
        "SRL: Logical shift fills with zeros, not sign bit");
}

#[test]
fn test_srli_basic() {
    let mut cpu = CPU::default();
    let mut inst = IType::default();
    
    cpu.x1 = 0xFF000000;
    
    inst.rd = Register::X2;
    inst.rs1 = Register::X1;
    inst.imm.set_unsigned(8).unwrap();
    
    let srli = Instruction::SRLI(inst);
    cpu.execute(srli).unwrap();
    
    assert_eq!(cpu.x2, 0x00FF0000, "SRLI: Shifts right logically");
}

#[test]
fn test_sra_positive_number() {
    let mut cpu = CPU::default();
    let mut inst = RType::default();
    
    cpu.x1 = 0x7FFFFFFF; // Maximum positive number
    cpu.x2 = 4;
    
    inst.rd = Register::X3;
    inst.rs1 = Register::X1;
    inst.rs2 = Register::X2;
    
    let sra = Instruction::SRA(inst);
    cpu.execute(sra).unwrap();
    
    assert_eq!(cpu.x3, 0x07FFFFFF, 
        "SRA: Positive number shifts with zero fill");
}

#[test]
fn test_sra_negative_number() {
    let mut cpu = CPU::default();
    let mut inst = RType::default();
    
    cpu.x1 = 0x80000000; // Minimum negative number
    cpu.x2 = 4;
    
    inst.rd = Register::X3;
    inst.rs1 = Register::X1;
    inst.rs2 = Register::X2;
    
    let sra = Instruction::SRA(inst);
    cpu.execute(sra).unwrap();
    
    assert_eq!(cpu.x3, 0xF8000000, 
        "SRA: Negative number shifts with sign extension");
}

#[test]
fn test_sra_sign_extension_pattern() {
    let mut cpu = CPU::default();
    let mut inst = RType::default();
    
    cpu.x1 = 0xFFFF0000; // Negative with pattern
    cpu.x2 = 8;
    
    inst.rd = Register::X3;
    inst.rs1 = Register::X1;
    inst.rs2 = Register::X2;
    
    let sra = Instruction::SRA(inst);
    cpu.execute(sra).unwrap();
    
    assert_eq!(cpu.x3, 0xFFFFFF00, 
        "SRA: Sign bit propagates through shift");
}

#[test]
fn test_srai_basic() {
    let mut cpu = CPU::default();
    let mut inst = IType::default();
    
    cpu.x1 = -1024i32 as u32; // Negative number
    
    inst.rd = Register::X2;
    inst.rs1 = Register::X1;
    inst.imm.set_unsigned(2).unwrap();
    
    let srai = Instruction::SRAI(inst);
    cpu.execute(srai).unwrap();
    
    assert_eq!(cpu.x2 as i32, -256, 
        "SRAI: -1024 >> 2 = -256 (arithmetic)");
}

#[test]
fn test_shift_by_zero() {
    let mut cpu = CPU::default();
    let mut inst = RType::default();
    
    cpu.x1 = 0x12345678;
    cpu.x2 = 0; // No shift
    
    inst.rd = Register::X3;
    inst.rs1 = Register::X1;
    inst.rs2 = Register::X2;
    
    // Test all shift operations with zero
    let sll = Instruction::SLL(inst);
    cpu.execute(sll).unwrap();
    assert_eq!(cpu.x3, cpu.x1, "SLL by 0 preserves value");
    
    let srl = Instruction::SRL(inst);
    cpu.execute(srl).unwrap();
    assert_eq!(cpu.x3, cpu.x1, "SRL by 0 preserves value");
    
    let sra = Instruction::SRA(inst);
    cpu.execute(sra).unwrap();
    assert_eq!(cpu.x3, cpu.x1, "SRA by 0 preserves value");
}

#[test]
fn test_shift_edge_cases() {
    let mut cpu = CPU::default();
    let mut inst = IType::default();
    
    // Shift 1 by maximum amount
    cpu.x1 = 1;
    inst.rd = Register::X2;
    inst.rs1 = Register::X1;
    inst.imm.set_unsigned(31).unwrap();
    
    let slli = Instruction::SLLI(inst);
    cpu.execute(slli).unwrap();
    assert_eq!(cpu.x2, 0x80000000, "SLLI: 1 << 31");
    
    // Shift MSB by maximum amount
    cpu.x1 = 0x80000000;
    let srli = Instruction::SRLI(inst);
    cpu.execute(srli).unwrap();
    assert_eq!(cpu.x2, 1, "SRLI: 0x80000000 >> 31 = 1");
    
    // Arithmetic shift of -1
    cpu.x1 = 0xFFFFFFFF;
    let srai = Instruction::SRAI(inst);
    cpu.execute(srai).unwrap();
    assert_eq!(cpu.x2, 0xFFFFFFFF, "SRAI: -1 >> 31 = -1");
}

#[test]
fn test_shift_chain() {
    let mut cpu = CPU::default();
    
    // Start with a value
    cpu.x1 = 0x0F; // 15
    
    // Shift left by 4
    let mut inst = IType::default();
    inst.rd = Register::X2;
    inst.rs1 = Register::X1;
    inst.imm.set_unsigned(4).unwrap();
    
    let slli = Instruction::SLLI(inst);
    cpu.execute(slli).unwrap();
    assert_eq!(cpu.x2, 0xF0, "Step 1: 15 << 4 = 240");
    
    // Shift right logically by 2
    inst.rd = Register::X3;
    inst.rs1 = Register::X2;
    inst.imm.set_unsigned(2).unwrap();
    
    let srli = Instruction::SRLI(inst);
    cpu.execute(srli).unwrap();
    assert_eq!(cpu.x3, 0x3C, "Step 2: 240 >> 2 = 60");
}