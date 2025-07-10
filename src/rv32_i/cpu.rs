//! Represents the state of registers and memory for a little endian, single
//! hardware thread ("hart") RV32I CPU.
//!
//! Registers can be accessed directly, via `get_register()`, or `get_abi()`
//! (for [ABI](crate::rv32_i::ABI) aliases). Registers operate as native u32 values for ease of use.
//! Memory operates as little endian, so the 16-bit value `0x12ab` would be
//! stored in memory as `[0xab, 0x12]`.

use super::*;

// Standard CSR addresses
const CSR_CYCLE: u16 = 0xC00; // Cycle counter (read-only)
const CSR_TIME: u16 = 0xC01; // Timer (read-only)
const CSR_INSTRET: u16 = 0xC02; // Instructions retired (read-only)
const CSR_MSTATUS: u16 = 0x300; // Machine status register
const CSR_MISA: u16 = 0x301; // Machine ISA register
const CSR_MIE: u16 = 0x304; // Machine interrupt enable
const CSR_MTVEC: u16 = 0x305; // Machine trap vector base address
const CSR_MSCRATCH: u16 = 0x340; // Machine scratch register
const CSR_MEPC: u16 = 0x341; // Machine exception program counter
const CSR_MCAUSE: u16 = 0x342; // Machine trap cause
const CSR_MTVAL: u16 = 0x343; // Machine trap value
const CSR_MIP: u16 = 0x344; // Machine interrupt pending

#[derive(Debug, Clone)]
pub struct CPU {
    pub memory: Vec<u8>,
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

    // CSR (Control and Status Register) support
    pub csrs: [u32; 4096],          // CSR values indexed by address
    pub csr_exists: [bool; 4096],   // Which CSR addresses are implemented
    pub csr_readonly: [bool; 4096], // Which CSRs are read-only

    // State tracking for undo/redo (only when REPL feature is enabled)
    #[cfg(feature = "repl")]
    pub memory_changes: Vec<crate::history::MemoryDelta>,
    #[cfg(feature = "repl")]
    pub csr_changes: Vec<(u32, u32, u32)>, // (address, old_value, new_value)
    #[cfg(feature = "repl")]
    pub register_changes: Vec<(Register, u32, u32)>, // (register, old_value, new_value)
}

impl Default for CPU {
    /// Initializes the [CPU] with 1 mebibyte (2^20) of memory
    fn default() -> Self {
        Self::new(2usize.pow(20)) // default 1 mebibyte
    }
}

impl CPU {
    /// Creates a single hardware thread ("hart") CPU implementing the RV32I
    /// instruction set. Memory size is counted in bytes; `default()` will
    /// initialize with 1 mebibyte.
    pub fn new(memory_size: usize) -> Self {
        let mut cpu = Self {
            memory: vec![0; memory_size],
            x0: 0,
            x1: 0,
            x2: 0,
            x3: 0,
            x4: 0,
            x5: 0,
            x6: 0,
            x7: 0,
            x8: 0,
            x9: 0,
            x10: 0,
            x11: 0,
            x12: 0,
            x13: 0,
            x14: 0,
            x15: 0,
            x16: 0,
            x17: 0,
            x18: 0,
            x19: 0,
            x20: 0,
            x21: 0,
            x22: 0,
            x23: 0,
            x24: 0,
            x25: 0,
            x26: 0,
            x27: 0,
            x28: 0,
            x29: 0,
            x30: 0,
            x31: 0,
            pc: 0,
            csrs: [0; 4096],
            csr_exists: [false; 4096],
            csr_readonly: [false; 4096],
            #[cfg(feature = "repl")]
            memory_changes: Vec::new(),
            #[cfg(feature = "repl")]
            csr_changes: Vec::new(),
            #[cfg(feature = "repl")]
            register_changes: Vec::new(),
        };

        // Initialize standard CSRs
        cpu.init_csrs();
        cpu
    }

    /// Gets the value for a given register.
    ///
    /// `Register::X0` will always remain zero
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

    /// Sets a given register to the provided value.
    ///
    /// `Register::X0` will always remain zero
    pub fn set_register(&mut self, r: Register, v: u32) {
        #[cfg(feature = "repl")]
        let old_value = self.get_register(r);

        match r {
            Register::X0 => self.x0 = 0,
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

        #[cfg(feature = "repl")]
        {
            let new_value = self.get_register(r);
            if old_value != new_value {
                self.register_changes.push((r, old_value, new_value));
            }
        }
    }

    /// Gets the content of a register by it's ABI name
    pub fn get_abi(&self, abi: ABI) -> u32 {
        self.get_register(abi.to_register())
    }

    /// Sets the content of a register by it's ABI name
    pub fn set_abi(&mut self, abi: ABI, v: u32) {
        self.set_register(abi.to_register(), v)
    }

    /// Initialize standard CSRs with their default values
    pub fn init_csrs(&mut self) {
        // User-level CSRs (read-only performance counters)
        self.csr_exists[CSR_CYCLE as usize] = true;
        self.csr_readonly[CSR_CYCLE as usize] = true;

        self.csr_exists[CSR_TIME as usize] = true;
        self.csr_readonly[CSR_TIME as usize] = true;

        self.csr_exists[CSR_INSTRET as usize] = true;
        self.csr_readonly[CSR_INSTRET as usize] = true;

        // Machine-level CSRs
        self.csr_exists[CSR_MSTATUS as usize] = true;
        self.csrs[CSR_MSTATUS as usize] = 0x00001800; // MPP = 11 (M-mode)

        self.csr_exists[CSR_MISA as usize] = true;
        self.csr_readonly[CSR_MISA as usize] = true;
        self.csrs[CSR_MISA as usize] = 0x40000100; // RV32I base ISA

        self.csr_exists[CSR_MIE as usize] = true;
        self.csr_exists[CSR_MIP as usize] = true;
        self.csr_exists[CSR_MTVEC as usize] = true;
        self.csr_exists[CSR_MSCRATCH as usize] = true;
        self.csr_exists[CSR_MEPC as usize] = true;
        self.csr_exists[CSR_MCAUSE as usize] = true;
        self.csr_exists[CSR_MTVAL as usize] = true;
    }

    /// Read a CSR value
    pub fn read_csr(&self, addr: u16) -> Result<u32, Error> {
        if addr >= 4096 || !self.csr_exists[addr as usize] {
            return Err(Error::IllegalInstruction(format!(
                "CSR address 0x{addr:03x} does not exist"
            )));
        }

        // Special handling for dynamic CSRs
        match addr {
            CSR_CYCLE => {
                // For now, return a dummy cycle count
                // In a real implementation, this would track actual cycles
                Ok(0) // TODO: Implement cycle counting
            }
            CSR_TIME => {
                // For now, return 0
                // In a real implementation, this would return wall-clock time
                Ok(0) // TODO: Implement timer
            }
            CSR_INSTRET => {
                // For now, return 0
                // In a real implementation, this would count retired instructions
                Ok(0) // TODO: Implement instruction counting
            }
            _ => Ok(self.csrs[addr as usize]),
        }
    }

    /// Write a CSR value (returns old value like CSRRW instruction)
    pub fn write_csr(&mut self, addr: u16, value: u32) -> Result<u32, Error> {
        if addr >= 4096 || !self.csr_exists[addr as usize] {
            return Err(Error::IllegalInstruction(format!(
                "CSR address 0x{addr:03x} does not exist"
            )));
        }

        if self.csr_readonly[addr as usize] {
            return Err(Error::IllegalInstruction(format!(
                "CSR address 0x{addr:03x} is read-only"
            )));
        }

        // Read old value first (atomic read-modify-write)
        let old_value = self.read_csr(addr)?;

        // Apply WARL (Write Any Read Legal) transformations
        let legal_value = match addr {
            CSR_MSTATUS => {
                // Only certain bits are writable in mstatus
                // Preserve read-only bits
                // Writable bits: MIE (bit 3), MPIE (bit 7), MPP (bits 11-12)
                let mask = 0x00001888; // Bits 3, 7, 11, 12
                (self.csrs[addr as usize] & !mask) | (value & mask)
            }
            _ => value,
        };

        // Track CSR change if needed
        #[cfg(feature = "repl")]
        if legal_value != old_value {
            self.csr_changes.push((addr as u32, old_value, legal_value));
        }

        self.csrs[addr as usize] = legal_value;
        Ok(old_value)
    }

    /// Set bits in a CSR (for CSRRS instruction)
    pub fn set_csr_bits(&mut self, addr: u16, mask: u32) -> Result<u32, Error> {
        let old_value = self.read_csr(addr)?;
        if mask != 0 {
            self.write_csr(addr, old_value | mask)?;
        }
        Ok(old_value)
    }

    /// Clear bits in a CSR (for CSRRC instruction)
    pub fn clear_csr_bits(&mut self, addr: u16, mask: u32) -> Result<u32, Error> {
        let old_value = self.read_csr(addr)?;
        if mask != 0 {
            self.write_csr(addr, old_value & !mask)?;
        }
        Ok(old_value)
    }

    /// Does what it says on the tin!
    ///
    /// ```
    /// use brubeck::rv32_i::*;
    ///
    /// let mut cpu = CPU::default();
    /// let nop = Instruction::NOP;
    /// let result = cpu.execute(nop);
    ///
    /// // successful execution is ok!
    /// assert!(result.is_ok());
    ///
    /// // PC should be incremented by the length of the NOP instruction
    /// assert_eq!(cpu.pc, Instruction::LENGTH);
    /// ```
    pub fn execute(&mut self, instruction: Instruction) -> Result<(), Error> {
        match instruction {
            Instruction::ADD(i) => self.rv32i_add(i),
            Instruction::ADDI(i) => self.rv32i_addi(i),
            Instruction::AND(i) => self.rv32i_and(i),
            Instruction::ANDI(i) => self.rv32i_andi(i),
            Instruction::AUIPC(i) => self.rv32i_auipc(i),
            Instruction::BEQ(i) => self.rv32i_beq(i),
            Instruction::BGE(i) => self.rv32i_bge(i),
            Instruction::BGEU(i) => self.rv32i_bgeu(i),
            Instruction::BLT(i) => self.rv32i_blt(i),
            Instruction::BLTU(i) => self.rv32i_bltu(i),
            Instruction::BNE(i) => self.rv32i_bne(i),
            Instruction::JAL(i) => self.rv32i_jal(i),
            Instruction::JALR(i) => self.rv32i_jalr(i),
            Instruction::LB(i) => self.rv32i_lb(i),
            Instruction::LBU(i) => self.rv32i_lbu(i),
            Instruction::LH(i) => self.rv32i_lh(i),
            Instruction::LHU(i) => self.rv32i_lhu(i),
            Instruction::LUI(i) => self.rv32i_lui(i),
            Instruction::LW(i) => self.rv32i_lw(i),
            Instruction::NOP => self.rv32i_nop(),
            Instruction::OR(i) => self.rv32i_or(i),
            Instruction::ORI(i) => self.rv32i_ori(i),
            Instruction::SB(i) => self.rv32i_sb(i),
            Instruction::SH(i) => self.rv32i_sh(i),
            Instruction::SLL(i) => self.rv32i_sll(i),
            Instruction::SLLI(i) => self.rv32i_slli(i),
            Instruction::SLT(i) => self.rv32i_slt(i),
            Instruction::SLTI(i) => self.rv32i_slti(i),
            Instruction::SLTIU(i) => self.rv32i_sltiu(i),
            Instruction::SLTU(i) => self.rv32i_sltu(i),
            Instruction::SRA(i) => self.rv32i_sra(i),
            Instruction::SRAI(i) => self.rv32i_srai(i),
            Instruction::SRL(i) => self.rv32i_srl(i),
            Instruction::SRLI(i) => self.rv32i_srli(i),
            Instruction::SUB(i) => self.rv32i_sub(i),
            Instruction::SW(i) => self.rv32i_sw(i),
            Instruction::XOR(i) => self.rv32i_xor(i),
            Instruction::XORI(i) => self.rv32i_xori(i),
            Instruction::FENCE(_) => self.rv32i_fence(),
            Instruction::ECALL(_) => self.rv32i_ecall(),
            Instruction::EBREAK(_) => self.rv32i_ebreak(),

            // CSR Instructions
            Instruction::CSRRW(i) => self.rv32i_csrrw(i),
            Instruction::CSRRS(i) => self.rv32i_csrrs(i),
            Instruction::CSRRC(i) => self.rv32i_csrrc(i),
            Instruction::CSRRWI(i) => self.rv32i_csrrwi(i),
            Instruction::CSRRSI(i) => self.rv32i_csrrsi(i),
            Instruction::CSRRCI(i) => self.rv32i_csrrci(i),
        }?;

        Ok(())
    }

    /*
     *  All functions below are either instructions or helper functions for execution.
     *
     *  Naming follows the convention isa_instruction (eg: rv32i_nop)
     */

    fn increment_pc(&mut self) -> Result<(), Error> {
        self.pc += Instruction::LENGTH;
        Ok(())
    }

    fn rv32i_nop(&mut self) -> Result<(), Error> {
        self.increment_pc()
    }

    /// ADD and SUB perform addition and subtraction respectively. Overflows
    /// are ignored and the low XLEN bits of results are written to the
    /// destination.
    fn rv32i_add(&mut self, instruction: RType) -> Result<(), Error> {
        let a = self.get_register(instruction.rs1);
        let b = self.get_register(instruction.rs2);
        self.set_register(instruction.rd, a.wrapping_add(b));
        self.increment_pc()
    }

    fn rv32i_sub(&mut self, instruction: RType) -> Result<(), Error> {
        let a = self.get_register(instruction.rs1);
        let b = self.get_register(instruction.rs2);
        self.set_register(instruction.rd, a.wrapping_sub(b));
        self.increment_pc()
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
        self.increment_pc()
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

        self.increment_pc()
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

        self.increment_pc()
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

        self.increment_pc()
    }

    fn rv32i_ori(&mut self, instruction: IType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let imm = instruction.imm.as_u32();

        let value = imm | rs1;
        self.set_register(instruction.rd, value);

        self.increment_pc()
    }

    fn rv32i_xori(&mut self, instruction: IType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let imm = instruction.imm.as_u32();

        let value = imm ^ rs1;
        self.set_register(instruction.rd, value);

        self.increment_pc()
    }

    /// LUI (load upper immediate) is used to build 32-bit constants and uses
    /// the U-type format. LUI places the U-immediate value in the top 20 bits
    /// of the destination register rd, filling in the lowest 12 bits with
    /// zeros.
    fn rv32i_lui(&mut self, instruction: UType) -> Result<(), Error> {
        let mut imm = instruction.imm.as_u32();
        imm <<= 12;
        self.set_register(instruction.rd, imm);

        self.increment_pc()
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

        self.increment_pc()
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

        self.increment_pc()
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

        self.increment_pc()
    }

    /// AND, OR, and XOR perform bitwise logical operations
    fn rv32i_and(&mut self, instruction: RType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let rs2 = self.get_register(instruction.rs2);

        let value = rs1 & rs2;
        self.set_register(instruction.rd, value);

        self.increment_pc()
    }

    fn rv32i_or(&mut self, instruction: RType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let rs2 = self.get_register(instruction.rs2);

        let value = rs1 | rs2;
        self.set_register(instruction.rd, value);

        self.increment_pc()
    }

    fn rv32i_xor(&mut self, instruction: RType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let rs2 = self.get_register(instruction.rs2);

        let value = rs1 ^ rs2;
        self.set_register(instruction.rd, value);

        self.increment_pc()
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

        self.increment_pc()
    }

    fn rv32i_srl(&mut self, instruction: RType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let rs2 = self.get_register(instruction.rs2);

        // lower 5 bit register mask
        let mask = 0b0000_0000_0000_0000_0000_0000_0001_1111;
        let shift_amount = rs2 & mask;

        let value = rs1 >> shift_amount;
        self.set_register(instruction.rd, value);

        self.increment_pc()
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

        self.increment_pc()
    }

    /// Shifts by a constant are encoded as a specialization of the I-type
    /// format. The operand to be shifted is in rs1, and the shift amount is
    /// encoded in the lower 5 bits of the I-immediate field. The right shift
    /// type is encoded in bit 30. SLLI is a logical left shift (zeros are
    /// shifted into the lower bits); SRLI is a logical right shift (zeros
    /// are shifted into the upper bits); and SRAI is an arithmetic right shift
    /// (the original sign bit is copied into the vacated upper bits).
    fn rv32i_slli(&mut self, instruction: IType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let imm = instruction.imm.as_u32();

        let mask = 0b0000_0000_0000_0000_0000_0000_0001_1111;
        let shift_amount = imm & mask;

        let value = rs1 << shift_amount;
        self.set_register(instruction.rd, value);

        self.increment_pc()
    }

    fn rv32i_srli(&mut self, instruction: IType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let imm = instruction.imm.as_u32();

        // lower 5 bit register mask
        let mask = 0b0000_0000_0000_0000_0000_0000_0001_1111;
        let shift_amount = imm & mask;

        let value = rs1 >> shift_amount;
        self.set_register(instruction.rd, value);

        self.increment_pc()
    }

    fn rv32i_srai(&mut self, instruction: IType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let imm = instruction.imm.as_u32();

        // lower 5 bit register mask
        let mask = 0b0000_0000_0000_0000_0000_0000_0001_1111;
        let shift_amount = imm & mask;

        // Rust uses arithmetic right shifts on signed values!
        let value = (rs1 as i32) >> shift_amount;
        self.set_register(instruction.rd, value as u32);

        self.increment_pc()
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
        let return_address = self.pc.wrapping_add(Instruction::LENGTH);

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

        let return_address = self.pc.wrapping_add(Instruction::LENGTH);

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
            self.pc += Instruction::LENGTH;
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
            self.pc += Instruction::LENGTH;
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
            self.pc += Instruction::LENGTH;
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
            self.pc += Instruction::LENGTH;
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
            self.pc += Instruction::LENGTH;
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
            self.pc += Instruction::LENGTH;
        }

        Ok(())
    }

    /// Load and store instructions transfer a value between the registers and
    /// memory. Loads are encoded in the I-type format and stores are S-type.
    /// The effective byte address is obtained by adding register rs1 to the
    /// sign-extended 12-bit offset. Loads copy a value from memory to register
    /// rd. Stores copy the value in register rs2 to memory
    ///
    /// The LW instruction loads a 32-bit value from memory into rd.
    fn rv32i_lw(&mut self, instruction: IType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let imm = instruction.imm.as_u32();

        let offset = rs1.wrapping_add(imm);
        let index = offset as usize;

        if index >= self.memory.len() {
            return Err(Error::AccessViolation(rs1));
        }

        let mut value_buf = [0u8; 4];
        value_buf.clone_from_slice(&self.memory[index..index + 4]);
        let value = u32::from_le_bytes(value_buf);

        self.set_register(instruction.rd, value);
        self.increment_pc()
    }

    /// LH loads a 16-bit value from memory, then sign-extends to 32-bits before
    /// storing in rd.
    fn rv32i_lh(&mut self, instruction: IType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let imm = instruction.imm.as_u32();

        let offset = rs1.wrapping_add(imm);
        let index = offset as usize;

        if index >= self.memory.len() {
            return Err(Error::AccessViolation(rs1));
        }

        let mut value_buf = [0u8; 2];
        value_buf.clone_from_slice(&self.memory[index..index + 2]);
        let u16_value = u16::from_le_bytes(value_buf);
        // Sign-extend from 16-bit to 32-bit
        let value = u16_value as i16 as i32 as u32;

        self.set_register(instruction.rd, value);
        self.increment_pc()
    }

    /// LHU loads a 16-bit value from memory but then zero extends to 32-bits
    /// before storing in rd.
    fn rv32i_lhu(&mut self, instruction: IType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let imm = instruction.imm.as_u32();

        let offset = rs1.wrapping_add(imm);
        let index = offset as usize;

        if index >= self.memory.len() {
            return Err(Error::AccessViolation(rs1));
        }

        let mut value_buf = [0u8; 2];
        value_buf.clone_from_slice(&self.memory[index..index + 2]);
        let u16_value = u16::from_le_bytes(value_buf);

        let value = u16_value as u32;

        self.set_register(instruction.rd, value);
        self.increment_pc()
    }

    /// LB loads a 8-bit value from memory, then sign-extends to 32-bits before
    /// storing in rd.
    fn rv32i_lb(&mut self, instruction: IType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let imm = instruction.imm.as_u32();

        let offset = rs1.wrapping_add(imm);
        let index = offset as usize;

        if index >= self.memory.len() {
            return Err(Error::AccessViolation(rs1));
        }

        let u8_value = self.memory[index];
        // Sign-extend from 8-bit to 32-bit
        let value = u8_value as i8 as i32 as u32;

        self.set_register(instruction.rd, value);
        self.increment_pc()
    }

    /// LBU loads a 8-bit value from memory but then zero extends to 32-bits
    /// before storing in rd.
    fn rv32i_lbu(&mut self, instruction: IType) -> Result<(), Error> {
        let rs1 = self.get_register(instruction.rs1);
        let imm = instruction.imm.as_u32();

        let offset = rs1.wrapping_add(imm);
        let index = offset as usize;

        if index >= self.memory.len() {
            return Err(Error::AccessViolation(rs1));
        }

        let u8_value = self.memory[index];
        let value = u8_value as u32;

        self.set_register(instruction.rd, value);
        self.increment_pc()
    }

    /// The SW, SH, and SB instructions store 32-bit, 16-bit, and 8-bit values
    /// from the low bits of register rs2 to memory
    fn rv32i_sw(&mut self, instruction: SType) -> Result<(), Error> {
        self.store(instruction, 4)?;
        self.increment_pc()
    }

    fn rv32i_sh(&mut self, instruction: SType) -> Result<(), Error> {
        self.store(instruction, 2)?;
        self.increment_pc()
    }

    fn rv32i_sb(&mut self, instruction: SType) -> Result<(), Error> {
        self.store(instruction, 1)?;
        self.increment_pc()
    }

    fn store(&mut self, instruction: SType, bytes: usize) -> Result<(), Error> {
        let base = self.get_register(instruction.rs1);
        let src = self.get_register(instruction.rs2);
        let imm = instruction.imm.as_u32();

        let address = base.wrapping_add(imm);
        let mut index = address as usize;

        if index >= self.memory.len() {
            return Err(Error::AccessViolation(address));
        }

        for (byte_index, byte) in src.to_le_bytes().into_iter().enumerate() {
            if byte_index < bytes {
                #[cfg(feature = "repl")]
                {
                    // Track the memory change
                    let old_value = self.memory[index];
                    self.memory[index] = byte;
                    self.memory_changes.push(crate::history::MemoryDelta {
                        address: index as u32,
                        old_value,
                        new_value: byte,
                    });
                }
                #[cfg(not(feature = "repl"))]
                {
                    self.memory[index] = byte;
                }
                index += 1;
            }
        }

        Ok(())
    }

    /// FENCE instruction is used to order device I/O and memory accesses.
    /// In a simple implementation, it can be implemented as a NOP.
    ///
    /// Reference: RISC-V ISA Manual, Volume I: Version 20191213
    /// Section 2.7 - Memory Ordering Instructions
    fn rv32i_fence(&mut self) -> Result<(), Error> {
        // For a simple single-threaded implementation, FENCE acts as NOP
        // In a more complex implementation, this would ensure memory ordering
        self.increment_pc()
    }

    /// ECALL (Environment Call) is used to make a service request to the
    /// execution environment. In a real system, this would trap to the OS.
    ///
    /// Reference: RISC-V ISA Manual, Volume I: Version 20191213
    /// Section 2.8 - Environment Call and Breakpoints
    fn rv32i_ecall(&mut self) -> Result<(), Error> {
        // For now, we'll treat this as an unhandled system call
        // In a real implementation, this would:
        // 1. Save PC to a CSR (mepc/sepc/uepc)
        // 2. Set mcause/scause/ucause to indicate an environment call
        // 3. Transfer control to the trap handler
        // For educational purposes, we'll return a specific error
        Err(Error::EnvironmentCall)
    }

    /// EBREAK is used to return control to a debugging environment.
    ///
    /// Reference: RISC-V ISA Manual, Volume I: Version 20191213
    /// Section 2.8 - Environment Call and Breakpoints
    fn rv32i_ebreak(&mut self) -> Result<(), Error> {
        // For now, we'll treat this as a breakpoint trap
        // In a real implementation, this would:
        // 1. Save PC to a CSR (mepc/sepc/uepc)
        // 2. Set mcause/scause/ucause to indicate a breakpoint
        // 3. Transfer control to the debugger
        // For educational purposes, we'll return a specific error
        Err(Error::Breakpoint)
    }

    /// CSR Instructions - Control and Status Register Operations
    /// Reference: RISC-V ISA Manual, Chapter 9 "Zicsr" Extension
    ///
    /// CSRRW (Atomic Read/Write CSR)
    /// Atomically swaps values in the CSRs and integer registers.
    /// Old CSR value → rd, rs1 → CSR
    /// If rd=x0, then the instruction shall not read the CSR (avoids side effects)
    fn rv32i_csrrw(&mut self, instruction: IType) -> Result<(), Error> {
        // For CSR instructions, the immediate field contains the CSR address (12 bits, unsigned)
        // We need to mask off the sign extension that was applied during parsing
        let csr_addr = (instruction.imm.as_u32() & 0xFFF) as u16;
        let rs1_value = self.get_register(instruction.rs1);

        // Check if CSR exists
        if csr_addr >= 4096 || !self.csr_exists[csr_addr as usize] {
            return Err(Error::IllegalInstruction(format!(
                "CSR address 0x{csr_addr:03x} does not exist"
            )));
        }

        // Check if writing to read-only CSR
        if self.csr_readonly[csr_addr as usize] {
            return Err(Error::IllegalInstruction(format!(
                "CSR address 0x{csr_addr:03x} is read-only"
            )));
        }

        // Read old value only if rd != x0 (to avoid side effects)
        let old_value = if instruction.rd != Register::X0 {
            self.read_csr(csr_addr)?
        } else {
            0 // Don't actually read, just use dummy value
        };

        // Write new value
        self.write_csr(csr_addr, rs1_value)?;

        // Write old value to rd (only if rd != x0)
        if instruction.rd != Register::X0 {
            self.set_register(instruction.rd, old_value);
        }

        Ok(())
    }

    /// CSRRS (Atomic Read and Set Bits in CSR)
    /// Reads the value of the CSR, then sets bits based on rs1.
    /// Old CSR value → rd, CSR | rs1 → CSR
    /// If rs1=x0, then the instruction will not write to the CSR (avoids side effects)
    fn rv32i_csrrs(&mut self, instruction: IType) -> Result<(), Error> {
        // For CSR instructions, the immediate field contains the CSR address (12 bits, unsigned)
        // We need to mask off the sign extension that was applied during parsing
        let csr_addr = (instruction.imm.as_u32() & 0xFFF) as u16;
        let rs1_value = self.get_register(instruction.rs1);

        // Check if CSR exists
        if csr_addr >= 4096 || !self.csr_exists[csr_addr as usize] {
            return Err(Error::IllegalInstruction(format!(
                "CSR address 0x{csr_addr:03x} does not exist"
            )));
        }

        // Always read the CSR value
        let old_value = self.read_csr(csr_addr)?;

        // Write old value to rd
        self.set_register(instruction.rd, old_value);

        // Set bits only if rs1 != x0 (to avoid side effects)
        if instruction.rs1 != Register::X0 {
            // Check if writing to read-only CSR
            if self.csr_readonly[csr_addr as usize] {
                return Err(Error::IllegalInstruction(format!(
                    "CSR address 0x{csr_addr:03x} is read-only"
                )));
            }

            let new_value = old_value | rs1_value;
            self.write_csr(csr_addr, new_value)?;
        }

        Ok(())
    }

    /// CSRRC (Atomic Read and Clear Bits in CSR)
    /// Reads the value of the CSR, then clears bits based on rs1.
    /// Old CSR value → rd, CSR & ~rs1 → CSR
    /// If rs1=x0, then the instruction will not write to the CSR (avoids side effects)
    fn rv32i_csrrc(&mut self, instruction: IType) -> Result<(), Error> {
        // For CSR instructions, the immediate field contains the CSR address (12 bits, unsigned)
        // We need to mask off the sign extension that was applied during parsing
        let csr_addr = (instruction.imm.as_u32() & 0xFFF) as u16;
        let rs1_value = self.get_register(instruction.rs1);

        // Check if CSR exists
        if csr_addr >= 4096 || !self.csr_exists[csr_addr as usize] {
            return Err(Error::IllegalInstruction(format!(
                "CSR address 0x{csr_addr:03x} does not exist"
            )));
        }

        // Always read the CSR value
        let old_value = self.read_csr(csr_addr)?;

        // Write old value to rd
        self.set_register(instruction.rd, old_value);

        // Clear bits only if rs1 != x0 (to avoid side effects)
        if instruction.rs1 != Register::X0 {
            // Check if writing to read-only CSR
            if self.csr_readonly[csr_addr as usize] {
                return Err(Error::IllegalInstruction(format!(
                    "CSR address 0x{csr_addr:03x} is read-only"
                )));
            }

            let new_value = old_value & !rs1_value;
            self.write_csr(csr_addr, new_value)?;
        }

        Ok(())
    }

    /// CSRRWI (Atomic Read/Write CSR Immediate)
    /// Atomically writes a zero-extended 5-bit immediate to a CSR.
    /// Old CSR value → rd, zero-extended uimm → CSR
    /// If rd=x0, then the instruction shall not read the CSR (avoids side effects)
    fn rv32i_csrrwi(&mut self, instruction: IType) -> Result<(), Error> {
        // For CSR instructions, the immediate field contains the CSR address (12 bits, unsigned)
        // We need to mask off the sign extension that was applied during parsing
        let csr_addr = (instruction.imm.as_u32() & 0xFFF) as u16;
        // Extract 5-bit immediate from rs1 field (bits 19-15 of instruction)
        // For CSR immediate instructions, rs1 contains the immediate value, not a register
        let uimm = instruction.rs1.to_u32() & 0x1F;

        // Check if CSR exists
        if csr_addr >= 4096 || !self.csr_exists[csr_addr as usize] {
            return Err(Error::IllegalInstruction(format!(
                "CSR address 0x{csr_addr:03x} does not exist"
            )));
        }

        // Check if writing to read-only CSR
        if self.csr_readonly[csr_addr as usize] {
            return Err(Error::IllegalInstruction(format!(
                "CSR address 0x{csr_addr:03x} is read-only"
            )));
        }

        // Read old value only if rd != x0 (to avoid side effects)
        let old_value = if instruction.rd != Register::X0 {
            self.read_csr(csr_addr)?
        } else {
            0 // Don't actually read, just use dummy value
        };

        // Write immediate value (zero-extended)
        self.write_csr(csr_addr, uimm)?;

        // Write old value to rd (only if rd != x0)
        if instruction.rd != Register::X0 {
            self.set_register(instruction.rd, old_value);
        }

        Ok(())
    }

    /// CSRRSI (Atomic Read and Set Bits in CSR Immediate)
    /// Reads the value of the CSR, then sets bits based on 5-bit immediate.
    /// Old CSR value → rd, CSR | zero-extended uimm → CSR
    /// If uimm=0, then the instruction will not write to the CSR (avoids side effects)
    fn rv32i_csrrsi(&mut self, instruction: IType) -> Result<(), Error> {
        // For CSR instructions, the immediate field contains the CSR address (12 bits, unsigned)
        // We need to mask off the sign extension that was applied during parsing
        let csr_addr = (instruction.imm.as_u32() & 0xFFF) as u16;
        // Extract 5-bit immediate from rs1 field
        let uimm = instruction.rs1.to_u32() & 0x1F;

        // Check if CSR exists
        if csr_addr >= 4096 || !self.csr_exists[csr_addr as usize] {
            return Err(Error::IllegalInstruction(format!(
                "CSR address 0x{csr_addr:03x} does not exist"
            )));
        }

        // Always read the CSR value
        let old_value = self.read_csr(csr_addr)?;

        // Write old value to rd
        self.set_register(instruction.rd, old_value);

        // Set bits only if uimm != 0 (to avoid side effects)
        if uimm != 0 {
            // Check if writing to read-only CSR
            if self.csr_readonly[csr_addr as usize] {
                return Err(Error::IllegalInstruction(format!(
                    "CSR address 0x{csr_addr:03x} is read-only"
                )));
            }

            let new_value = old_value | uimm;
            self.write_csr(csr_addr, new_value)?;
        }

        Ok(())
    }

    /// CSRRCI (Atomic Read and Clear Bits in CSR Immediate)
    /// Reads the value of the CSR, then clears bits based on 5-bit immediate.
    /// Old CSR value → rd, CSR & ~zero-extended uimm → CSR
    /// If uimm=0, then the instruction will not write to the CSR (avoids side effects)
    fn rv32i_csrrci(&mut self, instruction: IType) -> Result<(), Error> {
        // For CSR instructions, the immediate field contains the CSR address (12 bits, unsigned)
        // We need to mask off the sign extension that was applied during parsing
        let csr_addr = (instruction.imm.as_u32() & 0xFFF) as u16;
        // Extract 5-bit immediate from rs1 field
        let uimm = instruction.rs1.to_u32() & 0x1F;

        // Check if CSR exists
        if csr_addr >= 4096 || !self.csr_exists[csr_addr as usize] {
            return Err(Error::IllegalInstruction(format!(
                "CSR address 0x{csr_addr:03x} does not exist"
            )));
        }

        // Always read the CSR value
        let old_value = self.read_csr(csr_addr)?;

        // Write old value to rd
        self.set_register(instruction.rd, old_value);

        // Clear bits only if uimm != 0 (to avoid side effects)
        if uimm != 0 {
            // Check if writing to read-only CSR
            if self.csr_readonly[csr_addr as usize] {
                return Err(Error::IllegalInstruction(format!(
                    "CSR address 0x{csr_addr:03x} is read-only"
                )));
            }

            let new_value = old_value & !uimm;
            self.write_csr(csr_addr, new_value)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    MisalignedJump(u32),
    AccessViolation(u32),
    EnvironmentCall,
    Breakpoint,
    IllegalInstruction(String),
}

impl CPU {
    /// Gets all 32 registers as an array (for state capture)
    #[cfg(feature = "repl")]
    pub fn get_all_registers(&self) -> [u32; 32] {
        [
            self.x0, self.x1, self.x2, self.x3, self.x4, self.x5, self.x6, self.x7, self.x8,
            self.x9, self.x10, self.x11, self.x12, self.x13, self.x14, self.x15, self.x16,
            self.x17, self.x18, self.x19, self.x20, self.x21, self.x22, self.x23, self.x24,
            self.x25, self.x26, self.x27, self.x28, self.x29, self.x30, self.x31,
        ]
    }

    /// Sets all 32 registers from an array (for state restoration)
    #[cfg(feature = "repl")]
    pub fn set_all_registers(&mut self, regs: &[u32; 32]) {
        self.x0 = 0; // x0 is always zero
        self.x1 = regs[1];
        self.x2 = regs[2];
        self.x3 = regs[3];
        self.x4 = regs[4];
        self.x5 = regs[5];
        self.x6 = regs[6];
        self.x7 = regs[7];
        self.x8 = regs[8];
        self.x9 = regs[9];
        self.x10 = regs[10];
        self.x11 = regs[11];
        self.x12 = regs[12];
        self.x13 = regs[13];
        self.x14 = regs[14];
        self.x15 = regs[15];
        self.x16 = regs[16];
        self.x17 = regs[17];
        self.x18 = regs[18];
        self.x19 = regs[19];
        self.x20 = regs[20];
        self.x21 = regs[21];
        self.x22 = regs[22];
        self.x23 = regs[23];
        self.x24 = regs[24];
        self.x25 = regs[25];
        self.x26 = regs[26];
        self.x27 = regs[27];
        self.x28 = regs[28];
        self.x29 = regs[29];
        self.x30 = regs[30];
        self.x31 = regs[31];
    }

    /// Clears the tracking vectors (call before each instruction)
    #[cfg(feature = "repl")]
    pub fn clear_tracking(&mut self) {
        self.memory_changes.clear();
        self.csr_changes.clear();
        self.register_changes.clear();
    }

    /// Resets the CPU to its initial state
    pub fn reset(&mut self) {
        // Reset all registers to 0
        self.x0 = 0;
        self.x1 = 0;
        self.x2 = 0;
        self.x3 = 0;
        self.x4 = 0;
        self.x5 = 0;
        self.x6 = 0;
        self.x7 = 0;
        self.x8 = 0;
        self.x9 = 0;
        self.x10 = 0;
        self.x11 = 0;
        self.x12 = 0;
        self.x13 = 0;
        self.x14 = 0;
        self.x15 = 0;
        self.x16 = 0;
        self.x17 = 0;
        self.x18 = 0;
        self.x19 = 0;
        self.x20 = 0;
        self.x21 = 0;
        self.x22 = 0;
        self.x23 = 0;
        self.x24 = 0;
        self.x25 = 0;
        self.x26 = 0;
        self.x27 = 0;
        self.x28 = 0;
        self.x29 = 0;
        self.x30 = 0;
        self.x31 = 0;
        self.pc = 0;

        // Clear memory
        self.memory.fill(0);

        // Reset CSRs
        self.csrs = [0; 4096];
        self.csr_exists = [false; 4096];
        self.csr_readonly = [false; 4096];
        self.init_csrs();

        // Clear tracking if REPL feature is enabled
        #[cfg(feature = "repl")]
        self.clear_tracking();
    }

    /// Restores memory from a set of deltas (for undo)
    #[cfg(feature = "repl")]
    pub fn restore_memory(&mut self, deltas: &[crate::history::MemoryDelta]) {
        for delta in deltas {
            if (delta.address as usize) < self.memory.len() {
                self.memory[delta.address as usize] = delta.old_value;
            }
        }
    }

    /// Restores CSRs from a set of changes (for undo)
    #[cfg(feature = "repl")]
    pub fn restore_csrs(&mut self, changes: &[(u32, u32, u32)]) {
        for &(addr, old_value, _new_value) in changes {
            if (addr as usize) < 4096 && self.csr_exists[addr as usize] {
                self.csrs[addr as usize] = old_value;
            }
        }
    }
}

