//! Unit tests for comparison instructions (SLT, SLTI, SLTU, SLTIU)
//!
//! These tests verify signed and unsigned comparison behavior, including
//! edge cases around sign extension and boundary values.

use brubeck::rv32_i::{
    cpu::CPU,
    formats::IType,
    instructions::Instruction,
    registers::Register,
};

#[test]
fn test_slti_signed_comparison() {
    let mut cpu = CPU::default();
    let mut inst = IType::default();

    inst.rd = Register::X1;
    inst.rs1 = Register::X2;
    inst.imm.set_unsigned(0).unwrap();

    let slti = Instruction::SLTI(inst);

    // Test equal values (0 == 0)
    let result = cpu.execute(slti);
    assert!(result.is_ok());
    assert_eq!(cpu.x1, 0, "SLTI: 0 < 0 should be false (0)");
    assert_eq!(cpu.pc, Instruction::LENGTH);

    // Test rs1 < immediate (0 < 1)
    inst.imm.set_signed(1).unwrap();
    let slti = Instruction::SLTI(inst);
    let result = cpu.execute(slti);
    assert!(result.is_ok());
    assert_eq!(cpu.x1, 1, "SLTI: 0 < 1 should be true (1)");
    assert_eq!(cpu.pc, Instruction::LENGTH * 2);

    // Test rs1 > immediate (0 > -1)
    inst.imm.set_signed(-1).unwrap();
    let slti = Instruction::SLTI(inst);
    let result = cpu.execute(slti);
    assert!(result.is_ok());
    assert_eq!(cpu.x1, 0, "SLTI: 0 < -1 should be false (0)");
    assert_eq!(cpu.pc, Instruction::LENGTH * 3);
}

#[test]
fn test_slti_edge_cases() {
    let mut cpu = CPU::default();
    let mut inst = IType::default();

    inst.rd = Register::X1;
    inst.rs1 = Register::X2;

    // Test with maximum positive value in rs1
    cpu.set_register(Register::X2, i32::MAX as u32);
    inst.imm.set_signed(-1).unwrap();
    let slti = Instruction::SLTI(inst);
    cpu.execute(slti).unwrap();
    assert_eq!(cpu.x1, 0, "SLTI: MAX_INT < -1 should be false");

    // Test with minimum negative value in rs1
    cpu.set_register(Register::X2, i32::MIN as u32);
    inst.imm.set_signed(0).unwrap();
    let slti = Instruction::SLTI(inst);
    cpu.execute(slti).unwrap();
    assert_eq!(cpu.x1, 1, "SLTI: MIN_INT < 0 should be true");
}

#[test]
fn test_sltiu_unsigned_comparison() {
    let mut cpu = CPU::default();
    let mut inst = IType::default();

    cpu.x2 = 255; // Initial value to compare against

    inst.rd = Register::X1;
    inst.rs1 = Register::X2;

    // Test equal values (255 == 255)
    inst.imm.set_unsigned(255).unwrap();
    let sltiu = Instruction::SLTIU(inst);
    let result = cpu.execute(sltiu);
    assert!(result.is_ok());
    assert_eq!(cpu.x1, 0, "SLTIU: 255 < 255 should be false (0)");
    assert_eq!(cpu.pc, Instruction::LENGTH);

    // Test rs1 < immediate (255 < 256)
    inst.imm.set_unsigned(256).unwrap();
    let sltiu = Instruction::SLTIU(inst);
    let result = cpu.execute(sltiu);
    assert!(result.is_ok());
    assert_eq!(cpu.x1, 1, "SLTIU: 255 < 256 should be true (1)");
    assert_eq!(cpu.pc, Instruction::LENGTH * 2);

    // Test rs1 > immediate (255 > 254)
    inst.imm.set_unsigned(254).unwrap();
    let sltiu = Instruction::SLTIU(inst);
    let result = cpu.execute(sltiu);
    assert!(result.is_ok());
    assert_eq!(cpu.x1, 0, "SLTIU: 255 < 254 should be false (0)");
    assert_eq!(cpu.pc, Instruction::LENGTH * 3);
}

#[test]
fn test_sltiu_sign_extension() {
    // SLTIU sign-extends the immediate but treats both values as unsigned for comparison
    let mut cpu = CPU::default();
    let mut inst = IType::default();

    inst.rd = Register::X1;
    inst.rs1 = Register::X2;

    cpu.set_register(Register::X2, 1);

    // Set immediate to -1, which sign-extends to 0xFFFFFFFF
    inst.imm.set_signed(-1).unwrap();
    let sltiu = Instruction::SLTIU(inst);
    cpu.execute(sltiu).unwrap();
    
    assert_eq!(cpu.x1, 1, 
        "SLTIU: 1 < 0xFFFFFFFF (unsigned) should be true");
}

#[test]
fn test_slt_sltu_register() {
    // Note: These tests are placeholders for when SLT/SLTU (R-type) are migrated
    // Currently only the immediate versions (SLTI/SLTIU) are tested here
    
    // TODO: Add tests for SLT (signed register comparison)
    // TODO: Add tests for SLTU (unsigned register comparison)
}