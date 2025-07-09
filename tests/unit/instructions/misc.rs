//! Unit tests for miscellaneous instructions (NOP, FENCE, EBREAK, ECALL)
//!
//! These tests verify special-purpose instructions including
//! the NOP instruction and system instructions.

use brubeck::rv32_i::{
    cpu::CPU,
    instructions::Instruction,
};

#[test]
fn test_nop() {
    let mut cpu = CPU::default();
    let nop = Instruction::NOP;
    
    // NOP should only increment PC
    let initial_state = cpu.clone();
    
    let result = cpu.execute(nop);
    assert!(result.is_ok());
    assert_eq!(cpu.pc, 4, "NOP: PC should increment by 4");
    
    // Verify no other state changed
    assert_eq!(cpu.x1, initial_state.x1);
    assert_eq!(cpu.x2, initial_state.x2);
    // ... other registers remain unchanged
    
    // Execute another NOP
    let result = cpu.execute(nop);
    assert!(result.is_ok());
    assert_eq!(cpu.pc, 8, "NOP: PC should increment to 8");
}

#[test]
fn test_nop_encoding() {
    // NOP is encoded as ADDI x0, x0, 0
    // This test verifies that the NOP instruction exists
    let nop = Instruction::NOP;
    
    // Just verify it can be created
    match nop {
        Instruction::NOP => (),
        _ => panic!("NOP should be NOP variant"),
    }
}

// TODO: Add tests for FENCE, EBREAK, ECALL when implemented