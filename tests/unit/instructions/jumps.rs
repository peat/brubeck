//! Unit tests for jump instructions (JAL, JALR)
//!
//! These tests verify unconditional jumps, link register behavior,
//! and alignment requirements as specified in the RISC-V ISA.

use brubeck::rv32_i::{
    cpu::CPU,
    formats::{IType, JType},
    instructions::Instruction,
    registers::Register,
};

#[test]
fn test_jal_basic() {
    let mut cpu = CPU::default();
    let mut inst = JType::default();
    
    inst.rd = Register::X1;
    inst.imm.set_unsigned(4).unwrap(); // Jump offset of 4 (will be multiplied by 2)
    
    let jal = Instruction::JAL(inst);
    let result = cpu.execute(jal);
    assert!(result.is_ok());
    
    // JAL jumps to PC + (immediate * 2)
    assert_eq!(cpu.pc, 8, "JAL: PC should be 0 + (4 * 2) = 8");
    
    // JAL stores return address (PC + 4) in rd
    assert_eq!(cpu.x1, 4, "JAL: Return address should be stored in x1");
}

#[test]
fn test_jal_misalignment() {
    // JAL requires target address to be 4-byte aligned in RV32I
    let mut cpu = CPU::default();
    let mut inst = JType::default();
    
    inst.rd = Register::X1;
    inst.imm.set_unsigned(1).unwrap(); // Offset of 1 (will become 2, misaligned)
    
    let jal = Instruction::JAL(inst);
    let result = cpu.execute(jal);
    
    assert!(result.is_err(), 
        "JAL: Should fail with misaligned target address (2 is not 4-byte aligned)");
}

#[test]
fn test_jal_negative_offset() {
    let mut cpu = CPU::default();
    let mut inst = JType::default();
    
    // Start from a higher PC to test backward jump
    cpu.pc = 1000;
    
    inst.rd = Register::X1;
    inst.imm.set_signed(-100).unwrap(); // Jump backward by 200 bytes
    
    let jal = Instruction::JAL(inst);
    cpu.execute(jal).unwrap();
    
    assert_eq!(cpu.pc, 800, "JAL: Should jump backward to 1000 + (-100 * 2) = 800");
    assert_eq!(cpu.x1, 1004, "JAL: Return address should be 1000 + 4");
}

#[test]
fn test_jal_x0_destination() {
    // JAL with rd=x0 is used for unconditional jumps without saving return address
    let mut cpu = CPU::default();
    let mut inst = JType::default();
    
    inst.rd = Register::X0;
    inst.imm.set_unsigned(10).unwrap();
    
    let jal = Instruction::JAL(inst);
    cpu.execute(jal).unwrap();
    
    assert_eq!(cpu.pc, 20, "JAL: Should jump even with x0 destination");
    assert_eq!(cpu.get_register(Register::X0), 0, "x0 should remain zero");
}

#[test]
fn test_jalr_basic() {
    let mut cpu = CPU::default();
    let mut inst = IType::default();
    
    inst.rs1 = Register::X2;
    inst.rd = Register::X1;
    inst.imm.set_unsigned(12).unwrap();
    
    // JALR jumps to rs1 + immediate
    cpu.x2 = 0; // Base address in x2
    
    let jalr = Instruction::JALR(inst);
    let result = cpu.execute(jalr);
    assert!(result.is_ok());
    
    assert_eq!(cpu.pc, 12, "JALR: PC should be x2(0) + imm(12) = 12");
    assert_eq!(cpu.x1, 4, "JALR: Return address should be stored in x1");
}

#[test]
fn test_jalr_with_base() {
    let mut cpu = CPU::default();
    let mut inst = IType::default();
    
    inst.rs1 = Register::X2;
    inst.rd = Register::X1;
    inst.imm.set_signed(-12).unwrap();
    
    cpu.pc = 0;
    cpu.x2 = 24; // Base address
    
    let jalr = Instruction::JALR(inst);
    cpu.execute(jalr).unwrap();
    
    assert_eq!(cpu.pc, 12, "JALR: PC should be x2(24) + imm(-12) = 12");
    assert_eq!(cpu.x1, 4, "JALR: Return address should be 0 + 4");
}

#[test]
fn test_jalr_least_significant_bit() {
    // JALR sets the least-significant bit of the result to zero
    let mut cpu = CPU::default();
    let mut inst = IType::default();
    
    inst.rs1 = Register::X2;
    inst.rd = Register::X1;
    inst.imm.set_unsigned(0).unwrap();
    
    // Set base to an odd address
    cpu.x2 = 13; // Odd address
    
    let jalr = Instruction::JALR(inst);
    cpu.execute(jalr).unwrap();
    
    assert_eq!(cpu.pc, 12, 
        "JALR: Should clear LSB, so 13 becomes 12");
}

#[test]
fn test_jalr_return_pattern() {
    // Common pattern: JALR x0, 0(x1) is used for function return
    let mut cpu = CPU::default();
    let mut inst = IType::default();
    
    inst.rd = Register::X0;  // No need to save return address
    inst.rs1 = Register::X1;  // Return address register
    inst.imm.set_unsigned(0).unwrap();
    
    cpu.x1 = 0x1000; // Simulated return address
    
    let jalr = Instruction::JALR(inst);
    cpu.execute(jalr).unwrap();
    
    assert_eq!(cpu.pc, 0x1000, "JALR: Should return to address in x1");
    assert_eq!(cpu.get_register(Register::X0), 0, "x0 should remain zero");
}

#[test] 
fn test_jal_jalr_call_return() {
    // Test function call and return pattern
    let mut cpu = CPU::default();
    
    // 1. JAL to function (save return address)
    let mut jal_inst = JType::default();
    jal_inst.rd = Register::X1;  // Return address register
    jal_inst.imm.set_unsigned(100).unwrap(); // Jump forward 200 bytes
    
    let jal = Instruction::JAL(jal_inst);
    cpu.execute(jal).unwrap();
    
    let return_addr = cpu.x1;
    assert_eq!(return_addr, 4, "Return address should be saved");
    assert_eq!(cpu.pc, 200, "Should jump to function");
    
    // 2. JALR to return (using saved address)
    let mut jalr_inst = IType::default();
    jalr_inst.rd = Register::X0;  // Don't save return address
    jalr_inst.rs1 = Register::X1; // Use saved return address
    jalr_inst.imm.set_unsigned(0).unwrap();
    
    let jalr = Instruction::JALR(jalr_inst);
    cpu.execute(jalr).unwrap();
    
    assert_eq!(cpu.pc, return_addr, "Should return to saved address");
}