use super::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Instruction {
    // ✅ indicates it's implemented, not verified!
    ADD(RType),   // ✅
    ADDI(IType),  // ✅
    AND(RType),   // ✅
    ANDI(IType),  // ✅
    AUIPC(UType), // ✅
    BEQ(BType),   // ✅
    BGE(BType),   // ✅
    BGEU(BType),  // ✅
    BLT(BType),   // ✅
    BLTU(BType),  // ✅
    BNE(BType),   // ✅
    EBREAK(IType),
    ECALL(IType),
    FENCE(IType),
    JAL(JType),   // ✅
    JALR(IType),  // ✅
    LB(IType),    // ✅
    LBU(IType),   // ✅
    LH(IType),    // ✅
    LHU(IType),   // ✅
    LUI(UType),   // ✅
    LW(IType),    // ✅
    NOP,          // ✅
    OR(RType),    // ✅
    ORI(IType),   // ✅
    SB(SType),    // ✅
    SH(SType),    // ✅
    SLL(RType),   // ✅
    SLLI(IType),  // ✅
    SLT(RType),   // ✅
    SLTI(IType),  // ✅
    SLTIU(IType), // ✅
    SLTU(RType),  // ✅
    SRA(RType),   // ✅
    SRAI(IType),  // ✅
    SRL(RType),   // ✅
    SRLI(IType),  // ✅
    SUB(RType),   // ✅
    SW(SType),    // ✅
    XOR(RType),   // ✅
    XORI(IType),  // ✅
}

impl Instruction {
    pub const LENGTH: u32 = 4; // 4 bytes, 32 bits
}
