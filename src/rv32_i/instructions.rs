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
    XORI(IType),

    // CSR Instructions (Control and Status Register)
    CSRRW(IType),
    CSRRS(IType),
    CSRRC(IType),
    CSRRWI(IType),
    CSRRSI(IType),
    CSRRCI(IType),
}

impl Instruction {
    pub const LENGTH: u32 = 4; // 4 bytes, 32 bits

    /// Returns the mnemonic (instruction name) as a string
    pub fn mnemonic(&self) -> &'static str {
        match self {
            Instruction::ADD(_) => "ADD",
            Instruction::ADDI(_) => "ADDI",
            Instruction::AND(_) => "AND",
            Instruction::ANDI(_) => "ANDI",
            Instruction::AUIPC(_) => "AUIPC",
            Instruction::BEQ(_) => "BEQ",
            Instruction::BGE(_) => "BGE",
            Instruction::BGEU(_) => "BGEU",
            Instruction::BLT(_) => "BLT",
            Instruction::BLTU(_) => "BLTU",
            Instruction::BNE(_) => "BNE",
            Instruction::EBREAK(_) => "EBREAK",
            Instruction::ECALL(_) => "ECALL",
            Instruction::FENCE(_) => "FENCE",
            Instruction::JAL(_) => "JAL",
            Instruction::JALR(_) => "JALR",
            Instruction::LB(_) => "LB",
            Instruction::LBU(_) => "LBU",
            Instruction::LH(_) => "LH",
            Instruction::LHU(_) => "LHU",
            Instruction::LUI(_) => "LUI",
            Instruction::LW(_) => "LW",
            Instruction::NOP => "NOP",
            Instruction::OR(_) => "OR",
            Instruction::ORI(_) => "ORI",
            Instruction::SB(_) => "SB",
            Instruction::SH(_) => "SH",
            Instruction::SLL(_) => "SLL",
            Instruction::SLLI(_) => "SLLI",
            Instruction::SLT(_) => "SLT",
            Instruction::SLTI(_) => "SLTI",
            Instruction::SLTIU(_) => "SLTIU",
            Instruction::SLTU(_) => "SLTU",
            Instruction::SRA(_) => "SRA",
            Instruction::SRAI(_) => "SRAI",
            Instruction::SRL(_) => "SRL",
            Instruction::SRLI(_) => "SRLI",
            Instruction::SUB(_) => "SUB",
            Instruction::SW(_) => "SW",
            Instruction::XOR(_) => "XOR",
            Instruction::XORI(_) => "XORI",
            Instruction::CSRRW(_) => "CSRRW",
            Instruction::CSRRS(_) => "CSRRS",
            Instruction::CSRRC(_) => "CSRRC",
            Instruction::CSRRWI(_) => "CSRRWI",
            Instruction::CSRRSI(_) => "CSRRSI",
            Instruction::CSRRCI(_) => "CSRRCI",
        }
    }
}
