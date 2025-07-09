//! Unit tests for load and store instructions
//!
//! These tests verify memory access patterns, different widths (byte, halfword, word),
//! sign extension behavior, and little-endian byte ordering.

use brubeck::rv32_i::{
    cpu::CPU,
    formats::{IType, SType},
    instructions::Instruction,
    registers::Register,
};

#[test]
fn test_lw_basic() {
    let mut cpu = CPU::default();
    let mut inst = IType::default();
    
    // Set up memory with test pattern
    cpu.memory[1024] = 0x78; // LSB
    cpu.memory[1025] = 0x56;
    cpu.memory[1026] = 0x34;
    cpu.memory[1027] = 0x12; // MSB
    
    cpu.x1 = 1024; // Base address
    inst.rs1 = Register::X1;
    inst.rd = Register::X2;
    inst.imm.set_unsigned(0).unwrap(); // Zero offset
    
    let lw = Instruction::LW(inst);
    let result = cpu.execute(lw);
    assert!(result.is_ok());
    
    // Little-endian: memory bytes are loaded in order
    assert_eq!(cpu.x2, 0x12345678, 
        "LW: Should load 32-bit value in little-endian order");
}

#[test]
fn test_lw_with_offset() {
    let mut cpu = CPU::default();
    let mut inst = IType::default();
    
    // Set up memory
    cpu.memory[1024] = 0x11;
    cpu.memory[1025] = 0x22;
    cpu.memory[1026] = 0x33;
    cpu.memory[1027] = 0x44;
    cpu.memory[1028] = 0x55;
    cpu.memory[1029] = 0x66;
    
    cpu.x1 = 1024;
    inst.rs1 = Register::X1;
    inst.rd = Register::X2;
    inst.imm.set_signed(2).unwrap(); // +2 offset
    
    let lw = Instruction::LW(inst);
    cpu.execute(lw).unwrap();
    
    assert_eq!(cpu.x2, 0x66554433,
        "LW: Should load from base + offset (1024 + 2)");
}

#[test]
fn test_lh_sign_extension() {
    let mut cpu = CPU::default();
    let mut inst = IType::default();
    
    // Test positive halfword
    cpu.memory[1024] = 0x34;
    cpu.memory[1025] = 0x12; // 0x1234 (positive)
    
    cpu.x1 = 1024;
    inst.rs1 = Register::X1;
    inst.rd = Register::X2;
    inst.imm.set_unsigned(0).unwrap();
    
    let lh = Instruction::LH(inst);
    cpu.execute(lh).unwrap();
    assert_eq!(cpu.x2, 0x00001234,
        "LH: Positive halfword should be zero-extended");
    
    // Test negative halfword
    cpu.memory[1024] = 0x00;
    cpu.memory[1025] = 0x80; // 0x8000 (negative in signed 16-bit)
    
    cpu.execute(lh).unwrap();
    assert_eq!(cpu.x2, 0xFFFF8000,
        "LH: Negative halfword should be sign-extended");
}

#[test]
fn test_lhu_zero_extension() {
    let mut cpu = CPU::default();
    let mut inst = IType::default();
    
    // Test value that would be negative if sign-extended
    cpu.memory[1024] = 0xFF;
    cpu.memory[1025] = 0xFF; // 0xFFFF
    
    cpu.x1 = 1024;
    inst.rs1 = Register::X1;
    inst.rd = Register::X2;
    inst.imm.set_unsigned(0).unwrap();
    
    let lhu = Instruction::LHU(inst);
    cpu.execute(lhu).unwrap();
    assert_eq!(cpu.x2, 0x0000FFFF,
        "LHU: Should zero-extend, not sign-extend");
}

#[test]
fn test_lb_sign_extension() {
    let mut cpu = CPU::default();
    let mut inst = IType::default();
    
    cpu.x1 = 1024;
    inst.rs1 = Register::X1;
    inst.rd = Register::X2;
    inst.imm.set_unsigned(0).unwrap();
    
    // Test positive byte
    cpu.memory[1024] = 0x7F; // Maximum positive signed byte
    let lb = Instruction::LB(inst);
    cpu.execute(lb).unwrap();
    assert_eq!(cpu.x2, 0x0000007F,
        "LB: Positive byte should be zero-extended");
    
    // Test negative byte
    cpu.memory[1024] = 0x80; // Minimum negative signed byte
    cpu.execute(lb).unwrap();
    assert_eq!(cpu.x2, 0xFFFFFF80,
        "LB: Negative byte should be sign-extended");
    
    // Test -1
    cpu.memory[1024] = 0xFF;
    cpu.execute(lb).unwrap();
    assert_eq!(cpu.x2 as i32, -1,
        "LB: 0xFF should be sign-extended to -1");
}

#[test]
fn test_lbu_zero_extension() {
    let mut cpu = CPU::default();
    let mut inst = IType::default();
    
    cpu.x1 = 1024;
    inst.rs1 = Register::X1;
    inst.rd = Register::X2;
    inst.imm.set_unsigned(0).unwrap();
    
    // Test byte that would be negative if sign-extended
    cpu.memory[1024] = 0xFF;
    let lbu = Instruction::LBU(inst);
    cpu.execute(lbu).unwrap();
    assert_eq!(cpu.x2, 0x000000FF,
        "LBU: Should zero-extend to 255, not -1");
}

#[test]
fn test_sw_basic() {
    let mut cpu = CPU::default();
    let mut inst = SType::default();
    
    cpu.x1 = 100; // Base address
    cpu.x2 = 0x12345678; // Value to store
    
    inst.rs1 = Register::X1;
    inst.rs2 = Register::X2;
    inst.imm.set_unsigned(0).unwrap();
    
    let sw = Instruction::SW(inst);
    cpu.execute(sw).unwrap();
    
    // Check little-endian storage
    assert_eq!(cpu.memory[100], 0x78, "SW: LSB at lowest address");
    assert_eq!(cpu.memory[101], 0x56, "SW: Byte 1");
    assert_eq!(cpu.memory[102], 0x34, "SW: Byte 2");
    assert_eq!(cpu.memory[103], 0x12, "SW: MSB at highest address");
}

#[test]
fn test_sh_truncation() {
    let mut cpu = CPU::default();
    let mut inst = SType::default();
    
    cpu.x1 = 200;
    cpu.x2 = 0xFFFF1234; // Only lower 16 bits should be stored
    
    inst.rs1 = Register::X1;
    inst.rs2 = Register::X2;
    inst.imm.set_unsigned(0).unwrap();
    
    let sh = Instruction::SH(inst);
    cpu.execute(sh).unwrap();
    
    assert_eq!(cpu.memory[200], 0x34, "SH: Store low byte of halfword");
    assert_eq!(cpu.memory[201], 0x12, "SH: Store high byte of halfword");
    assert_eq!(cpu.memory[202], 0, "SH: Should not modify beyond halfword");
}

#[test]
fn test_sb_truncation() {
    let mut cpu = CPU::default();
    let mut inst = SType::default();
    
    cpu.x1 = 300;
    cpu.x2 = 0xFFFFFF78; // Only lowest byte should be stored
    
    inst.rs1 = Register::X1;
    inst.rs2 = Register::X2;
    inst.imm.set_unsigned(0).unwrap();
    
    let sb = Instruction::SB(inst);
    cpu.execute(sb).unwrap();
    
    assert_eq!(cpu.memory[300], 0x78, "SB: Store only lowest byte");
    assert_eq!(cpu.memory[301], 0, "SB: Should not modify next byte");
}

#[test]
fn test_sw_lw_roundtrip() {
    let mut cpu = CPU::default();
    
    cpu.x1 = 100; // Base address
    cpu.x2 = 0xDEADBEEF; // Test value
    
    // Store word
    let mut store_inst = SType::default();
    store_inst.rs1 = Register::X1;
    store_inst.rs2 = Register::X2;
    let sw = Instruction::SW(store_inst);
    cpu.execute(sw).unwrap();
    
    // Load word back
    let mut load_inst = IType::default();
    load_inst.rs1 = Register::X1;
    load_inst.rd = Register::X3;
    let lw = Instruction::LW(load_inst);
    cpu.execute(lw).unwrap();
    
    assert_eq!(cpu.x3, cpu.x2, "SW/LW: Round trip should preserve value");
}

#[test]
fn test_sh_lh_roundtrip() {
    let mut cpu = CPU::default();
    
    cpu.x1 = 100;
    cpu.x2 = 0xFFFF8765; // Test with sign bit set
    
    // Store halfword
    let mut store_inst = SType::default();
    store_inst.rs1 = Register::X1;
    store_inst.rs2 = Register::X2;
    let sh = Instruction::SH(store_inst);
    cpu.execute(sh).unwrap();
    
    // Load halfword back (signed)
    let mut load_inst = IType::default();
    load_inst.rs1 = Register::X1;
    load_inst.rd = Register::X3;
    let lh = Instruction::LH(load_inst);
    cpu.execute(lh).unwrap();
    
    assert_eq!(cpu.x3, 0xFFFF8765,
        "SH/LH: Should preserve signed 16-bit value with sign extension");
}

#[test]
fn test_sb_lb_roundtrip() {
    let mut cpu = CPU::default();
    
    cpu.x1 = 100;
    cpu.x2 = 0xFFFFFF81; // Test with sign bit set
    
    // Store byte
    let mut store_inst = SType::default();
    store_inst.rs1 = Register::X1;
    store_inst.rs2 = Register::X2;
    let sb = Instruction::SB(store_inst);
    cpu.execute(sb).unwrap();
    
    // Load byte back (signed)
    let mut load_inst = IType::default();
    load_inst.rs1 = Register::X1;
    load_inst.rd = Register::X3;
    let lb = Instruction::LB(load_inst);
    cpu.execute(lb).unwrap();
    
    assert_eq!(cpu.x3, 0xFFFFFF81,
        "SB/LB: Should preserve signed 8-bit value with sign extension");
}

#[test]
fn test_load_store_with_negative_offset() {
    let mut cpu = CPU::default();
    
    // Set up memory
    cpu.memory[96] = 0xAB;
    cpu.memory[97] = 0xCD;
    cpu.memory[98] = 0xEF;
    cpu.memory[99] = 0x12;
    
    cpu.x1 = 100; // Base address
    
    // Load with negative offset
    let mut load_inst = IType::default();
    load_inst.rs1 = Register::X1;
    load_inst.rd = Register::X2;
    load_inst.imm.set_signed(-4).unwrap(); // Access address 96
    
    let lw = Instruction::LW(load_inst);
    cpu.execute(lw).unwrap();
    
    assert_eq!(cpu.x2, 0x12EFCDAB,
        "LW: Should correctly handle negative offset");
}

#[test]
fn test_misaligned_access_behavior() {
    // Note: The current implementation doesn't enforce alignment
    // This test documents the current behavior
    let mut cpu = CPU::default();
    
    // Set up memory
    cpu.memory[1001] = 0x11;
    cpu.memory[1002] = 0x22;
    cpu.memory[1003] = 0x33;
    cpu.memory[1004] = 0x44;
    
    cpu.x1 = 1001; // Misaligned address
    
    let mut inst = IType::default();
    inst.rs1 = Register::X1;
    inst.rd = Register::X2;
    inst.imm.set_unsigned(0).unwrap();
    
    let lw = Instruction::LW(inst);
    let result = cpu.execute(lw);
    
    // Document current behavior (no alignment enforcement)
    assert!(result.is_ok(), 
        "Current implementation allows misaligned access");
    assert_eq!(cpu.x2, 0x44332211,
        "Misaligned LW loads bytes in little-endian order");
}