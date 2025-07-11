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

/// Represents a change to memory at a specific address
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDelta {
    pub addr: u32,
    pub old_data: Vec<u8>,
    pub new_data: Vec<u8>,
}

/// Records what changed during instruction execution (observation)
#[derive(Debug, Clone, PartialEq)]
pub struct StateDelta {
    pub register_changes: Vec<(Register, u32, u32)>, // (reg, old, new)
    pub memory_changes: Vec<MemoryDelta>,
    pub csr_changes: Vec<(u32, u32, u32)>, // (addr, old, new)
    pub pc_change: (u32, u32),             // (old, new)
}

/// Specifies what changes to apply to CPU state (command)
#[derive(Debug, Clone, PartialEq)]
pub struct Modify {
    pub register_changes: Vec<(Register, u32)>, // (reg, new_value)
    pub memory_changes: Vec<(u32, Vec<u8>)>,    // (addr, new_data)
    pub csr_changes: Vec<(u32, u32)>,           // (addr, new_value)
    pub pc_change: Option<u32>,                 // new_pc
}

impl StateDelta {
    /// Creates a new empty state delta
    pub fn new() -> Self {
        Self {
            register_changes: Vec::new(),
            memory_changes: Vec::new(),
            csr_changes: Vec::new(),
            pc_change: (0, 0),
        }
    }

    /// Converts this delta to a Modify that will reverse the changes
    pub fn to_reverse_modify(&self) -> Modify {
        Modify {
            register_changes: self
                .register_changes
                .iter()
                .map(|(reg, old, _new)| (*reg, *old))
                .collect(),
            memory_changes: self
                .memory_changes
                .iter()
                .map(|md| (md.addr, md.old_data.clone()))
                .collect(),
            csr_changes: self
                .csr_changes
                .iter()
                .map(|(addr, old, _new)| (*addr, *old))
                .collect(),
            pc_change: Some(self.pc_change.0),
        }
    }

    /// Converts this delta to a Modify that will apply the changes
    pub fn to_forward_modify(&self) -> Modify {
        Modify {
            register_changes: self
                .register_changes
                .iter()
                .map(|(reg, _old, new)| (*reg, *new))
                .collect(),
            memory_changes: self
                .memory_changes
                .iter()
                .map(|md| (md.addr, md.new_data.clone()))
                .collect(),
            csr_changes: self
                .csr_changes
                .iter()
                .map(|(addr, _old, new)| (*addr, *new))
                .collect(),
            pc_change: Some(self.pc_change.1),
        }
    }
}

impl Default for StateDelta {
    fn default() -> Self {
        Self::new()
    }
}

impl Modify {
    /// Creates a new empty modify command
    pub fn new() -> Self {
        Self {
            register_changes: Vec::new(),
            memory_changes: Vec::new(),
            csr_changes: Vec::new(),
            pc_change: None,
        }
    }
}

impl Default for Modify {
    fn default() -> Self {
        Self::new()
    }
}

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
        self.set_register_direct(r, v);
    }

    /// Sets a register directly without any tracking (internal use)
    fn set_register_direct(&mut self, r: Register, v: u32) {
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
    }

    /// Gets all 32 registers as an array (internal use)
    fn get_all_registers_internal(&self) -> [u32; 32] {
        [
            self.x0, self.x1, self.x2, self.x3, self.x4, self.x5, self.x6, self.x7, self.x8,
            self.x9, self.x10, self.x11, self.x12, self.x13, self.x14, self.x15, self.x16,
            self.x17, self.x18, self.x19, self.x20, self.x21, self.x22, self.x23, self.x24,
            self.x25, self.x26, self.x27, self.x28, self.x29, self.x30, self.x31,
        ]
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
    pub fn read_csr(&self, addr: u16) -> Result<u32, CPUError> {
        if addr >= 4096 || !self.csr_exists[addr as usize] {
            return Err(CPUError::IllegalInstruction(format!(
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
    pub fn write_csr(&mut self, addr: u16, value: u32) -> Result<u32, CPUError> {
        if addr >= 4096 || !self.csr_exists[addr as usize] {
            return Err(CPUError::IllegalInstruction(format!(
                "CSR address 0x{addr:03x} does not exist"
            )));
        }

        if self.csr_readonly[addr as usize] {
            return Err(CPUError::IllegalInstruction(format!(
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

        self.csrs[addr as usize] = legal_value;
        Ok(old_value)
    }

    /// Set bits in a CSR (for CSRRS instruction)
    pub fn set_csr_bits(&mut self, addr: u16, mask: u32) -> Result<u32, CPUError> {
        let old_value = self.read_csr(addr)?;
        if mask != 0 {
            self.write_csr(addr, old_value | mask)?;
        }
        Ok(old_value)
    }

    /// Clear bits in a CSR (for CSRRC instruction)
    pub fn clear_csr_bits(&mut self, addr: u16, mask: u32) -> Result<u32, CPUError> {
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
    /// Apply a set of modifications to the CPU state atomically
    ///
    /// This method applies all the changes specified in the `Modify` struct
    /// and returns a `StateDelta` indicating what actually changed.
    pub fn apply(&mut self, modify: &Modify) -> Result<StateDelta, CPUError> {
        // First, validate all operations without making changes (atomic validation)

        // Validate memory changes
        for (addr, new_data) in &modify.memory_changes {
            let addr = *addr as usize;
            if addr + new_data.len() > self.memory.len() {
                return Err(CPUError::IllegalInstruction(format!(
                    "Memory modification out of bounds: address 0x{:x} + {} bytes",
                    addr,
                    new_data.len()
                )));
            }
        }

        // Validate CSR changes
        for (addr, _new_val) in &modify.csr_changes {
            let addr = *addr as u16;
            if addr >= 4096 || !self.csr_exists[addr as usize] {
                return Err(CPUError::IllegalInstruction(format!(
                    "CSR address 0x{addr:03x} does not exist"
                )));
            }
        }

        // All validations passed, now apply changes atomically
        let mut delta = StateDelta::new();

        // Apply register changes
        for (reg, new_val) in &modify.register_changes {
            let old_val = self.get_register(*reg);
            self.set_register_direct(*reg, *new_val);
            if old_val != *new_val {
                delta.register_changes.push((*reg, old_val, *new_val));
            }
        }

        // Apply memory changes
        for (addr, new_data) in &modify.memory_changes {
            let addr = *addr as usize;
            let old_data = self.memory[addr..addr + new_data.len()].to_vec();
            if old_data != *new_data {
                self.memory[addr..addr + new_data.len()].copy_from_slice(new_data);
                delta.memory_changes.push(MemoryDelta {
                    addr: addr as u32,
                    old_data,
                    new_data: new_data.clone(),
                });
            }
        }

        // Apply CSR changes
        for (addr, new_val) in &modify.csr_changes {
            let old_val = self.read_csr(*addr as u16)?;
            self.write_csr(*addr as u16, *new_val)?;
            if old_val != *new_val {
                delta.csr_changes.push((*addr, old_val, *new_val));
            }
        }

        // Apply PC change
        if let Some(new_pc) = modify.pc_change {
            let old_pc = self.pc;
            self.pc = new_pc;
            delta.pc_change = (old_pc, new_pc);
        }

        Ok(delta)
    }

    /// Executes a RISC-V instruction and returns detailed information about what changed.
    ///
    /// This method provides complete visibility into instruction execution by returning
    /// a `StateDelta` that records all state changes: register modifications, memory writes,
    /// CSR updates, and PC changes. This enables powerful features like:
    ///
    /// - **Perfect undo/redo**: Use `StateDelta.to_reverse_modify()` with `apply()`
    /// - **Debugging**: See exactly what each instruction modified
    /// - **Scripting**: Compose complex state changes atomically
    /// - **Analysis**: Track program behavior over time
    ///
    /// # Example
    /// ```
    /// use brubeck::rv32_i::{CPU, Instruction, Register, IType};
    /// use brubeck::Immediate;
    ///
    /// let mut cpu = CPU::default();
    /// // ADDI x1, x0, 42
    /// let mut itype = IType::new();
    /// itype.opcode = 0b0010011;  // ADDI opcode
    /// itype.funct3 = 0b000;      // ADDI funct3
    /// itype.rd = Register::X1;
    /// itype.rs1 = Register::X0;
    /// itype.imm.set_signed(42);
    /// let inst = Instruction::ADDI(itype);
    /// let delta = cpu.execute(inst)?;
    ///
    /// // See what changed
    /// println!("Modified {} registers", delta.register_changes.len());
    ///
    /// // Perfect undo
    /// let undo = delta.to_reverse_modify();
    /// cpu.apply(&undo)?;
    /// # Ok::<(), brubeck::rv32_i::CPUError>(())
    /// ```
    pub fn execute(&mut self, instruction: Instruction) -> Result<StateDelta, CPUError> {
        // Capture state before execution
        let old_pc = self.pc;
        let old_registers = self.get_all_registers_internal();
        let old_memory = self.memory.clone(); // This is expensive but simple for now
        let old_csrs = self.csrs;

        // Execute the instruction using the existing logic
        let result = self.execute_internal(instruction);

        // If execution failed, don't return a delta
        result?;

        // Capture state after execution and build delta
        let mut delta = StateDelta::new();
        delta.pc_change = (old_pc, self.pc);

        // Track register changes
        let new_registers = self.get_all_registers_internal();
        for (i, (&old_val, &new_val)) in old_registers.iter().zip(new_registers.iter()).enumerate()
        {
            if old_val != new_val {
                delta
                    .register_changes
                    .push((Register::from_u32(i as u32), old_val, new_val));
            }
        }

        // Track memory changes (simplified - this could be optimized)
        for (i, (&old_byte, &new_byte)) in old_memory.iter().zip(self.memory.iter()).enumerate() {
            if old_byte != new_byte {
                delta.memory_changes.push(MemoryDelta {
                    addr: i as u32,
                    old_data: vec![old_byte],
                    new_data: vec![new_byte],
                });
            }
        }

        // Track CSR changes
        for (i, (&old_val, &new_val)) in old_csrs.iter().zip(self.csrs.iter()).enumerate() {
            if old_val != new_val {
                delta.csr_changes.push((i as u32, old_val, new_val));
            }
        }

        Ok(delta)
    }

    /// Internal execute method (renamed from the original execute)
    fn execute_internal(&mut self, instruction: Instruction) -> Result<(), CPUError> {
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

    fn increment_pc(&mut self) -> Result<(), CPUError> {
        self.pc += Instruction::LENGTH;
        Ok(())
    }

    fn rv32i_nop(&mut self) -> Result<(), CPUError> {
        self.increment_pc()
    }

    /// ADD and SUB perform addition and subtraction respectively. Overflows
    /// are ignored and the low XLEN bits of results are written to the
    /// destination.
    fn rv32i_add(&mut self, instruction: RType) -> Result<(), CPUError> {
        let a = self.get_register(instruction.rs1);
        let b = self.get_register(instruction.rs2);
        self.set_register(instruction.rd, a.wrapping_add(b));
        self.increment_pc()
    }

    fn rv32i_sub(&mut self, instruction: RType) -> Result<(), CPUError> {
        let a = self.get_register(instruction.rs1);
        let b = self.get_register(instruction.rs2);
        self.set_register(instruction.rd, a.wrapping_sub(b));
        self.increment_pc()
    }

    /// ADDI adds the sign-extended 12-bit immediate to register rs1. Arithmetic
    /// overflow is ignored and the result is simply the low XLEN bits of the
    /// result. ADDI rd, rs1, 0 is used to implement the MV rd, rs1 assembler
    /// pseudo-instruction.
    fn rv32i_addi(&mut self, instruction: IType) -> Result<(), CPUError> {
        let rs1 = self.get_register(instruction.rs1);
        let imm = instruction.imm.as_u32();

        let new_value = rs1.wrapping_add(imm);

        self.set_register(instruction.rd, new_value);
        self.increment_pc()
    }

    /// SLTI (set less than immediate) places the value 1 in register rd if
    /// register rs1 is less than the sign-extended immediate when both are
    /// treated as signed numbers, else 0 is written to rd.
    fn rv32i_slti(&mut self, instruction: IType) -> Result<(), CPUError> {
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
    fn rv32i_sltiu(&mut self, instruction: IType) -> Result<(), CPUError> {
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
    fn rv32i_andi(&mut self, instruction: IType) -> Result<(), CPUError> {
        let rs1 = self.get_register(instruction.rs1);
        let imm = instruction.imm.as_u32();

        let value = imm & rs1;
        self.set_register(instruction.rd, value);

        self.increment_pc()
    }

    fn rv32i_ori(&mut self, instruction: IType) -> Result<(), CPUError> {
        let rs1 = self.get_register(instruction.rs1);
        let imm = instruction.imm.as_u32();

        let value = imm | rs1;
        self.set_register(instruction.rd, value);

        self.increment_pc()
    }

    fn rv32i_xori(&mut self, instruction: IType) -> Result<(), CPUError> {
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
    fn rv32i_lui(&mut self, instruction: UType) -> Result<(), CPUError> {
        let mut imm = instruction.imm.as_u32();
        imm <<= 12;
        self.set_register(instruction.rd, imm);

        self.increment_pc()
    }

    /// AUIPC (add upper immediate to pc) is used to build pc-relative
    /// addresses and uses the U-type format. AUIPC forms a 32-bit offset from
    /// the 20-bit U-immediate, filling in the lowest 12 bits with zeros, adds
    /// this offset to the pc, then places the result in register rd.
    fn rv32i_auipc(&mut self, instruction: UType) -> Result<(), CPUError> {
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
    fn rv32i_slt(&mut self, instruction: RType) -> Result<(), CPUError> {
        let rs1 = self.get_register(instruction.rs1) as i32;
        let rs2 = self.get_register(instruction.rs2) as i32;

        if rs1 < rs2 {
            self.set_register(instruction.rd, 1);
        } else {
            self.set_register(instruction.rd, 0);
        }

        self.increment_pc()
    }

    fn rv32i_sltu(&mut self, instruction: RType) -> Result<(), CPUError> {
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
    fn rv32i_and(&mut self, instruction: RType) -> Result<(), CPUError> {
        let rs1 = self.get_register(instruction.rs1);
        let rs2 = self.get_register(instruction.rs2);

        let value = rs1 & rs2;
        self.set_register(instruction.rd, value);

        self.increment_pc()
    }

    fn rv32i_or(&mut self, instruction: RType) -> Result<(), CPUError> {
        let rs1 = self.get_register(instruction.rs1);
        let rs2 = self.get_register(instruction.rs2);

        let value = rs1 | rs2;
        self.set_register(instruction.rd, value);

        self.increment_pc()
    }

    fn rv32i_xor(&mut self, instruction: RType) -> Result<(), CPUError> {
        let rs1 = self.get_register(instruction.rs1);
        let rs2 = self.get_register(instruction.rs2);

        let value = rs1 ^ rs2;
        self.set_register(instruction.rd, value);

        self.increment_pc()
    }

    /// SLL, SRL, and SRA perform logical left, logical right, and arithmetic
    /// right shifts on the value in register rs1 by the shift amount held in
    /// the lower 5 bits of register rs2.
    fn rv32i_sll(&mut self, instruction: RType) -> Result<(), CPUError> {
        let rs1 = self.get_register(instruction.rs1);
        let rs2 = self.get_register(instruction.rs2);

        // lower 5 bit register mask
        let mask = 0b0000_0000_0000_0000_0000_0000_0001_1111;
        let shift_amount = rs2 & mask;
        let value = rs1 << shift_amount;
        self.set_register(instruction.rd, value);

        self.increment_pc()
    }

    fn rv32i_srl(&mut self, instruction: RType) -> Result<(), CPUError> {
        let rs1 = self.get_register(instruction.rs1);
        let rs2 = self.get_register(instruction.rs2);

        // lower 5 bit register mask
        let mask = 0b0000_0000_0000_0000_0000_0000_0001_1111;
        let shift_amount = rs2 & mask;

        let value = rs1 >> shift_amount;
        self.set_register(instruction.rd, value);

        self.increment_pc()
    }

    fn rv32i_sra(&mut self, instruction: RType) -> Result<(), CPUError> {
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
    fn rv32i_slli(&mut self, instruction: IType) -> Result<(), CPUError> {
        let rs1 = self.get_register(instruction.rs1);
        let imm = instruction.imm.as_u32();

        let mask = 0b0000_0000_0000_0000_0000_0000_0001_1111;
        let shift_amount = imm & mask;

        let value = rs1 << shift_amount;
        self.set_register(instruction.rd, value);

        self.increment_pc()
    }

    fn rv32i_srli(&mut self, instruction: IType) -> Result<(), CPUError> {
        let rs1 = self.get_register(instruction.rs1);
        let imm = instruction.imm.as_u32();

        // lower 5 bit register mask
        let mask = 0b0000_0000_0000_0000_0000_0000_0001_1111;
        let shift_amount = imm & mask;

        let value = rs1 >> shift_amount;
        self.set_register(instruction.rd, value);

        self.increment_pc()
    }

    fn rv32i_srai(&mut self, instruction: IType) -> Result<(), CPUError> {
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
    fn rv32i_jal(&mut self, instruction: JType) -> Result<(), CPUError> {
        let mut offset = instruction.imm.as_u32();

        // shift left one bit; multiply by 2
        offset <<= 1;

        // create the offset address
        let offset_address = self.pc.wrapping_add(offset);

        // validate the offset address is 32-bit aligned
        if offset_address % 4 != 0 {
            return Err(CPUError::MisalignedJump(offset_address));
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
    fn rv32i_jalr(&mut self, instruction: IType) -> Result<(), CPUError> {
        let offset = instruction.imm.as_u32();
        let rs1 = self.get_register(instruction.rs1);

        let mut offset_address = rs1.wrapping_add(offset);

        // I'm sure there's a better way to zero the LSB
        offset_address >>= 1;
        offset_address <<= 1;

        // validate the offset address is 32-bit aligned
        if offset_address % 4 != 0 {
            return Err(CPUError::MisalignedJump(offset_address));
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
    fn rv32i_beq(&mut self, instruction: BType) -> Result<(), CPUError> {
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

    fn rv32i_bne(&mut self, instruction: BType) -> Result<(), CPUError> {
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
    fn rv32i_blt(&mut self, instruction: BType) -> Result<(), CPUError> {
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

    fn rv32i_bltu(&mut self, instruction: BType) -> Result<(), CPUError> {
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
    fn rv32i_bge(&mut self, instruction: BType) -> Result<(), CPUError> {
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

    fn rv32i_bgeu(&mut self, instruction: BType) -> Result<(), CPUError> {
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
    fn rv32i_lw(&mut self, instruction: IType) -> Result<(), CPUError> {
        let rs1 = self.get_register(instruction.rs1);
        let imm = instruction.imm.as_u32();

        let offset = rs1.wrapping_add(imm);
        let index = offset as usize;

        if index >= self.memory.len() {
            return Err(CPUError::AccessViolation(rs1));
        }

        let mut value_buf = [0u8; 4];
        value_buf.clone_from_slice(&self.memory[index..index + 4]);
        let value = u32::from_le_bytes(value_buf);

        self.set_register(instruction.rd, value);
        self.increment_pc()
    }

    /// LH loads a 16-bit value from memory, then sign-extends to 32-bits before
    /// storing in rd.
    fn rv32i_lh(&mut self, instruction: IType) -> Result<(), CPUError> {
        let rs1 = self.get_register(instruction.rs1);
        let imm = instruction.imm.as_u32();

        let offset = rs1.wrapping_add(imm);
        let index = offset as usize;

        if index >= self.memory.len() {
            return Err(CPUError::AccessViolation(rs1));
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
    fn rv32i_lhu(&mut self, instruction: IType) -> Result<(), CPUError> {
        let rs1 = self.get_register(instruction.rs1);
        let imm = instruction.imm.as_u32();

        let offset = rs1.wrapping_add(imm);
        let index = offset as usize;

        if index >= self.memory.len() {
            return Err(CPUError::AccessViolation(rs1));
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
    fn rv32i_lb(&mut self, instruction: IType) -> Result<(), CPUError> {
        let rs1 = self.get_register(instruction.rs1);
        let imm = instruction.imm.as_u32();

        let offset = rs1.wrapping_add(imm);
        let index = offset as usize;

        if index >= self.memory.len() {
            return Err(CPUError::AccessViolation(rs1));
        }

        let u8_value = self.memory[index];
        // Sign-extend from 8-bit to 32-bit
        let value = u8_value as i8 as i32 as u32;

        self.set_register(instruction.rd, value);
        self.increment_pc()
    }

    /// LBU loads a 8-bit value from memory but then zero extends to 32-bits
    /// before storing in rd.
    fn rv32i_lbu(&mut self, instruction: IType) -> Result<(), CPUError> {
        let rs1 = self.get_register(instruction.rs1);
        let imm = instruction.imm.as_u32();

        let offset = rs1.wrapping_add(imm);
        let index = offset as usize;

        if index >= self.memory.len() {
            return Err(CPUError::AccessViolation(rs1));
        }

        let u8_value = self.memory[index];
        let value = u8_value as u32;

        self.set_register(instruction.rd, value);
        self.increment_pc()
    }

    /// The SW, SH, and SB instructions store 32-bit, 16-bit, and 8-bit values
    /// from the low bits of register rs2 to memory
    fn rv32i_sw(&mut self, instruction: SType) -> Result<(), CPUError> {
        self.store(instruction, 4)?;
        self.increment_pc()
    }

    fn rv32i_sh(&mut self, instruction: SType) -> Result<(), CPUError> {
        self.store(instruction, 2)?;
        self.increment_pc()
    }

    fn rv32i_sb(&mut self, instruction: SType) -> Result<(), CPUError> {
        self.store(instruction, 1)?;
        self.increment_pc()
    }

    fn store(&mut self, instruction: SType, bytes: usize) -> Result<(), CPUError> {
        let base = self.get_register(instruction.rs1);
        let src = self.get_register(instruction.rs2);
        let imm = instruction.imm.as_u32();

        let address = base.wrapping_add(imm);
        let mut index = address as usize;

        if index >= self.memory.len() {
            return Err(CPUError::AccessViolation(address));
        }

        for (byte_index, byte) in src.to_le_bytes().into_iter().enumerate() {
            if byte_index < bytes {
                self.memory[index] = byte;
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
    fn rv32i_fence(&mut self) -> Result<(), CPUError> {
        // For a simple single-threaded implementation, FENCE acts as NOP
        // In a more complex implementation, this would ensure memory ordering
        self.increment_pc()
    }

    /// ECALL (Environment Call) is used to make a service request to the
    /// execution environment. In a real system, this would trap to the OS.
    ///
    /// Reference: RISC-V ISA Manual, Volume I: Version 20191213
    /// Section 2.8 - Environment Call and Breakpoints
    fn rv32i_ecall(&mut self) -> Result<(), CPUError> {
        // For now, we'll treat this as an unhandled system call
        // In a real implementation, this would:
        // 1. Save PC to a CSR (mepc/sepc/uepc)
        // 2. Set mcause/scause/ucause to indicate an environment call
        // 3. Transfer control to the trap handler
        // For educational purposes, we'll return a specific error
        Err(CPUError::EnvironmentCall)
    }

    /// EBREAK is used to return control to a debugging environment.
    ///
    /// Reference: RISC-V ISA Manual, Volume I: Version 20191213
    /// Section 2.8 - Environment Call and Breakpoints
    fn rv32i_ebreak(&mut self) -> Result<(), CPUError> {
        // For now, we'll treat this as a breakpoint trap
        // In a real implementation, this would:
        // 1. Save PC to a CSR (mepc/sepc/uepc)
        // 2. Set mcause/scause/ucause to indicate a breakpoint
        // 3. Transfer control to the debugger
        // For educational purposes, we'll return a specific error
        Err(CPUError::Breakpoint)
    }

    /// CSR Instructions - Control and Status Register Operations
    /// Reference: RISC-V ISA Manual, Chapter 9 "Zicsr" Extension
    ///
    /// CSRRW (Atomic Read/Write CSR)
    /// Atomically swaps values in the CSRs and integer registers.
    /// Old CSR value → rd, rs1 → CSR
    /// If rd=x0, then the instruction shall not read the CSR (avoids side effects)
    fn rv32i_csrrw(&mut self, instruction: IType) -> Result<(), CPUError> {
        // For CSR instructions, the immediate field contains the CSR address (12 bits, unsigned)
        // We need to mask off the sign extension that was applied during parsing
        let csr_addr = (instruction.imm.as_u32() & 0xFFF) as u16;
        let rs1_value = self.get_register(instruction.rs1);

        // Check if CSR exists
        if csr_addr >= 4096 || !self.csr_exists[csr_addr as usize] {
            return Err(CPUError::IllegalInstruction(format!(
                "CSR address 0x{csr_addr:03x} does not exist"
            )));
        }

        // Check if writing to read-only CSR
        if self.csr_readonly[csr_addr as usize] {
            return Err(CPUError::IllegalInstruction(format!(
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
    fn rv32i_csrrs(&mut self, instruction: IType) -> Result<(), CPUError> {
        // For CSR instructions, the immediate field contains the CSR address (12 bits, unsigned)
        // We need to mask off the sign extension that was applied during parsing
        let csr_addr = (instruction.imm.as_u32() & 0xFFF) as u16;
        let rs1_value = self.get_register(instruction.rs1);

        // Check if CSR exists
        if csr_addr >= 4096 || !self.csr_exists[csr_addr as usize] {
            return Err(CPUError::IllegalInstruction(format!(
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
                return Err(CPUError::IllegalInstruction(format!(
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
    fn rv32i_csrrc(&mut self, instruction: IType) -> Result<(), CPUError> {
        // For CSR instructions, the immediate field contains the CSR address (12 bits, unsigned)
        // We need to mask off the sign extension that was applied during parsing
        let csr_addr = (instruction.imm.as_u32() & 0xFFF) as u16;
        let rs1_value = self.get_register(instruction.rs1);

        // Check if CSR exists
        if csr_addr >= 4096 || !self.csr_exists[csr_addr as usize] {
            return Err(CPUError::IllegalInstruction(format!(
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
                return Err(CPUError::IllegalInstruction(format!(
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
    fn rv32i_csrrwi(&mut self, instruction: IType) -> Result<(), CPUError> {
        // For CSR instructions, the immediate field contains the CSR address (12 bits, unsigned)
        // We need to mask off the sign extension that was applied during parsing
        let csr_addr = (instruction.imm.as_u32() & 0xFFF) as u16;
        // Extract 5-bit immediate from rs1 field (bits 19-15 of instruction)
        // For CSR immediate instructions, rs1 contains the immediate value, not a register
        let uimm = instruction.rs1.to_u32() & 0x1F;

        // Check if CSR exists
        if csr_addr >= 4096 || !self.csr_exists[csr_addr as usize] {
            return Err(CPUError::IllegalInstruction(format!(
                "CSR address 0x{csr_addr:03x} does not exist"
            )));
        }

        // Check if writing to read-only CSR
        if self.csr_readonly[csr_addr as usize] {
            return Err(CPUError::IllegalInstruction(format!(
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
    fn rv32i_csrrsi(&mut self, instruction: IType) -> Result<(), CPUError> {
        // For CSR instructions, the immediate field contains the CSR address (12 bits, unsigned)
        // We need to mask off the sign extension that was applied during parsing
        let csr_addr = (instruction.imm.as_u32() & 0xFFF) as u16;
        // Extract 5-bit immediate from rs1 field
        let uimm = instruction.rs1.to_u32() & 0x1F;

        // Check if CSR exists
        if csr_addr >= 4096 || !self.csr_exists[csr_addr as usize] {
            return Err(CPUError::IllegalInstruction(format!(
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
                return Err(CPUError::IllegalInstruction(format!(
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
    fn rv32i_csrrci(&mut self, instruction: IType) -> Result<(), CPUError> {
        // For CSR instructions, the immediate field contains the CSR address (12 bits, unsigned)
        // We need to mask off the sign extension that was applied during parsing
        let csr_addr = (instruction.imm.as_u32() & 0xFFF) as u16;
        // Extract 5-bit immediate from rs1 field
        let uimm = instruction.rs1.to_u32() & 0x1F;

        // Check if CSR exists
        if csr_addr >= 4096 || !self.csr_exists[csr_addr as usize] {
            return Err(CPUError::IllegalInstruction(format!(
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
                return Err(CPUError::IllegalInstruction(format!(
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
pub enum CPUError {
    MisalignedJump(u32),
    AccessViolation(u32),
    EnvironmentCall,
    Breakpoint,
    IllegalInstruction(String),
}

impl std::fmt::Display for CPUError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CPUError::MisalignedJump(addr) => write!(f, "Misaligned jump to address 0x{addr:08x}"),
            CPUError::AccessViolation(addr) => {
                write!(f, "Memory address out of bounds: 0x{addr:08x}")
            }
            CPUError::EnvironmentCall => write!(f, "Environment call"),
            CPUError::Breakpoint => write!(f, "Breakpoint"),
            CPUError::IllegalInstruction(desc) => write!(f, "Illegal instruction: {desc}"),
        }
    }
}

impl CPU {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csr_initialization() {
        let cpu = CPU::default();

        // Verify user-level CSRs exist and are read-only
        assert!(cpu.csr_exists[0xC00]); // cycle
        assert!(cpu.csr_readonly[0xC00]);

        assert!(cpu.csr_exists[0xC01]); // time
        assert!(cpu.csr_readonly[0xC01]);

        assert!(cpu.csr_exists[0xC02]); // instret
        assert!(cpu.csr_readonly[0xC02]);

        // Verify machine-level CSRs exist
        assert!(cpu.csr_exists[0x300]); // mstatus
        assert!(!cpu.csr_readonly[0x300]); // mstatus is writable

        assert!(cpu.csr_exists[0x301]); // misa
        assert!(cpu.csr_readonly[0x301]); // misa is read-only

        // Verify mstatus initial value
        assert_eq!(cpu.csrs[0x300], 0x00001800); // MPP = 11

        // Verify misa initial value
        assert_eq!(cpu.csrs[0x301], 0x40000100); // RV32I
    }

    #[test]
    fn test_csr_read_basic() {
        let cpu = CPU::default();

        // Read existing CSR
        let mstatus = cpu.read_csr(0x300).unwrap();
        assert_eq!(mstatus, 0x00001800);

        // Read MISA
        let misa = cpu.read_csr(0x301).unwrap();
        assert_eq!(misa, 0x40000100);

        // Read dynamic CSRs (should return 0 for now)
        assert_eq!(cpu.read_csr(0xC00).unwrap(), 0); // cycle
        assert_eq!(cpu.read_csr(0xC01).unwrap(), 0); // time
        assert_eq!(cpu.read_csr(0xC02).unwrap(), 0); // instret
    }

    #[test]
    fn test_csr_read_nonexistent() {
        let cpu = CPU::default();

        // Try to read non-existent CSR
        let result = cpu.read_csr(0x999);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CPUError::IllegalInstruction(_)
        ));

        // Try to read at boundary
        let result = cpu.read_csr(0xFFF);
        assert!(result.is_err());

        // Try to read out of bounds
        let result = cpu.read_csr(0x1000);
        assert!(result.is_err());
    }

    #[test]
    fn test_csr_write_basic() {
        let mut cpu = CPU::default();

        // Write to writable CSR
        cpu.write_csr(0x340, 0xDEADBEEF).unwrap(); // mscratch
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0xDEADBEEF);

        // Write to another writable CSR
        cpu.write_csr(0x305, 0x12345678).unwrap(); // mtvec
        assert_eq!(cpu.read_csr(0x305).unwrap(), 0x12345678);
    }

    #[test]
    fn test_csr_write_readonly() {
        let mut cpu = CPU::default();

        // Try to write to read-only CSRs
        let result = cpu.write_csr(0xC00, 0x1234); // cycle is read-only
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CPUError::IllegalInstruction(_)
        ));

        let result = cpu.write_csr(0x301, 0x5678); // misa is read-only
        assert!(result.is_err());

        // Verify values didn't change
        assert_eq!(cpu.read_csr(0xC00).unwrap(), 0);
        assert_eq!(cpu.read_csr(0x301).unwrap(), 0x40000100);
    }

    #[test]
    fn test_csr_write_nonexistent() {
        let mut cpu = CPU::default();

        // Try to write to non-existent CSR
        let result = cpu.write_csr(0x999, 0x1234);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CPUError::IllegalInstruction(_)
        ));
    }

    #[test]
    fn test_csr_mstatus_warl() {
        let mut cpu = CPU::default();

        // mstatus has WARL behavior - only certain bits are writable
        // Initial value: 0x00001800
        // Writable mask: 0x00001888 (MIE bit 3, MPIE bit 7, MPP bits 11-12)

        // Try to write all bits
        cpu.write_csr(0x300, 0xFFFFFFFF).unwrap();

        // Only writable bits should change
        let mstatus = cpu.read_csr(0x300).unwrap();
        println!("After writing 0xFFFFFFFF, mstatus = 0x{mstatus:08x}");
        // Initial: 0x00001800 (MPP=11)
        // Mask:    0x00001888 (allows changing MIE, MPIE, MPP)
        // Result should be: 0x00001888 (all writable bits set)

        // Write specific pattern (clear all writable bits)
        cpu.write_csr(0x300, 0x00000000).unwrap();
        let mstatus = cpu.read_csr(0x300).unwrap();
        println!("After writing 0x00000000, mstatus = 0x{mstatus:08x}");
        assert_eq!(mstatus, 0x00000000); // All writable bits cleared
    }

    #[test]
    fn test_csr_set_bits() {
        let mut cpu = CPU::default();

        // Set bits in mscratch
        cpu.write_csr(0x340, 0x00FF00FF).unwrap();

        // Set additional bits
        let old = cpu.set_csr_bits(0x340, 0x0F0F0F0F).unwrap();
        assert_eq!(old, 0x00FF00FF); // Returns old value

        // Verify new value has bits set
        let new = cpu.read_csr(0x340).unwrap();
        assert_eq!(new, 0x0FFF0FFF); // OR of old and mask

        // Setting with mask 0 should not write
        let old = cpu.set_csr_bits(0x340, 0).unwrap();
        assert_eq!(old, 0x0FFF0FFF);
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0x0FFF0FFF); // Unchanged
    }

    #[test]
    fn test_csr_clear_bits() {
        let mut cpu = CPU::default();

        // Set initial value in mscratch
        cpu.write_csr(0x340, 0xFFFFFFFF).unwrap();

        // Clear some bits
        let old = cpu.clear_csr_bits(0x340, 0x0F0F0F0F).unwrap();
        assert_eq!(old, 0xFFFFFFFF); // Returns old value

        // Verify new value has bits cleared
        let new = cpu.read_csr(0x340).unwrap();
        assert_eq!(new, 0xF0F0F0F0); // AND with NOT mask

        // Clearing with mask 0 should not write
        let old = cpu.clear_csr_bits(0x340, 0).unwrap();
        assert_eq!(old, 0xF0F0F0F0);
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0xF0F0F0F0); // Unchanged
    }

    #[test]
    fn test_csr_set_clear_readonly() {
        let mut cpu = CPU::default();

        // Try to set bits in read-only CSR
        let result = cpu.set_csr_bits(0x301, 0xFF); // misa is read-only
        assert!(result.is_err());

        // Try to clear bits in read-only CSR
        let result = cpu.clear_csr_bits(0x301, 0xFF);
        assert!(result.is_err());

        // Verify value unchanged
        assert_eq!(cpu.read_csr(0x301).unwrap(), 0x40000100);
    }

    #[test]
    fn test_csr_all_machine_csrs() {
        let cpu = CPU::default();

        // Verify all machine CSRs we initialized exist
        assert!(cpu.csr_exists[0x300]); // mstatus
        assert!(cpu.csr_exists[0x301]); // misa
        assert!(cpu.csr_exists[0x304]); // mie
        assert!(cpu.csr_exists[0x305]); // mtvec
        assert!(cpu.csr_exists[0x340]); // mscratch
        assert!(cpu.csr_exists[0x341]); // mepc
        assert!(cpu.csr_exists[0x342]); // mcause
        assert!(cpu.csr_exists[0x343]); // mtval
        assert!(cpu.csr_exists[0x344]); // mip
    }

    #[test]
    fn test_csr_boundary_conditions() {
        let mut cpu = CPU::default();

        // Test CSR address 0
        assert!(!cpu.csr_exists[0]);
        assert!(cpu.read_csr(0).is_err());

        // Test maximum valid CSR address (0xFFF = 4095)
        assert!(!cpu.csr_exists[0xFFF]);
        assert!(cpu.read_csr(0xFFF).is_err());

        // Create a CSR at the boundary
        cpu.csr_exists[0xFFF] = true;
        cpu.csrs[0xFFF] = 0x12345678;

        // Should now be readable
        assert_eq!(cpu.read_csr(0xFFF).unwrap(), 0x12345678);

        // And writable
        cpu.write_csr(0xFFF, 0x87654321).unwrap();
        assert_eq!(cpu.read_csr(0xFFF).unwrap(), 0x87654321);
    }

    #[test]
    fn test_csr_bit_manipulation_edge_cases() {
        let mut cpu = CPU::default();

        // Test with all bits set
        cpu.write_csr(0x340, 0xFFFFFFFF).unwrap();
        cpu.set_csr_bits(0x340, 0xFFFFFFFF).unwrap();
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0xFFFFFFFF);

        // Clear all bits
        cpu.clear_csr_bits(0x340, 0xFFFFFFFF).unwrap();
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0);

        // Set pattern
        cpu.set_csr_bits(0x340, 0xAAAAAAAA).unwrap();
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0xAAAAAAAA);

        // Clear alternating pattern
        cpu.clear_csr_bits(0x340, 0x55555555).unwrap();
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0xAAAAAAAA); // No overlap

        // Clear overlapping pattern
        cpu.clear_csr_bits(0x340, 0xAAAAAAAA).unwrap();
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0);
    }

    // ===== CSR SPECIFICATION COMPLIANCE TESTS =====

    // Test the key spec requirement from section 2.1:
    // "If rd=x0, then the instruction shall not read the CSR and shall not
    // cause any of the side effects that might occur on a CSR read."
    #[test]
    fn test_csrrw_rd_x0_no_read() {
        let mut cpu = CPU::default();

        // Create a custom CSR that tracks reads (simulating side effects)
        let test_csr: u16 = 0x800;
        cpu.csr_exists[test_csr as usize] = true;
        cpu.csrs[test_csr as usize] = 0x12345678;

        // CSRRW with rd=x0 should NOT read the CSR
        // In a real implementation with side effects, this would be observable
        // For now, we just verify the operation succeeds
        cpu.write_csr(test_csr, 0xABCDEF00).unwrap();
        assert_eq!(cpu.read_csr(test_csr).unwrap(), 0xABCDEF00);
    }

    // Test from spec: "For both CSRRS and CSRRC, if rs1=x0, then the instruction
    // will not write to the CSR at all, and so shall not cause any of the side
    // effects that might otherwise occur on a CSR write, nor raise illegal-instruction
    // exceptions on accesses to read-only CSRs."
    #[test]
    fn test_csrrs_csrrc_rs1_x0_no_write() {
        let mut cpu = CPU::default();

        // Test with read-only CSR - should NOT raise exception when rs1=x0
        let old_misa = cpu.set_csr_bits(0x301, 0).unwrap(); // misa is read-only
        assert_eq!(old_misa, 0x40000100); // Should return old value
        assert_eq!(cpu.read_csr(0x301).unwrap(), 0x40000100); // Unchanged

        // Same for clear_bits
        let old_misa = cpu.clear_csr_bits(0x301, 0).unwrap();
        assert_eq!(old_misa, 0x40000100);
        assert_eq!(cpu.read_csr(0x301).unwrap(), 0x40000100);

        // But with non-zero mask, should fail on read-only
        assert!(cpu.set_csr_bits(0x301, 1).is_err());
        assert!(cpu.clear_csr_bits(0x301, 1).is_err());
    }

    // Test: "Note that if rs1 specifies a register other than x0, and that register
    // holds a zero value, the instruction will not action any attendant per-field
    // side effects, but will action any side effects caused by writing to the entire CSR."
    #[test]
    fn test_csrrs_csrrc_zero_value_behavior() {
        let mut cpu = CPU::default();

        // When rs1 != x0 but value is 0, write still happens
        // This is different from rs1 = x0 case!
        cpu.write_csr(0x340, 0xFFFFFFFF).unwrap();

        // This simulates CSRRS with rs1 containing 0
        // The write happens (triggering any CSR-write side effects)
        // but no bits change
        let old = cpu.set_csr_bits(0x340, 0).unwrap();
        assert_eq!(old, 0xFFFFFFFF);
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0xFFFFFFFF);
    }

    // Test: "A CSRRW with rs1=x0 will attempt to write zero to the destination CSR."
    #[test]
    fn test_csrrw_rs1_x0_writes_zero() {
        let mut cpu = CPU::default();

        // Set a non-zero value
        cpu.write_csr(0x340, 0xDEADBEEF).unwrap();

        // CSRRW with rs1=x0 writes 0
        cpu.write_csr(0x340, 0).unwrap();
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0);
    }

    // Test immediate instruction behavior
    #[test]
    fn test_immediate_variants_5bit() {
        let mut cpu = CPU::default();

        // Immediate values are 5-bit zero-extended
        // Max immediate value is 31 (0b11111)
        cpu.write_csr(0x340, 0).unwrap();

        // Simulate CSRRSI with uimm=31
        cpu.set_csr_bits(0x340, 31).unwrap();
        assert_eq!(cpu.read_csr(0x340).unwrap(), 31);

        // Clear lower 5 bits with immediate
        cpu.clear_csr_bits(0x340, 31).unwrap();
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0);
    }

    // Test: "CSR reads the value prior to the execution of the instruction"
    #[test]
    fn test_read_before_write_semantics() {
        let mut cpu = CPU::default();

        cpu.write_csr(0x340, 0x1234).unwrap();

        // All CSR instructions return the OLD value
        let old = cpu.write_csr(0x340, 0x5678).unwrap();
        assert_eq!(old, 0x1234); // Returns value before write

        let old = cpu.set_csr_bits(0x340, 0xFF00).unwrap();
        assert_eq!(old, 0x5678); // Returns value before set

        let old = cpu.clear_csr_bits(0x340, 0x00FF).unwrap();
        assert_eq!(old, 0xFF78); // Returns value before clear
    }

    // Test WARL behavior for specific fields
    #[test]
    fn test_warl_field_behavior() {
        let mut cpu = CPU::default();

        // Test mstatus WARL behavior more thoroughly
        // Initial: 0x00001800 (MPP=11)

        // Try to set all bits
        cpu.write_csr(0x300, 0xFFFFFFFF).unwrap();
        let mstatus = cpu.read_csr(0x300).unwrap();

        // Only MIE(3), MPIE(7), MPP(11-12) should be set
        assert_eq!(mstatus & 0x00000008, 0x00000008); // MIE set
        assert_eq!(mstatus & 0x00000080, 0x00000080); // MPIE set
        assert_eq!(mstatus & 0x00001800, 0x00001800); // MPP = 11

        // All other bits should be 0
        assert_eq!(mstatus & !0x00001888, 0);
    }

    // Test proper error handling for all error cases
    #[test]
    fn test_comprehensive_error_handling() {
        let mut cpu = CPU::default();

        // Non-existent CSR
        assert!(matches!(
            cpu.read_csr(0x999),
            Err(CPUError::IllegalInstruction(_))
        ));
        assert!(matches!(
            cpu.write_csr(0x999, 0),
            Err(CPUError::IllegalInstruction(_))
        ));
        assert!(matches!(
            cpu.set_csr_bits(0x999, 1),
            Err(CPUError::IllegalInstruction(_))
        ));
        assert!(matches!(
            cpu.clear_csr_bits(0x999, 1),
            Err(CPUError::IllegalInstruction(_))
        ));

        // Out of bounds CSR address
        assert!(matches!(
            cpu.read_csr(0x1000),
            Err(CPUError::IllegalInstruction(_))
        ));

        // Read-only CSR writes (with non-zero mask/value)
        assert!(matches!(
            cpu.write_csr(0x301, 0x12345678), // misa is read-only
            Err(CPUError::IllegalInstruction(_))
        ));
        assert!(matches!(
            cpu.set_csr_bits(0x301, 0xFF), // non-zero mask
            Err(CPUError::IllegalInstruction(_))
        ));
        assert!(matches!(
            cpu.clear_csr_bits(0x301, 0xFF), // non-zero mask
            Err(CPUError::IllegalInstruction(_))
        ));
    }

    // Test CSR address space boundaries thoroughly
    #[test]
    fn test_csr_address_validation() {
        let mut cpu = CPU::default();

        // Valid CSR addresses are 0x000 to 0xFFF (12 bits)
        // Test boundary conditions

        // Address 0x000 - valid but doesn't exist by default
        assert!(cpu.read_csr(0x000).is_err());

        // Address 0xFFF - valid but doesn't exist by default
        assert!(cpu.read_csr(0xFFF).is_err());

        // Address 0x1000 and above - invalid (> 12 bits)
        assert!(cpu.read_csr(0x1000).is_err());
        assert!(cpu.read_csr(0xFFFF).is_err());

        // Create CSRs at boundaries
        cpu.csr_exists[0x000] = true;
        cpu.csr_exists[0xFFF] = true;

        // Now they should be accessible
        let _ = cpu.write_csr(0x000, 0x11111111).unwrap();
        let _ = cpu.write_csr(0xFFF, 0x22222222).unwrap();
        assert_eq!(cpu.read_csr(0x000).unwrap(), 0x11111111);
        assert_eq!(cpu.read_csr(0xFFF).unwrap(), 0x22222222);
    }

    // Test that operations are atomic (read old value, write new value)
    #[test]
    fn test_atomic_operations() {
        let mut cpu = CPU::default();

        // Set initial value
        cpu.write_csr(0x340, 0xAAAA5555).unwrap();

        // Atomic set bits - should return old value and update
        let old = cpu.set_csr_bits(0x340, 0x0F0F0F0F).unwrap();
        assert_eq!(old, 0xAAAA5555); // Old value returned
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0xAFAF5F5F); // New value stored

        // Atomic clear bits
        let old = cpu.clear_csr_bits(0x340, 0xF0F0F0F0).unwrap();
        assert_eq!(old, 0xAFAF5F5F); // Old value returned
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0x0F0F0F0F); // New value stored
    }

    // Test all initialized CSRs have correct properties
    #[test]
    fn test_all_standard_csrs_properties() {
        let cpu = CPU::default();

        // User-level CSRs
        assert!(cpu.csr_exists[0xC00] && cpu.csr_readonly[0xC00]); // cycle
        assert!(cpu.csr_exists[0xC01] && cpu.csr_readonly[0xC01]); // time
        assert!(cpu.csr_exists[0xC02] && cpu.csr_readonly[0xC02]); // instret

        // Machine-level CSRs
        assert!(cpu.csr_exists[0x300] && !cpu.csr_readonly[0x300]); // mstatus (r/w)
        assert!(cpu.csr_exists[0x301] && cpu.csr_readonly[0x301]); // misa (r/o)
        assert!(cpu.csr_exists[0x304] && !cpu.csr_readonly[0x304]); // mie (r/w)
        assert!(cpu.csr_exists[0x305] && !cpu.csr_readonly[0x305]); // mtvec (r/w)
        assert!(cpu.csr_exists[0x340] && !cpu.csr_readonly[0x340]); // mscratch (r/w)
        assert!(cpu.csr_exists[0x341] && !cpu.csr_readonly[0x341]); // mepc (r/w)
        assert!(cpu.csr_exists[0x342] && !cpu.csr_readonly[0x342]); // mcause (r/w)
        assert!(cpu.csr_exists[0x343] && !cpu.csr_readonly[0x343]); // mtval (r/w)
        assert!(cpu.csr_exists[0x344] && !cpu.csr_readonly[0x344]); // mip (r/w)
    }

    // Test CSR read always returns 32-bit value (zero-extended for RV32)
    #[test]
    fn test_csr_read_zero_extension() {
        let cpu = CPU::default();

        // All CSR reads should return valid u32 values
        // For RV32, CSRs are naturally 32-bit, but this documents the behavior
        assert_eq!(cpu.read_csr(0x301).unwrap(), 0x40000100); // Full 32-bit value
        assert_eq!(cpu.read_csr(0x300).unwrap(), 0x00001800); // Full 32-bit value
    }

    // Test specific MISA encoding
    #[test]
    fn test_misa_encoding() {
        let cpu = CPU::default();

        // MISA encodes the ISA
        let misa = cpu.read_csr(0x301).unwrap();

        // Bits 31-30: MXL (01 = 32-bit)
        assert_eq!((misa >> 30) & 0b11, 0b01);

        // Bit 8: I (base integer ISA)
        assert_eq!((misa >> 8) & 1, 1);

        // Our implementation: 0x40000100
        // 0100_0000_0000_0000_0000_0001_0000_0000
        // MXL=01 (32-bit), I bit set
    }

    #[test]
    fn test_apply_modify_basic() {
        let mut cpu = CPU::default();

        // Create a modify command to set register x1 to 42
        let modify = Modify {
            register_changes: vec![(Register::X1, 42)],
            memory_changes: vec![],
            csr_changes: vec![],
            pc_change: None,
        };

        // Apply the modification
        let delta = cpu.apply(&modify).unwrap();

        // Verify the register was changed
        assert_eq!(cpu.get_register(Register::X1), 42);

        // Verify the delta recorded the change
        assert_eq!(delta.register_changes.len(), 1);
        assert_eq!(delta.register_changes[0], (Register::X1, 0, 42));
    }

    #[test]
    fn test_execute_with_delta() {
        let mut cpu = CPU::default();

        // Create an ADDI instruction: ADDI x1, x0, 42
        let mut inst = super::formats::IType::new();
        inst.rd = Register::X1;
        inst.rs1 = Register::X0;
        inst.imm.set_signed(42).unwrap();
        let instruction = super::Instruction::ADDI(inst);

        // Execute with delta tracking
        let delta = cpu.execute(instruction).unwrap();

        // Verify the register was changed
        assert_eq!(cpu.get_register(Register::X1), 42);

        // Verify the delta recorded the change
        assert_eq!(delta.register_changes.len(), 1);
        assert_eq!(delta.register_changes[0], (Register::X1, 0, 42));

        // Verify PC was incremented
        assert_eq!(delta.pc_change, (0, 4));
    }

    #[test]
    fn test_delta_to_reverse_modify() {
        let mut cpu = CPU::default();

        // Execute an instruction
        let mut inst = super::formats::IType::new();
        inst.rd = Register::X1;
        inst.rs1 = Register::X0;
        inst.imm.set_signed(42).unwrap();
        let instruction = super::Instruction::ADDI(inst);

        let delta = cpu.execute(instruction).unwrap();

        // Now reverse the delta
        let reverse_modify = delta.to_reverse_modify();
        let undo_delta = cpu.apply(&reverse_modify).unwrap();

        // Verify we're back to the original state
        assert_eq!(cpu.get_register(Register::X1), 0);
        assert_eq!(cpu.pc, 0);

        // Verify the undo delta
        assert_eq!(undo_delta.register_changes.len(), 1);
        assert_eq!(undo_delta.register_changes[0], (Register::X1, 42, 0));
        assert_eq!(undo_delta.pc_change, (4, 0));
    }

    #[test]
    fn test_documentation_example() {
        let mut cpu = CPU::default();

        // Create an ADDI instruction like in our documentation
        let mut inst = super::formats::IType::new();
        inst.rd = Register::X1;
        inst.rs1 = Register::X0;
        inst.imm.set_signed(42).unwrap();
        let instruction = super::Instruction::ADDI(inst);

        // This should work like the documentation example
        let delta = cpu.execute(instruction).unwrap();

        // See what changed
        assert_eq!(delta.register_changes.len(), 1);
        assert_eq!(delta.register_changes[0], (Register::X1, 0, 42));

        // Perfect undo
        let undo = delta.to_reverse_modify();
        let _undo_delta = cpu.apply(&undo).unwrap();

        // Verify we're back to original state
        assert_eq!(cpu.get_register(Register::X1), 0);
        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn test_cpu_reset() {
        let mut cpu = CPU::default();

        // Modify CPU state
        cpu.set_register(Register::X1, 0xDEADBEEF);
        cpu.set_register(Register::X15, 0x12345678);
        cpu.pc = 0x1000;
        cpu.memory[0] = 0xFF;
        cpu.memory[100] = 0xAB;
        cpu.write_csr(0x340, 0xCAFEBABE).unwrap(); // mscratch

        // Verify state was changed
        assert_eq!(cpu.get_register(Register::X1), 0xDEADBEEF);
        assert_eq!(cpu.pc, 0x1000);
        assert_eq!(cpu.memory[0], 0xFF);
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0xCAFEBABE);

        // Reset CPU
        cpu.reset();

        // Verify all state is cleared
        for i in 0..32 {
            assert_eq!(
                cpu.get_register(Register::from_u32(i)),
                0,
                "Register X{i} should be 0"
            );
        }
        assert_eq!(cpu.pc, 0);
        assert_eq!(cpu.memory[0], 0);
        assert_eq!(cpu.memory[100], 0);

        // Verify CSRs are reset but standard ones still exist
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0); // mscratch cleared
        assert_eq!(cpu.read_csr(0x300).unwrap(), 0x00001800); // mstatus has default value
        assert_eq!(cpu.read_csr(0x301).unwrap(), 0x40000100); // misa has default value
        assert!(cpu.csr_exists[0x300]); // mstatus exists
        assert!(cpu.csr_exists[0x301]); // misa exists
        assert!(cpu.csr_readonly[0x301]); // misa is read-only
    }

    #[test]
    fn test_state_delta_memory_changes() {
        let mut cpu = CPU::default();

        // Store word instruction that modifies memory
        let mut inst = super::formats::SType::new();
        inst.rs1 = Register::X0; // base = 0
        inst.rs2 = Register::X1; // value register
        inst.imm.set_signed(100).unwrap(); // offset = 100

        // Set x1 to a test value
        cpu.set_register(Register::X1, 0xDEADBEEF);

        let instruction = super::Instruction::SW(inst);
        let delta = cpu.execute(instruction).unwrap();

        // Verify memory was changed and delta recorded it
        // SW instruction may record each byte separately
        assert!(!delta.memory_changes.is_empty());

        // Find the memory change that includes address 100
        let mem_change = delta
            .memory_changes
            .iter()
            .find(|mc| mc.addr == 100)
            .unwrap();
        assert_eq!(mem_change.old_data, vec![0]);
        assert_eq!(mem_change.new_data, vec![0xEF]); // first byte of little endian
    }

    #[test]
    fn test_state_delta_csr_changes() {
        let mut cpu = CPU::default();

        // Set up x2 with a non-zero value first
        cpu.set_register(Register::X2, 0x12345678);

        // CSR write instruction: CSRRW x1, mscratch, x2
        // This should write x2 (0x12345678) to mscratch and read old value (0) to x1
        let mut inst = super::formats::IType::new();
        inst.rd = Register::X1;
        inst.rs1 = Register::X2;
        inst.imm.set_signed(0x340).unwrap(); // mscratch CSR

        let instruction = super::Instruction::CSRRW(inst);
        let delta = cpu.execute(instruction).unwrap();

        // Verify CSR was changed and delta recorded it
        assert_eq!(delta.csr_changes.len(), 1);
        assert_eq!(delta.csr_changes[0], (0x340, 0, 0x12345678)); // old=0, new=0x12345678

        // Register x1 should be changed only if old CSR value is different from x1's current value
        // Since old CSR value was 0 and x1 starts at 0, no register change is recorded
        // But let's verify the CPU state is correct
        assert_eq!(cpu.get_register(Register::X1), 0); // Should have old CSR value
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0x12345678); // Should have new value
    }

    #[test]
    fn test_complex_modify_multiple_changes() {
        let mut cpu = CPU::default();

        // Create a complex modification
        let modify = Modify {
            register_changes: vec![
                (Register::X1, 0x1111),
                (Register::X2, 0x2222),
                (Register::X3, 0x3333),
            ],
            memory_changes: vec![
                (0x100, vec![0xAA, 0xBB, 0xCC, 0xDD]),
                (0x200, vec![0x11, 0x22]),
            ],
            csr_changes: vec![
                (0x340, 0x12345678), // mscratch
                (0x341, 0x87654321), // mepc
            ],
            pc_change: Some(0x1000),
        };

        // Apply the modification
        let delta = cpu.apply(&modify).unwrap();

        // Verify all changes were applied
        assert_eq!(cpu.get_register(Register::X1), 0x1111);
        assert_eq!(cpu.get_register(Register::X2), 0x2222);
        assert_eq!(cpu.get_register(Register::X3), 0x3333);
        assert_eq!(cpu.pc, 0x1000);
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0x12345678);
        assert_eq!(cpu.read_csr(0x341).unwrap(), 0x87654321);
        assert_eq!(&cpu.memory[0x100..0x104], &[0xAA, 0xBB, 0xCC, 0xDD]);
        assert_eq!(&cpu.memory[0x200..0x202], &[0x11, 0x22]);

        // Verify delta recorded all changes
        assert_eq!(delta.register_changes.len(), 3);
        assert_eq!(delta.memory_changes.len(), 2);
        assert_eq!(delta.csr_changes.len(), 2);
        assert_eq!(delta.pc_change, (0, 0x1000));
    }

    #[test]
    fn test_forward_modify_conversion() {
        let mut cpu = CPU::default();

        // Execute an instruction to get a delta
        let mut inst = super::formats::IType::new();
        inst.rd = Register::X1;
        inst.rs1 = Register::X0;
        inst.imm.set_signed(42).unwrap();
        let instruction = super::Instruction::ADDI(inst);

        let delta = cpu.execute(instruction).unwrap();

        // Reset CPU state
        cpu.reset();

        // Use to_forward_modify to reapply the change
        let forward_modify = delta.to_forward_modify();
        let redo_delta = cpu.apply(&forward_modify).unwrap();

        // Verify state matches original execution
        assert_eq!(cpu.get_register(Register::X1), 42);
        assert_eq!(cpu.pc, 4);

        // Verify redo delta
        assert_eq!(redo_delta.register_changes.len(), 1);
        assert_eq!(redo_delta.register_changes[0], (Register::X1, 0, 42));
        assert_eq!(redo_delta.pc_change, (0, 4));
    }

    #[test]
    fn test_modify_error_conditions() {
        let mut cpu = CPU::default();

        // Test invalid CSR address
        let modify = Modify {
            register_changes: vec![],
            memory_changes: vec![],
            csr_changes: vec![(0x999, 0x12345678)], // Invalid CSR
            pc_change: None,
        };

        let result = cpu.apply(&modify);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("CSR address"));

        // Test memory out of bounds
        let modify = Modify {
            register_changes: vec![],
            memory_changes: vec![(cpu.memory.len() as u32, vec![0x42])], // Out of bounds
            csr_changes: vec![],
            pc_change: None,
        };

        let result = cpu.apply(&modify);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Memory modification out of bounds"));
    }

    #[test]
    fn test_atomic_operations_with_modify() {
        let mut cpu = CPU::default();

        // Create a modify that should partially fail
        let modify = Modify {
            register_changes: vec![(Register::X1, 0x1111)],
            memory_changes: vec![(0x100, vec![0xAA, 0xBB])],
            csr_changes: vec![(0x999, 0x12345678)], // Invalid CSR - should fail
            pc_change: Some(0x1000),
        };

        // Apply should fail atomically
        let result = cpu.apply(&modify);
        assert!(result.is_err());

        // Verify NO changes were applied (atomic failure)
        assert_eq!(cpu.get_register(Register::X1), 0); // unchanged
        assert_eq!(cpu.pc, 0); // unchanged
        assert_eq!(cpu.memory[0x100], 0); // unchanged
    }

    #[test]
    fn test_state_delta_roundtrip() {
        let mut cpu = CPU::default();

        // Execute a complex instruction (store word)
        cpu.set_register(Register::X1, 0x1000); // base address
        cpu.set_register(Register::X2, 0xDEADBEEF); // value to store

        let mut inst = super::formats::SType::new();
        inst.rs1 = Register::X1;
        inst.rs2 = Register::X2;
        inst.imm.set_signed(4).unwrap();
        let instruction = super::Instruction::SW(inst);

        let original_state = (
            cpu.get_register(Register::X1),
            cpu.get_register(Register::X2),
            cpu.pc,
        );
        let delta = cpu.execute(instruction).unwrap();

        // Verify the instruction executed
        assert_eq!(cpu.pc, 4);
        assert_eq!(&cpu.memory[0x1004..0x1008], &[0xEF, 0xBE, 0xAD, 0xDE]);

        // Use reverse modify to undo
        let reverse_modify = delta.to_reverse_modify();
        let undo_delta = cpu.apply(&reverse_modify).unwrap();

        // Verify we're back to original state
        assert_eq!(cpu.get_register(Register::X1), original_state.0);
        assert_eq!(cpu.get_register(Register::X2), original_state.1);
        assert_eq!(cpu.pc, original_state.2);
        assert_eq!(&cpu.memory[0x1004..0x1008], &[0, 0, 0, 0]);

        // Use forward modify to redo
        let forward_modify = delta.to_forward_modify();
        let redo_delta = cpu.apply(&forward_modify).unwrap();

        // Verify we're back to executed state
        assert_eq!(cpu.pc, 4);
        assert_eq!(&cpu.memory[0x1004..0x1008], &[0xEF, 0xBE, 0xAD, 0xDE]);

        // Verify delta symmetry
        assert_eq!(
            undo_delta.register_changes.len(),
            delta.register_changes.len()
        );
        assert_eq!(undo_delta.memory_changes.len(), delta.memory_changes.len());
        assert_eq!(
            redo_delta.register_changes.len(),
            delta.register_changes.len()
        );
        assert_eq!(redo_delta.memory_changes.len(), delta.memory_changes.len());
    }
}
