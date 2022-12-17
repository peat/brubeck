#[derive(Debug, Copy, Clone)]
pub enum Register {
    X0,
    X1,
    X2,
    X3,
    X4,
    X5,
    X6,
    X7,
    X8,
    X9,
    X10,
    X11,
    X12,
    X13,
    X14,
    X15,
    X16,
    X17,
    X18,
    X19,
    X20,
    X21,
    X22,
    X23,
    X24,
    X25,
    X26,
    X27,
    X28,
    X29,
    X30,
    X31,
    PC,
}

impl Default for Register {
    fn default() -> Self {
        Register::X0
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ABI {
    Zero,
    RA,
    SP,
    GP,
    TP,
    T0,
    T1,
    T2,
    S0,
    FP,
    S1,
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    S8,
    S9,
    S10,
    S11,
    T3,
    T4,
    T5,
    T6,
}

impl ABI {
    pub fn to_r(&self) -> Register {
        match self {
            Self::Zero => Register::X0,
            Self::RA => Register::X1,
            Self::SP => Register::X2,
            Self::GP => Register::X3,
            Self::TP => Register::X4,
            Self::T0 => Register::X5,
            Self::T1 => Register::X6,
            Self::T2 => Register::X7,
            Self::S0 => Register::X8,
            Self::FP => Register::X8,
            Self::S1 => Register::X9,
            Self::A0 => Register::X10,
            Self::A1 => Register::X11,
            Self::A2 => Register::X12,
            Self::A3 => Register::X13,
            Self::A4 => Register::X14,
            Self::A5 => Register::X15,
            Self::A6 => Register::X16,
            Self::A7 => Register::X17,
            Self::S2 => Register::X18,
            Self::S3 => Register::X19,
            Self::S4 => Register::X20,
            Self::S5 => Register::X21,
            Self::S6 => Register::X22,
            Self::S7 => Register::X23,
            Self::S8 => Register::X24,
            Self::S9 => Register::X25,
            Self::S10 => Register::X26,
            Self::S11 => Register::X27,
            Self::T3 => Register::X28,
            Self::T4 => Register::X29,
            Self::T5 => Register::X30,
            Self::T6 => Register::X31,
        }
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct CPU {
    pub x0: u32,
    pub x1: u32,
    pub x2: u32,
    pub x3: u32,
    pub x4: u32,
    pub x5: u32,
    pub x6: u32,
    pub x7: u32,
    pub x8: u32,
    pub x9: u32,
    pub x10: u32,
    pub x11: u32,
    pub x12: u32,
    pub x13: u32,
    pub x14: u32,
    pub x15: u32,
    pub x16: u32,
    pub x17: u32,
    pub x18: u32,
    pub x19: u32,
    pub x20: u32,
    pub x21: u32,
    pub x22: u32,
    pub x23: u32,
    pub x24: u32,
    pub x25: u32,
    pub x26: u32,
    pub x27: u32,
    pub x28: u32,
    pub x29: u32,
    pub x30: u32,
    pub x31: u32,
    pub pc: u32,
}

impl CPU {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_register(&self, r: Register) -> u32 {
        match r {
            Register::X0 => self.x0,
            Register::X1 => self.x1,
            Register::X2 => self.x2,
            Register::X3 => self.x3,
            Register::X4 => self.x4,
            Register::X5 => self.x5,
            Register::X6 => self.x6,
            Register::X7 => self.x7,
            Register::X8 => self.x8,
            Register::X9 => self.x9,
            Register::X10 => self.x10,
            Register::X11 => self.x11,
            Register::X12 => self.x12,
            Register::X13 => self.x13,
            Register::X14 => self.x14,
            Register::X15 => self.x15,
            Register::X16 => self.x16,
            Register::X17 => self.x17,
            Register::X18 => self.x18,
            Register::X19 => self.x19,
            Register::X20 => self.x20,
            Register::X21 => self.x21,
            Register::X22 => self.x22,
            Register::X23 => self.x23,
            Register::X24 => self.x24,
            Register::X25 => self.x25,
            Register::X26 => self.x26,
            Register::X27 => self.x27,
            Register::X28 => self.x28,
            Register::X29 => self.x29,
            Register::X30 => self.x30,
            Register::X31 => self.x31,
            Register::PC => self.pc,
        }
    }

    pub fn set_register(&mut self, r: Register, v: u32) {
        match r {
            Register::X0 => self.x0 = v,
            Register::X1 => self.x1 = v,
            Register::X2 => self.x2 = v,
            Register::X3 => self.x3 = v,
            Register::X4 => self.x4 = v,
            Register::X5 => self.x5 = v,
            Register::X6 => self.x6 = v,
            Register::X7 => self.x7 = v,
            Register::X8 => self.x8 = v,
            Register::X9 => self.x9 = v,
            Register::X10 => self.x10 = v,
            Register::X11 => self.x11 = v,
            Register::X12 => self.x12 = v,
            Register::X13 => self.x13 = v,
            Register::X14 => self.x14 = v,
            Register::X15 => self.x15 = v,
            Register::X16 => self.x16 = v,
            Register::X17 => self.x17 = v,
            Register::X18 => self.x18 = v,
            Register::X19 => self.x19 = v,
            Register::X20 => self.x20 = v,
            Register::X21 => self.x21 = v,
            Register::X22 => self.x22 = v,
            Register::X23 => self.x23 = v,
            Register::X24 => self.x24 = v,
            Register::X25 => self.x25 = v,
            Register::X26 => self.x26 = v,
            Register::X27 => self.x27 = v,
            Register::X28 => self.x28 = v,
            Register::X29 => self.x29 = v,
            Register::X30 => self.x30 = v,
            Register::X31 => self.x31 = v,
            Register::PC => self.pc = v,
        }
    }

    pub fn get(&self, abi: ABI) -> u32 {
        self.get_register(abi.to_r())
    }

    pub fn set(&mut self, abi: ABI, v: u32) {
        self.set_register(abi.to_r(), v)
    }

    pub fn execute(&mut self, instruction: RV32I) -> Result<(), Error> {
        match instruction {
            RV32I::ADD(i) => self.rv32i_add(i),
            RV32I::ADDI(i) => self.rv32i_addi(i),
            RV32I::NOP => self.rv32i_nop(),
            e => Err(Error::NotImplemented(e)),
        }
    }

    fn rv32i_nop(&mut self) -> Result<(), Error> {
        self.pc += RV32I::LENGTH;
        Ok(())
    }

    fn rv32i_add(&mut self, instruction: RType) -> Result<(), Error> {
        let a = self.get_register(instruction.rs1);
        let b = self.get_register(instruction.rs2);
        self.set_register(instruction.rd, a.wrapping_add(b));
        self.pc += RV32I::LENGTH;
        Ok(())
    }

    fn rv32i_addi(&mut self, instruction: IType) -> Result<(), Error> {
        // the immediate value is a 12-bit sign extended value. here be dragons.
        let immediate = instruction.imm;

        // determine if negative
        let sign_mask: u16 = 1 << 11;
        let is_negative = (immediate & sign_mask) != 0;

        // clear the top bits, including the signing bit
        let value_mask: u16 = 0b0000_0111_1111_1111;
        let immediate_value = (value_mask & immediate) as u32;

        let current_value = self.get_register(instruction.rs1);

        let new_value = if is_negative {
            current_value.wrapping_sub(immediate_value)
        } else {
            current_value.wrapping_add(immediate_value)
        };

        self.set_register(instruction.rd, new_value);
        self.pc += RV32I::LENGTH;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Error {
    NotImplemented(RV32I),
    ImmediateValueOutOfRange(i16),
}

#[derive(Debug, Copy, Clone, Default)]
pub struct RType {
    pub opcode: u8,
    pub rd: Register,
    pub funct3: u8,
    pub rs1: Register,
    pub rs2: Register,
    pub funct7: u8,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct IType {
    pub opcode: u8,
    pub rd: Register,
    pub funct3: u8,
    pub rs1: Register,
    pub imm: u16,
}

impl IType {
    pub fn set_imm(&mut self, value: i16) -> Result<(), Error> {
        if !(-2047..=2047).contains(&value) {
            return Err(Error::ImmediateValueOutOfRange(value));
        }

        let mut imm_value = value.unsigned_abs();

        if value < 0 {
            // set sign on bit 12
            imm_value += 1 << 11;
        }

        self.imm = imm_value;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SType {
    pub opcode: u8,
    pub imm: u16,
    pub funct3: u8,
    pub rs1: Register,
    pub rs2: Register,
}

#[derive(Debug, Copy, Clone)]
pub struct BType {
    pub opcode: u8,
    pub imm: u16,
    pub funct3: u8,
    pub rs1: Register,
    pub rs2: Register,
}

#[derive(Debug, Copy, Clone)]
pub struct UType {
    pub opcode: u8,
    pub rd: Register,
    pub imm: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct JType {
    pub opcode: u8,
    pub rd: Register,
    pub imm: u32,
}

#[derive(Debug, Copy, Clone)]
pub enum RV32I {
    LUI(UType),
    AUIPC(UType),
    JAL(JType),
    JALR(IType),
    BEQ(BType),
    BNE(BType),
    BLT(BType),
    BGE(BType),
    BLTU(BType),
    BGEU(BType),
    LB(IType),
    LH(IType),
    LW(IType),
    LBU(IType),
    LHU(IType),
    SB(SType),
    SH(SType),
    SW(SType),
    ADDI(IType),
    SLTI(IType),
    SLTIU(IType),
    XORI(IType),
    ORI(IType),
    ANDI(IType),
    SLLI(IType),
    SRLI(IType),
    SRAI(IType),
    ADD(RType),
    SUB(RType),
    SLL(RType),
    SLT(RType),
    SLTU(RType),
    XOR(RType),
    SRL(RType),
    SRA(RType),
    OR(RType),
    AND(RType),
    FENCE(IType),
    FENCEI(IType),
    ECALL(IType),
    EBREAK(IType),
    CSRRW(IType),
    CSRRS(IType),
    CSRRC(IType),
    CSRRWI(IType),
    CSRRSI(IType),
    CSRRCI(IType),
    NOP,
}

impl RV32I {
    const LENGTH: u32 = 4; // 4 bytes, 32 bits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rv32i_nop() {
        let mut cpu = CPU::new();
        let nop = RV32I::NOP;

        // start from zero in the PC
        let result = cpu.execute(nop);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, 4);

        // incrementing PC
        let result = cpu.execute(nop);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, 8);
    }

    #[test]
    fn rv32i_add() {
        let mut cpu = CPU::new();
        let mut inst = RType::default();

        inst.rd = Register::X1;
        inst.rs1 = Register::X2;
        inst.rs2 = Register::X3;

        let add = RV32I::ADD(inst);

        // zero values
        let result = cpu.execute(add);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 0);

        // non-overflowing addition
        cpu.set_register(Register::X2, 4);
        cpu.set_register(Register::X3, 8);
        let result = cpu.execute(add);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 12);

        // overflowing addition
        cpu.set_register(Register::X2, u32::MAX - 1);
        cpu.set_register(Register::X3, 3);
        let result = cpu.execute(add);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 1);
    }

    #[test]
    fn rv32i_addi() {
        let mut cpu = CPU::new();
        let mut inst = IType::default();

        inst.rd = Register::X1;
        inst.rs1 = Register::X1;
        inst.imm = 0;

        let addi = RV32I::ADDI(inst);

        // zero value
        let result = cpu.execute(addi);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 0);

        // positive values
        inst.imm = 5;
        let addi = RV32I::ADDI(inst);
        let result = cpu.execute(addi);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 5);

        // negative values
        let result = inst.set_imm(-3);
        assert!(result.is_ok());
        let addi = RV32I::ADDI(inst);
        let result = cpu.execute(addi);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 2);
    }
}
