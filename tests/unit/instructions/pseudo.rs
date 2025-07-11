//! Unit tests for pseudo-instruction expansion
//!
//! These tests verify that pseudo-instructions correctly expand
//! to their base RV32I instruction equivalents.

use brubeck::rv32_i::{Instruction, PseudoInstruction, Register};

#[test]
fn test_mv_expansion() {
    let pseudo = PseudoInstruction::MV {
        rd: Register::X1,
        rs: Register::X2,
    };

    let expanded = pseudo.expand().unwrap();
    assert_eq!(expanded.len(), 1);

    match &expanded[0] {
        Instruction::ADDI(i) => {
            assert_eq!(i.rd, Register::X1);
            assert_eq!(i.rs1, Register::X2);
            assert_eq!(i.imm.as_i32(), 0);
        }
        _ => panic!("MV should expand to ADDI"),
    }
}

#[test]
fn test_ret_expansion() {
    let pseudo = PseudoInstruction::RET;
    let expanded = pseudo.expand().unwrap();

    assert_eq!(expanded.len(), 1);
    match &expanded[0] {
        Instruction::JALR(i) => {
            assert_eq!(i.rd, Register::X0);
            assert_eq!(i.rs1, Register::X1);
            assert_eq!(i.imm.as_i32(), 0);
        }
        _ => panic!("RET should expand to JALR"),
    }
}
