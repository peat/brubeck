//! Unit tests for upper immediate instructions (LUI, AUIPC)
//!
//! These tests verify the loading of 20-bit immediates into the upper
//! portion of registers and PC-relative address calculations.

use brubeck::rv32_i::{
    cpu::CPU,
    formats::UType,
    instructions::Instruction,
    registers::Register,
};

#[test]
fn test_lui_basic() {
    let mut cpu = CPU::default();
    let mut inst = UType::default();

    inst.rd = Register::X1;
    let result = inst.imm.set_unsigned(1);
    assert!(result.is_ok());

    let lui = Instruction::LUI(inst);
    let result = cpu.execute(lui);
    assert!(result.is_ok());
    assert_eq!(cpu.x1, 0b0000_0000_0000_0000_0001_0000_0000_0000,
        "LUI: Should place immediate in upper 20 bits");
}

#[test]
fn test_lui_edge_cases() {
    let mut cpu = CPU::default();
    let mut inst = UType::default();
    inst.rd = Register::X1;

    // Test maximum 20-bit value
    inst.imm.set_unsigned(0xFFFFF).unwrap();
    let lui = Instruction::LUI(inst);
    cpu.execute(lui).unwrap();
    assert_eq!(cpu.x1, 0xFFFFF000,
        "LUI: Maximum immediate should fill upper 20 bits");

    // Test zero value
    inst.imm.set_unsigned(0).unwrap();
    let lui = Instruction::LUI(inst);
    cpu.execute(lui).unwrap();
    assert_eq!(cpu.x1, 0,
        "LUI: Zero immediate should result in zero");
}

#[test]
fn test_auipc_basic() {
    let mut cpu = CPU::default();
    let mut inst = UType::default();

    inst.rd = Register::X1;
    let result = inst.imm.set_unsigned(1);
    assert!(result.is_ok());

    // Execute from PC = 0
    let auipc = Instruction::AUIPC(inst);
    let result = cpu.execute(auipc);
    assert!(result.is_ok());
    assert_eq!(cpu.x1, 0b0000_0000_0000_0000_0001_0000_0000_0000,
        "AUIPC: Should add immediate<<12 to PC");
}

#[test]
fn test_auipc_with_pc() {
    let mut cpu = CPU::default();
    let mut inst = UType::default();

    inst.rd = Register::X1;
    inst.imm.set_unsigned(0x12345).unwrap();

    // Set PC to a non-zero value
    cpu.pc = 0x1000;

    let auipc = Instruction::AUIPC(inst);
    cpu.execute(auipc).unwrap();
    
    let expected = 0x1000 + (0x12345 << 12);
    assert_eq!(cpu.x1, expected,
        "AUIPC: Should correctly add (imm<<12) to current PC");
}

#[test]
fn test_auipc_pc_relative_addressing() {
    // AUIPC is used with JALR or load/store for PC-relative addressing
    // This test demonstrates the pattern
    let mut cpu = CPU::default();
    let mut inst = UType::default();

    inst.rd = Register::X1;
    
    // Simulate addressing something 0x2000 bytes ahead
    // We need to account for the 12 lower bits that will be added by the next instruction
    // If we want to address PC + 0x2000, and the lower 12 bits are 0, we use:
    // AUIPC x1, 0x2 (loads PC + 0x2000 into x1)
    inst.imm.set_unsigned(0x2).unwrap();
    
    cpu.pc = 0x1000;
    let auipc = Instruction::AUIPC(inst);
    cpu.execute(auipc).unwrap();
    
    assert_eq!(cpu.x1, 0x1000 + 0x2000,
        "AUIPC: Common pattern for PC-relative addressing");
}

#[test]
fn test_lui_auipc_x0_destination() {
    // Test that LUI/AUIPC to x0 still executes (may have side effects on PC)
    let mut cpu = CPU::default();
    let mut inst = UType::default();

    inst.rd = Register::X0; // Destination is x0
    inst.imm.set_unsigned(0x12345).unwrap();

    let initial_pc = cpu.pc;
    
    let lui = Instruction::LUI(inst);
    cpu.execute(lui).unwrap();
    assert_eq!(cpu.get_register(Register::X0), 0,
        "x0 should remain zero after LUI");
    assert_eq!(cpu.pc, initial_pc + Instruction::LENGTH,
        "PC should still advance after LUI to x0");

    let auipc = Instruction::AUIPC(inst);
    cpu.execute(auipc).unwrap();
    assert_eq!(cpu.get_register(Register::X0), 0,
        "x0 should remain zero after AUIPC");
}