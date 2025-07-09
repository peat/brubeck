//! Unit tests for arithmetic instructions (ADD, ADDI, SUB)
//! 
//! These tests verify the behavior of basic arithmetic operations as specified
//! in the RISC-V ISA manual, including overflow behavior and sign extension.

use brubeck::rv32_i::{
    cpu::CPU,
    formats::{IType, RType},
    instructions::Instruction,
    registers::Register,
};

#[test]
fn test_add_sub_basic() {
    let mut cpu = CPU::default();
    let mut rtype = RType::default();

    rtype.rd = Register::X1;
    rtype.rs1 = Register::X2;
    rtype.rs2 = Register::X3;

    let add = Instruction::ADD(rtype);
    let sub = Instruction::SUB(rtype);

    // Test zero values
    assert_eq!(cpu.x1, 0);
    assert_eq!(cpu.x2, 0);
    assert_eq!(cpu.x3, 0);

    let result = cpu.execute(add);
    assert!(result.is_ok());
    assert_eq!(cpu.x1, 0, "ADD: 0 + 0 should equal 0");

    // Test non-overflowing add and sub
    cpu.set_register(Register::X2, 8);
    cpu.set_register(Register::X3, 4);

    let result = cpu.execute(add);
    assert!(result.is_ok());
    assert_eq!(cpu.x1, 12, "ADD: 8 + 4 should equal 12");

    let result = cpu.execute(sub);
    assert!(result.is_ok());
    assert_eq!(cpu.x1, 4, "SUB: 8 - 4 should equal 4");
}

#[test]
fn test_add_sub_overflow() {
    // Test overflow behavior - RISC-V ignores overflow and keeps lower 32 bits
    let mut cpu = CPU::default();
    let mut rtype = RType::default();

    rtype.rd = Register::X1;
    rtype.rs1 = Register::X2;
    rtype.rs2 = Register::X3;

    let add = Instruction::ADD(rtype);
    let sub = Instruction::SUB(rtype);

    // Test unsigned overflow in addition
    cpu.set_register(Register::X2, 3);
    cpu.set_register(Register::X3, u32::MAX - 1);

    let result = cpu.execute(add);
    assert!(result.is_ok());
    assert_eq!(cpu.x1, 1, "ADD overflow: 3 + (2^32-2) should wrap to 1");

    // Test subtraction with overflow
    let result = cpu.execute(sub);
    assert!(result.is_ok());
    assert_eq!(cpu.x1, 5, "SUB overflow: 3 - (2^32-2) should wrap to 5");
}

#[test]
fn test_addi_basic() {
    let mut cpu = CPU::default();
    let mut inst = IType::default();

    inst.rd = Register::X1;
    inst.rs1 = Register::X1;
    inst.imm.set_unsigned(0).unwrap();

    let addi = Instruction::ADDI(inst);

    // Test adding zero
    let result = cpu.execute(addi);
    assert!(result.is_ok());
    assert_eq!(cpu.x1, 0, "ADDI: 0 + 0 should equal 0");

    // Test positive immediate
    inst.imm.set_unsigned(5).unwrap();
    let addi = Instruction::ADDI(inst);
    let result = cpu.execute(addi);
    assert!(result.is_ok());
    assert_eq!(cpu.x1, 5, "ADDI: 0 + 5 should equal 5");

    // Test negative immediate (sign-extended)
    let result = inst.imm.set_signed(-3);
    assert!(result.is_ok());
    let addi = Instruction::ADDI(inst);
    let result = cpu.execute(addi);
    assert!(result.is_ok());
    assert_eq!(cpu.x1, 2, "ADDI: 5 + (-3) should equal 2");
}

#[test]
fn test_addi_sign_extension() {
    // Test that ADDI properly sign-extends 12-bit immediates
    let mut cpu = CPU::default();
    let mut inst = IType::default();

    inst.rd = Register::X1;
    inst.rs1 = Register::X0; // Use x0 (always 0) as base

    // Test maximum positive 12-bit immediate (2047)
    inst.imm.set_signed(2047).unwrap();
    let addi = Instruction::ADDI(inst);
    cpu.execute(addi).unwrap();
    assert_eq!(cpu.x1, 2047, "ADDI: Maximum positive immediate");

    // Test minimum negative 12-bit immediate (-2048)
    inst.imm.set_signed(-2048).unwrap();
    let addi = Instruction::ADDI(inst);
    cpu.execute(addi).unwrap();
    assert_eq!(cpu.x1 as i32, -2048, "ADDI: Minimum negative immediate");
}

#[test]
fn test_x0_destination() {
    // Test that writes to x0 are ignored (x0 always reads as zero)
    let mut cpu = CPU::default();
    let mut rtype = RType::default();

    rtype.rd = Register::X0; // Destination is x0
    rtype.rs1 = Register::X1;
    rtype.rs2 = Register::X2;

    cpu.set_register(Register::X1, 100);
    cpu.set_register(Register::X2, 200);

    let add = Instruction::ADD(rtype);
    cpu.execute(add).unwrap();
    
    assert_eq!(cpu.get_register(Register::X0), 0, 
        "x0 should always read as zero, even after ADD attempt");
}