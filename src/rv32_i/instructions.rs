//! RV32I hardware instructions
//!
//! This module contains the 47 hardware instructions defined in the RV32I
//! base integer instruction set. These are the actual instructions that the
//! CPU can execute.
//!
//! For assembly convenience, see the `pseudo_instructions` module which 
//! provides common pseudo-instructions that expand to these real instructions.

use super::*;

/// RV32I hardware instruction
///
/// This enum represents the 47 instructions in the RV32I base integer ISA.
/// These are the actual machine instructions, not pseudo-instructions.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Instruction {
    ADD(RType),
    ADDI(IType),
    AND(RType),
    ANDI(IType),
    AUIPC(UType),
    BEQ(BType),
    BGE(BType),
    BGEU(BType),
    BLT(BType),
    BLTU(BType),
    BNE(BType),
    EBREAK(IType),
    ECALL(IType),
    FENCE(IType),
    JAL(JType),
    JALR(IType),
    LB(IType),
    LBU(IType),
    LH(IType),
    LHU(IType),
    LUI(UType),
    LW(IType),
    NOP,
    OR(RType),
    ORI(IType),
    SB(SType),
    SH(SType),
    SLL(RType),
    SLLI(IType),
    SLT(RType),
    SLTI(IType),
    SLTIU(IType),
    SLTU(RType),
    SRA(RType),
    SRAI(IType),
    SRL(RType),
    SRLI(IType),
    SUB(RType),
    SW(SType),
    XOR(RType),
    XORI(IType)
}

impl Instruction {
    pub const LENGTH: u32 = 4; // 4 bytes, 32 bits
}
