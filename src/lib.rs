mod immediate;
use immediate::Immediate;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
            RV32I::NOP => self.rv32i_nop(),
            RV32I::ADDI(i) => self.rv32i_addi(i),
            RV32I::SLTI(i) => self.rv32i_slti(i),
            RV32I::SLTIU(i) => self.rv32i_sltiu(i),
            RV32I::ANDI(i) => self.rv32i_andi(i),
            RV32I::ORI(i) => self.rv32i_ori(i),
            RV32I::XORI(i) => self.rv32i_xori(i),
            RV32I::LUI(i) => self.rv32i_lui(i),
            RV32I::AUIPC(i) => self.rv32i_auipc(i),
            RV32I::ADD(i) => self.rv32i_add(i),
            RV32I::SUB(i) => self.rv32i_sub(i),
            RV32I::SLT(i) => self.rv32i_slt(i),
            RV32I::SLTU(i) => self.rv32i_sltu(i),
            RV32I::AND(i) => self.rv32i_and(i),
            RV32I::OR(i) => self.rv32i_or(i),
            RV32I::XOR(i) => self.rv32i_xor(i),
            RV32I::SLL(i) => self.rv32i_sll(i),
            RV32I::SRL(i) => self.rv32i_srl(i),
            RV32I::SRA(i) => self.rv32i_sra(i),
            RV32I::JAL(i) => self.rv32i_jal(i),
            RV32I::JALR(i) => self.rv32i_jalr(i),
            RV32I::BEQ(i) => self.rv32i_beq(i),
            RV32I::BNE(i) => self.rv32i_bne(i),
            RV32I::BLT(i) => self.rv32i_blt(i),
            RV32I::BLTU(i) => self.rv32i_bltu(i),
            RV32I::BGE(i) => self.rv32i_bge(i),
            RV32I::BGEU(i) => self.rv32i_bgeu(i),
            e => Err(Error::NotImplemented(e)),
        }?;

        Ok(())
    }

    fn rv32i_nop(&mut self) -> Result<(), Error> {
        self.pc += RV32I::LENGTH;
        Ok(())
    }

    /// ADD and SUB perform addition and subtraction respectively. Overflows
    /// are ignored and the low XLEN bits of results are written to the
    /// destination.
    fn rv32i_add(&mut self, instruction: RType) -> Result<(), Error> {
        let a = self.get_register(instruction.rs1);
        let b = self.get_register(instruction.rs2);
        self.set_register(instruction.rd, a.wrapping_add(b));
        self.pc += RV32I::LENGTH;
        Ok(())
    }

    fn rv32i_sub(&mut self, instruction: RType) -> Result<(), Error> {
        let a = self.get_register(instruction.rs1);
        let b = self.get_register(instruction.rs2);
        self.set_register(instruction.rd, a.wrapping_sub(b));
        self.pc += RV32I::LENGTH;
        Ok(())
    }

    /// ADDI adds the sign-extended 12-bit immediate to register rs1. Arithmetic
    /// overflow is ignored and the result is simply the low XLEN bits of the
    /// result. ADDI rd, rs1, 0 is used to implement the MV rd, rs1 assembler
    /// pseudo-instruction.
    fn rv32i_addi(&mut self, instruction: IType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let imm = instruction.imm.as_u32();

        let new_value = rs1.wrapping_add(imm);

        self.set_register(instruction.rd, new_value);
        self.pc += RV32I::LENGTH;
        Ok(())
    }

    /// SLTI (set less than immediate) places the value 1 in register rd if
    /// register rs1 is less than the sign-extended immediate when both are
    /// treated as signed numbers, else 0 is written to rd.
    fn rv32i_slti(&mut self, instruction: IType) -> Result<(), Error> {
        // rs1 and the immediate value are treated as signed
        let signed_rs1 = self.get_register(instruction.rs1) as i32;
        let signed_imm = instruction.imm.as_i32();

        if signed_rs1 < signed_imm {
            self.set_register(instruction.rd, 1);
        } else {
            self.set_register(instruction.rd, 0);
        }

        self.pc += RV32I::LENGTH;
        Ok(())
    }

    /// SLTIU is similar but compares the values as unsigned numbers (i.e., the
    /// immediate is first sign-extended to XLEN bits then treated as an
    /// unsigned number). Note, SLTIU rd, rs1, 1 sets rd to 1 if rs1 equals
    /// zero, otherwise sets rd to 0 (assembler pseudo-op SEQZ rd, rs).
    fn rv32i_sltiu(&mut self, instruction: IType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let imm = instruction.imm.as_u32();

        if rs1 < imm {
            self.set_register(instruction.rd, 1);
        } else {
            self.set_register(instruction.rd, 0);
        }

        self.pc += RV32I::LENGTH;
        Ok(())
    }

    /// ANDI, ORI, XORI are logical operations that perform bitwise AND, OR,
    /// and XOR on register rs1 and the sign-extended 12-bit immediate and place
    /// the result in rd. Note, XORI rd, rs1, -1 performs a bitwise logical
    /// inversion of register rs1 (assembler pseudo-instruction NOT rd, rs).
    fn rv32i_andi(&mut self, instruction: IType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let imm = instruction.imm.as_u32();

        let value = imm & rs1;
        self.set_register(instruction.rd, value);

        self.pc += RV32I::LENGTH;
        Ok(())
    }

    fn rv32i_ori(&mut self, instruction: IType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let imm = instruction.imm.as_u32();

        let value = imm | rs1;
        self.set_register(instruction.rd, value);

        self.pc += RV32I::LENGTH;
        Ok(())
    }

    fn rv32i_xori(&mut self, instruction: IType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let imm = instruction.imm.as_u32();

        let value = imm ^ rs1;
        self.set_register(instruction.rd, value);

        self.pc += RV32I::LENGTH;
        Ok(())
    }

    /// LUI (load upper immediate) is used to build 32-bit constants and uses
    /// the U-type format. LUI places the U-immediate value in the top 20 bits
    /// of the destination register rd, filling in the lowest 12 bits with
    /// zeros.
    fn rv32i_lui(&mut self, instruction: UType) -> Result<(), Error> {
        let mut imm = instruction.imm.as_u32();
        imm <<= 12;
        self.set_register(instruction.rd, imm);

        self.pc += RV32I::LENGTH;
        Ok(())
    }

    /// AUIPC (add upper immediate to pc) is used to build pc-relative
    /// addresses and uses the U-type format. AUIPC forms a 32-bit offset from
    /// the 20-bit U-immediate, filling in the lowest 12 bits with zeros, adds
    /// this offset to the pc, then places the result in register rd.
    fn rv32i_auipc(&mut self, instruction: UType) -> Result<(), Error> {
        let mut imm = instruction.imm.as_u32();
        imm <<= 12;
        let pc = self.pc;
        let value = imm + pc;
        self.set_register(instruction.rd, value);

        self.pc += RV32I::LENGTH;
        Ok(())
    }

    /// SLT and SLTU perform signed and unsigned compares respectively, writing
    /// 1 to rd if rs1 < rs2, 0 otherwise. Note, SLTU rd, x0, rs2 sets rd to 1
    /// if rs2 is not equal to zero, otherwise sets rd to zero (assembler
    /// pseudo-op SNEZ rd, rs)
    fn rv32i_slt(&mut self, instruction: RType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1) as i32;
        let rs2 = self.get_register(instruction.rs2) as i32;

        if rs1 < rs2 {
            self.set_register(instruction.rd, 1);
        } else {
            self.set_register(instruction.rd, 0);
        }

        self.pc += RV32I::LENGTH;
        Ok(())
    }

    fn rv32i_sltu(&mut self, instruction: RType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let rs2 = self.get_register(instruction.rs2);

        if instruction.rs1 == Register::X0 {
            // exception for rs1 being x0
            if rs2 != 0 {
                self.set_register(instruction.rd, 1);
            } else {
                self.set_register(instruction.rd, 0);
            }
        } else {
            // normal case for comparison
            if rs1 < rs2 {
                self.set_register(instruction.rd, 1);
            } else {
                self.set_register(instruction.rd, 0);
            }
        }

        self.pc += RV32I::LENGTH;
        Ok(())
    }

    /// AND, OR, and XOR perform bitwise logical operations
    fn rv32i_and(&mut self, instruction: RType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let rs2 = self.get_register(instruction.rs2);

        let value = rs1 & rs2;
        self.set_register(instruction.rd, value);

        self.pc += RV32I::LENGTH;
        Ok(())
    }

    fn rv32i_or(&mut self, instruction: RType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let rs2 = self.get_register(instruction.rs2);

        let value = rs1 | rs2;
        self.set_register(instruction.rd, value);

        self.pc += RV32I::LENGTH;
        Ok(())
    }

    fn rv32i_xor(&mut self, instruction: RType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let rs2 = self.get_register(instruction.rs2);

        let value = rs1 ^ rs2;
        self.set_register(instruction.rd, value);

        self.pc += RV32I::LENGTH;
        Ok(())
    }

    /// SLL, SRL, and SRA perform logical left, logical right, and arithmetic
    /// right shifts on the value in register rs1 by the shift amount held in
    /// the lower 5 bits of register rs2.
    fn rv32i_sll(&mut self, instruction: RType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let rs2 = self.get_register(instruction.rs2);

        // lower 5 bit register mask
        let mask = 0b0000_0000_0000_0000_0000_0000_0001_1111;
        let shift_amount = rs2 & mask;
        let value = rs1 << shift_amount;
        self.set_register(instruction.rd, value);

        self.pc += RV32I::LENGTH;
        Ok(())
    }

    fn rv32i_srl(&mut self, instruction: RType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let rs2 = self.get_register(instruction.rs2);

        // lower 5 bit register mask
        let mask = 0b0000_0000_0000_0000_0000_0000_0001_1111;
        let shift_amount = rs2 & mask;

        let value = rs1 >> shift_amount;
        self.set_register(instruction.rd, value);

        self.pc += RV32I::LENGTH;
        Ok(())
    }

    fn rv32i_sra(&mut self, instruction: RType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let rs2 = self.get_register(instruction.rs2);

        // lower 5 bit register mask
        let mask = 0b0000_0000_0000_0000_0000_0000_0001_1111;
        let shift_amount = rs2 & mask;

        // Rust uses arithmetic right shifts on signed values!
        let value = (rs1 as i32) >> shift_amount;
        self.set_register(instruction.rd, value as u32);

        self.pc += RV32I::LENGTH;
        Ok(())
    }

    /// The jump and link (JAL) instruction uses the J-type format, where the
    /// J-immediate encodes a signed offset in multiples of 2 bytes. The offset
    /// is sign-extended and added to the pc to form the jump target address.
    /// Jumps can therefore target a ±1 MiB range. JAL stores the address of
    /// the instruction following the jump (pc+4) into register rd. The
    /// standard software calling convention uses x1 as the return
    /// address register and x5 as an alternate link register.
    /// Plain unconditional jumps (assembler pseudo-op J) are encoded as a JAL
    /// with rd=x0.
    fn rv32i_jal(&mut self, instruction: JType) -> Result<(), Error> {
        let mut offset = instruction.imm.as_u32();

        // shift left one bit; multiply by 2
        offset <<= 1;

        // create the offset address
        let offset_address = self.pc.wrapping_add(offset);

        // validate the offset address is 32-bit aligned
        if offset_address % 4 != 0 {
            return Err(Error::MisalignedJump(offset_address));
        }

        // set the return address
        let return_address = self.pc.wrapping_add(RV32I::LENGTH);

        self.set_register(Register::PC, offset_address);
        self.set_register(instruction.rd, return_address);

        Ok(())
    }

    /// The indirect jump instruction JALR (jump and link register) uses the
    /// I-type encoding. The target address is obtained by adding the 12-bit
    /// signed I-immediate to the register rs1, then setting the
    /// least-significant bit of the result to zero. The address of the
    /// instruction following the jump (pc+4) is written to register rd.
    /// Register x0 can be used as the destination if the result is not
    /// required.
    fn rv32i_jalr(&mut self, instruction: IType) -> Result<(), Error> {
        let offset = instruction.imm.as_u32();
        let rs1 = self.get_register(instruction.rs1);

        let mut offset_address = rs1.wrapping_add(offset);

        // I'm sure there's a better way to zero the LSB
        offset_address >>= 1;
        offset_address <<= 1;

        // validate the offset address is 32-bit aligned
        if offset_address % 4 != 0 {
            return Err(Error::MisalignedJump(offset_address));
        }

        let return_address = self.pc.wrapping_add(RV32I::LENGTH);

        self.set_register(Register::PC, offset_address);
        self.set_register(instruction.rd, return_address);

        Ok(())
    }

    /// All branch instructions use the B-type instruction format. The 12-bit
    /// B-immediate encodes signed offsets in multiples of 2, and is added to
    /// the current pc to give the target address. The conditional branch range
    /// is ±4 KiB
    ///
    /// BEQ and BNE take the branch if registers rs1 and rs2 are equal or
    /// unequal respectively.
    fn rv32i_beq(&mut self, instruction: BType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let rs2 = self.get_register(instruction.rs2);

        if rs1 == rs2 {
            let mut offset = instruction.imm.as_u32();
            offset <<= 1; // multiple of 2
            self.pc = self.pc.wrapping_add(offset);
        } else {
            self.pc += RV32I::LENGTH;
        }

        Ok(())
    }

    fn rv32i_bne(&mut self, instruction: BType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let rs2 = self.get_register(instruction.rs2);

        if rs1 != rs2 {
            let mut offset = instruction.imm.as_u32();
            offset <<= 1; // multiple of 2
            self.pc = self.pc.wrapping_add(offset);
        } else {
            self.pc += RV32I::LENGTH;
        }

        Ok(())
    }

    ///  BLT and BLTU take the branch if rs1 is less than rs2, using signed
    ///  and unsigned comparison respectively.
    fn rv32i_blt(&mut self, instruction: BType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1) as i32;
        let rs2 = self.get_register(instruction.rs2) as i32;

        if rs1 < rs2 {
            let mut offset = instruction.imm.as_u32();
            offset <<= 1; // multiple of 2
            self.pc = self.pc.wrapping_add(offset);
        } else {
            self.pc += RV32I::LENGTH;
        }

        Ok(())
    }

    fn rv32i_bltu(&mut self, instruction: BType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let rs2 = self.get_register(instruction.rs2);

        if rs1 < rs2 {
            let mut offset = instruction.imm.as_u32();
            offset <<= 1; // multiple of 2
            self.pc = self.pc.wrapping_add(offset);
        } else {
            self.pc += RV32I::LENGTH;
        }

        Ok(())
    }

    ///  BGE and BGEU take the branch if rs1 is greater than or equal to rs2,
    ///  using signed and unsigned comparison respectively.
    fn rv32i_bge(&mut self, instruction: BType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1) as i32;
        let rs2 = self.get_register(instruction.rs2) as i32;

        if rs1 >= rs2 {
            let mut offset = instruction.imm.as_u32();
            offset <<= 1; // multiple of 2
            self.pc = self.pc.wrapping_add(offset);
        } else {
            self.pc += RV32I::LENGTH;
        }

        Ok(())
    }

    fn rv32i_bgeu(&mut self, instruction: BType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let rs2 = self.get_register(instruction.rs2);

        if rs1 >= rs2 {
            let mut offset = instruction.imm.as_u32();
            offset <<= 1; // multiple of 2
            self.pc = self.pc.wrapping_add(offset);
        } else {
            self.pc += RV32I::LENGTH;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    NotImplemented(RV32I),
    MisalignedJump(u32),
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

#[derive(Debug, Copy, Clone)]
pub struct IType {
    pub opcode: u8,
    pub rd: Register,
    pub funct3: u8,
    pub rs1: Register,
    pub imm: Immediate,
}

impl Default for IType {
    fn default() -> Self {
        Self::new()
    }
}

impl IType {
    const IMM_BITS: u8 = 12;

    pub fn new() -> Self {
        Self {
            opcode: 0, // TODO
            rd: Register::default(),
            funct3: 0, // TODO
            rs1: Register::default(),
            imm: Immediate::new(Self::IMM_BITS),
        }
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
    pub imm: Immediate,
    pub funct3: u8,
    pub rs1: Register,
    pub rs2: Register,
}

impl BType {
    const IMM_BITS: u8 = 12;

    pub fn new() -> Self {
        Self {
            opcode: 0,
            imm: Immediate::new(Self::IMM_BITS),
            funct3: 0,
            rs1: Register::default(),
            rs2: Register::default(),
        }
    }
}

impl Default for BType {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct UType {
    pub opcode: u8,
    pub rd: Register,
    pub imm: Immediate,
}

impl Default for UType {
    fn default() -> Self {
        Self::new()
    }
}

impl UType {
    const IMM_BITS: u8 = 20;

    pub fn new() -> Self {
        Self {
            opcode: 0, // TODO
            rd: Register::default(),
            imm: Immediate::new(Self::IMM_BITS),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct JType {
    pub opcode: u8,
    pub rd: Register,
    pub imm: Immediate,
}

impl Default for JType {
    fn default() -> Self {
        Self::new()
    }
}

impl JType {
    const IMM_BITS: u8 = 20;

    pub fn new() -> Self {
        Self {
            opcode: 0, // TODO
            rd: Register::default(),
            imm: Immediate::new(Self::IMM_BITS),
        }
    }
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
    fn rv32i_add_sub() {
        let mut cpu = CPU::new();
        let mut inst = RType::default();

        inst.rd = Register::X1;
        inst.rs1 = Register::X2;
        inst.rs2 = Register::X3;

        let add = RV32I::ADD(inst);
        let sub = RV32I::SUB(inst);

        // zero values
        let result = cpu.execute(add);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 0);

        // non-overflowing add and sub
        cpu.set_register(Register::X2, 8);
        cpu.set_register(Register::X3, 4);

        let result = cpu.execute(add);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 12);

        let result = cpu.execute(sub);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 4);

        // overflowing addition
        cpu.set_register(Register::X2, 3);
        cpu.set_register(Register::X3, u32::MAX - 1);

        let result = cpu.execute(add);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 1);

        let result = cpu.execute(sub);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 5);
    }

    #[test]
    fn rv32i_addi() {
        let mut cpu = CPU::new();
        let mut inst = IType::new();

        inst.rd = Register::X1;
        inst.rs1 = Register::X1;
        inst.imm.set_unsigned(0).unwrap();

        let addi = RV32I::ADDI(inst);

        // zero value
        let result = cpu.execute(addi);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 0);

        // positive values
        inst.imm.set_unsigned(5).unwrap();
        let addi = RV32I::ADDI(inst);
        let result = cpu.execute(addi);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 5);

        // negative values; this is a mess!
        let result = inst.imm.set_signed(-3);
        assert!(result.is_ok());
        let addi = RV32I::ADDI(inst);
        let result = cpu.execute(addi);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 2);
    }

    #[test]
    fn rv32i_slti() {
        let mut cpu = CPU::new();
        let mut inst = IType::default();

        inst.rd = Register::X1;
        inst.rs1 = Register::X2;
        inst.imm.set_unsigned(0).unwrap();

        let slti = RV32I::SLTI(inst);

        // zero / equal value
        let result = cpu.execute(slti);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 0);
        assert_eq!(cpu.pc, RV32I::LENGTH);

        // greater than value
        inst.imm.set_signed(1).unwrap();
        let slti = RV32I::SLTI(inst);
        let result = cpu.execute(slti);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 1);
        assert_eq!(cpu.pc, RV32I::LENGTH * 2);

        // less than value (negative, just for kicks)
        inst.imm.set_signed(-1).unwrap();
        let slti = RV32I::SLTI(inst);
        let result = cpu.execute(slti);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 0);
        assert_eq!(cpu.pc, RV32I::LENGTH * 3);
    }

    #[test]
    fn rv32i_sltiu() {
        let mut cpu = CPU::new();
        let mut inst = IType::default();

        cpu.x2 = 255; // initial value to compare against

        inst.rd = Register::X1;
        inst.rs1 = Register::X2;

        // equal value
        inst.imm.set_unsigned(255).unwrap();
        let sltiu = RV32I::SLTIU(inst);
        let result = cpu.execute(sltiu);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 0);
        assert_eq!(cpu.pc, RV32I::LENGTH);

        // greater than value
        inst.imm.set_unsigned(256).unwrap();
        let sltiu = RV32I::SLTIU(inst);
        let result = cpu.execute(sltiu);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 1);
        assert_eq!(cpu.pc, RV32I::LENGTH * 2);

        // less than value
        inst.imm.set_unsigned(254).unwrap();
        let sltiu = RV32I::SLTIU(inst);
        let result = cpu.execute(sltiu);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 0);
        assert_eq!(cpu.pc, RV32I::LENGTH * 3);
    }

    #[test]
    fn rv32i_andi_ori_xori() {
        let mut cpu = CPU::new();
        let mut inst = IType::default();

        inst.rd = Register::X1;
        inst.rs1 = Register::X2;

        // all 1s across the register and imm
        let result = inst.imm.set_unsigned(inst.imm.unsigned_max());
        assert!(result.is_ok());
        cpu.x2 = u32::MAX;

        let andi = RV32I::ANDI(inst);
        let result = cpu.execute(andi);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, u32::MAX);

        let ori = RV32I::ORI(inst);
        let result = cpu.execute(ori);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, u32::MAX);

        let xori = RV32I::XORI(inst);
        let result = cpu.execute(xori);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 0);

        // all 0s in imm
        let result = inst.imm.set_unsigned(0);
        assert!(result.is_ok());
        cpu.x2 = u32::MAX;

        let andi = RV32I::ANDI(inst);
        let result = cpu.execute(andi);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 0);

        let ori = RV32I::ORI(inst);
        let result = cpu.execute(ori);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, u32::MAX);

        let xori = RV32I::XORI(inst);
        let result = cpu.execute(xori);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, u32::MAX);
    }

    #[test]
    fn rv32i_lui() {
        let mut cpu = CPU::new();
        let mut inst = UType::default();

        inst.rd = Register::X1;
        let result = inst.imm.set_unsigned(1);
        assert!(result.is_ok());

        let lui = RV32I::LUI(inst);
        let result = cpu.execute(lui);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 0b0000_0000_0000_0000_0001_0000_0000_0000);
    }

    #[test]
    fn rv32i_auipc() {
        let mut cpu = CPU::new();
        let mut inst = UType::default();

        inst.rd = Register::X1;
        let result = inst.imm.set_unsigned(1);
        assert!(result.is_ok());

        // from PC 0
        let auipc = RV32I::AUIPC(inst);
        let result = cpu.execute(auipc);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 0b0000_0000_0000_0000_0001_0000_0000_0000);

        // from 0 + RV32I::LENGTH
        let result = cpu.execute(auipc);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 0b0000_0000_0000_0000_0001_0000_0000_0100);
    }

    #[test]
    fn rv32i_jal() {
        let mut cpu = CPU::new();
        let mut inst = JType::default();

        inst.rd = Register::X1;
        let result = inst.imm.set_unsigned(4);
        assert!(result.is_ok());

        let jal = RV32I::JAL(inst);
        let result = cpu.execute(jal);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, 8); // current pc (0) + (4 * 2)
        assert_eq!(cpu.x1, 4); // current pc (0) + RV32I::LENGTH

        // misalignment check!
        let result = inst.imm.set_unsigned(1);
        assert!(result.is_ok());
        let jal = RV32I::JAL(inst);
        let result = cpu.execute(jal);
        assert!(result.is_err());
    }

    #[test]
    fn rv32i_jalr() {
        let mut cpu = CPU::new();
        let mut inst = IType::default();

        inst.rs1 = Register::X2;
        inst.rd = Register::X1;
        let result = inst.imm.set_unsigned(12);
        assert!(result.is_ok());

        let jalr = RV32I::JALR(inst);
        let result = cpu.execute(jalr);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, 12);
        assert_eq!(cpu.x1, 4);

        cpu.pc = 0;
        cpu.x2 = 24;
        let result = inst.imm.set_signed(-12);
        assert!(result.is_ok());

        let jalr = RV32I::JALR(inst);
        let result = cpu.execute(jalr);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, 12);
        assert_eq!(cpu.x1, 4);
    }

    #[test]
    fn rv32i_beq() {
        let mut cpu = CPU::new();
        let mut inst = BType::default();

        cpu.x1 = 24;
        cpu.x2 = 24;
        cpu.pc = 0;

        inst.rs1 = Register::X1;
        inst.rs2 = Register::X2;

        inst.imm.set_signed(64).unwrap();
        let beq = RV32I::BEQ(inst);
        let result = cpu.execute(beq);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, 128); // doubled

        inst.imm.set_signed(-128).unwrap();
        let beq = RV32I::BEQ(inst);
        let result = cpu.execute(beq);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, -128i32 as u32); // doubled

        inst.rs1 = Register::X3;
        cpu.pc = 0;

        inst.imm.set_signed(64).unwrap();
        let beq = RV32I::BEQ(inst);
        let result = cpu.execute(beq);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, RV32I::LENGTH); // skipped
    }

    #[test]
    fn rv32i_bne() {
        let mut cpu = CPU::new();
        let mut inst = BType::default();

        cpu.x1 = 23;
        cpu.x2 = 24;
        cpu.pc = 0;

        inst.rs1 = Register::X1;
        inst.rs2 = Register::X2;

        inst.imm.set_signed(64).unwrap();
        let beq = RV32I::BNE(inst);
        let result = cpu.execute(beq);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, 128); // doubled

        inst.imm.set_signed(-128).unwrap();
        let beq = RV32I::BNE(inst);
        let result = cpu.execute(beq);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, -128i32 as u32); // doubled

        cpu.x1 = 24; // should be equal now
        cpu.pc = 0;

        inst.imm.set_signed(64).unwrap();
        let beq = RV32I::BNE(inst);
        let result = cpu.execute(beq);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, RV32I::LENGTH); // skipped
    }
}
