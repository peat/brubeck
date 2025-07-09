//! Unit tests for logical instructions (AND, ANDI, OR, ORI, XOR, XORI)
//!
//! These tests verify bitwise logical operations, including immediate
//! sign extension and common bit manipulation patterns.

use brubeck::rv32_i::{
    cpu::CPU,
    formats::IType,
    instructions::Instruction,
    registers::Register,
};

#[test]
fn test_andi_ori_xori_all_ones() {
    let mut cpu = CPU::default();
    let mut inst = IType::default();

    inst.rd = Register::X1;
    inst.rs1 = Register::X2;

    // Set up all 1s in register and immediate (12-bit max)
    let result = inst.imm.set_unsigned(inst.imm.unsigned_max());
    assert!(result.is_ok());
    cpu.x2 = u32::MAX;

    // ANDI with all 1s
    let andi = Instruction::ANDI(inst);
    let result = cpu.execute(andi);
    assert!(result.is_ok());
    assert_eq!(cpu.x1, u32::MAX, "ANDI: all 1s AND all 1s = all 1s");

    // ORI with all 1s
    let ori = Instruction::ORI(inst);
    let result = cpu.execute(ori);
    assert!(result.is_ok());
    assert_eq!(cpu.x1, u32::MAX, "ORI: all 1s OR all 1s = all 1s");

    // XORI with all 1s (sign-extended immediate)
    let xori = Instruction::XORI(inst);
    let result = cpu.execute(xori);
    assert!(result.is_ok());
    assert_eq!(cpu.x1, 0, "XORI: all 1s XOR all 1s = 0");
}

#[test]
fn test_andi_ori_xori_all_zeros() {
    let mut cpu = CPU::default();
    let mut inst = IType::default();

    inst.rd = Register::X1;
    inst.rs1 = Register::X2;

    // Set up all 0s in immediate, all 1s in register
    let result = inst.imm.set_unsigned(0);
    assert!(result.is_ok());
    cpu.x2 = u32::MAX;

    // ANDI with immediate = 0
    let andi = Instruction::ANDI(inst);
    let result = cpu.execute(andi);
    assert!(result.is_ok());
    assert_eq!(cpu.x1, 0, "ANDI: value AND 0 = 0");

    // ORI with immediate = 0
    let ori = Instruction::ORI(inst);
    let result = cpu.execute(ori);
    assert!(result.is_ok());
    assert_eq!(cpu.x1, u32::MAX, "ORI: value OR 0 = value");

    // XORI with immediate = 0
    let xori = Instruction::XORI(inst);
    let result = cpu.execute(xori);
    assert!(result.is_ok());
    assert_eq!(cpu.x1, u32::MAX, "XORI: value XOR 0 = value");
}

#[test]
fn test_logical_sign_extension() {
    // Test that logical operations properly sign-extend 12-bit immediates
    let mut cpu = CPU::default();
    let mut inst = IType::default();

    inst.rd = Register::X1;
    inst.rs1 = Register::X2;
    cpu.x2 = 0xFFFF_0000; // Upper 16 bits set

    // Set immediate to -1 (0xFFF in 12 bits, sign-extends to 0xFFFFFFFF)
    inst.imm.set_signed(-1).unwrap();

    // ANDI with sign-extended -1
    let andi = Instruction::ANDI(inst);
    cpu.execute(andi).unwrap();
    assert_eq!(cpu.x1, 0xFFFF_0000, 
        "ANDI: 0xFFFF0000 AND 0xFFFFFFFF = 0xFFFF0000");

    // ORI with sign-extended -1
    let ori = Instruction::ORI(inst);
    cpu.execute(ori).unwrap();
    assert_eq!(cpu.x1, 0xFFFF_FFFF, 
        "ORI: 0xFFFF0000 OR 0xFFFFFFFF = 0xFFFFFFFF");

    // Reset x2 for XOR test
    cpu.x2 = 0xFFFF_0000;
    
    // XORI with sign-extended -1
    let xori = Instruction::XORI(inst);
    cpu.execute(xori).unwrap();
    assert_eq!(cpu.x1, 0x0000_FFFF, 
        "XORI: 0xFFFF0000 XOR 0xFFFFFFFF = 0x0000FFFF");
}

#[test]
fn test_xori_bitwise_not() {
    // XORI rd, rs1, -1 performs bitwise NOT (mentioned in spec)
    let mut cpu = CPU::default();
    let mut inst = IType::default();

    inst.rd = Register::X1;
    inst.rs1 = Register::X2;
    inst.imm.set_signed(-1).unwrap(); // -1 sign-extends to all 1s

    // Test NOT of various patterns
    cpu.x2 = 0xAAAA_AAAA;
    let xori = Instruction::XORI(inst);
    cpu.execute(xori).unwrap();
    assert_eq!(cpu.x1, 0x5555_5555, 
        "XORI with -1: NOT 0xAAAAAAAA = 0x55555555");

    cpu.x2 = 0x0F0F_0F0F;
    cpu.execute(xori).unwrap();
    assert_eq!(cpu.x1, 0xF0F0_F0F0, 
        "XORI with -1: NOT 0x0F0F0F0F = 0xF0F0F0F0");
}

#[test]
fn test_logical_common_patterns() {
    let mut cpu = CPU::default();
    let mut inst = IType::default();

    inst.rd = Register::X1;
    inst.rs1 = Register::X2;

    // Masking lower 8 bits with ANDI
    cpu.x2 = 0x1234_5678;
    inst.imm.set_unsigned(0xFF).unwrap();
    let andi = Instruction::ANDI(inst);
    cpu.execute(andi).unwrap();
    assert_eq!(cpu.x1, 0x78, "ANDI: Mask lower 8 bits");

    // Setting bits with ORI
    cpu.x2 = 0x1234_5600;
    inst.imm.set_unsigned(0x78).unwrap();
    let ori = Instruction::ORI(inst);
    cpu.execute(ori).unwrap();
    assert_eq!(cpu.x1, 0x1234_5678, "ORI: Set specific bits");

    // Toggling bits with XORI
    cpu.x2 = 0x1234_5678;
    inst.imm.set_unsigned(0xF0).unwrap();
    let xori = Instruction::XORI(inst);
    cpu.execute(xori).unwrap();
    assert_eq!(cpu.x1, 0x1234_5688, "XORI: Toggle specific bits");
}