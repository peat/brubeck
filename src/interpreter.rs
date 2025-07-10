//! The interpreter takes input, parses it, and executes it in the [CPU](crate::rv32_i::CPU)
//!
//! The interpreter supports standard RISC-V assembly syntax for the RV32I instruction
//! set, including common pseudo-instructions. It can parse register names (both 
//! x0-x31 and ABI names), immediate values in decimal/hex/binary formats, and 
//! execute instructions or inspect register state.
//!
//! ## Examples
//!
//! ```
//! use brubeck::interpreter::Interpreter;
//!
//! let mut i = Interpreter::new();
//!
//! // Execute an ADDI instruction that sets register x1 to 3
//! let output = i.interpret("ADDI x1, zero, 3");
//! assert!(output.is_ok());
//!
//! // Inspect register x1 to see its value
//! let output = i.interpret("x1");
//! assert!(output.unwrap().contains("3"));
//!
//! // The PC register shows the current program counter
//! let output = i.interpret("PC");
//! assert!(output.is_ok());
//! ```

use std::fmt::Display;

use crate::rv32_i::{BType, IType, Instruction, JType, PseudoInstruction, RType, Register, SType, UType, ABI, CPU};

#[cfg(feature = "repl")]
use crate::history::{HistoryManager, StateSnapshot};

pub struct Interpreter {
    cpu: CPU,
    #[cfg(feature = "repl")]
    history: HistoryManager,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    /// Creates a new Interpreter with 1 mebibyte of memory.
    pub fn new() -> Self {
        Self {
            cpu: CPU::default(), // initializes with 1 mebibyte of memory
            #[cfg(feature = "repl")]
            history: HistoryManager::new(1000), // Default history size
        }
    }

    /// Interprets a single command, which could be an instruction (eg: `ADDI x1, zero, 3`) or an
    /// inspection for registers (eg: `PC` or `X1`). Returns a String representation of the 
    /// result or an Error.
    pub fn interpret(&mut self, input: &str) -> Result<String, Error> {
        let command = parse(input)?;
        self.run_command(command)
    }

    /// Executes an [Instruction] directly, skipping the parsing steps.
    pub fn execute(&mut self, instruction: Instruction) -> Result<String, Error> {
        self.execute_with_tracking(instruction, None)
    }
    
    /// Internal method that executes with state tracking
    fn execute_with_tracking(
        &mut self, 
        instruction: Instruction,
        display_name: Option<String>
    ) -> Result<String, Error> {
        // Capture state before execution (only if REPL feature is enabled)
        #[cfg(feature = "repl")]
        let (old_registers, old_pc, instruction_text) = {
            self.cpu.clear_tracking();
            let regs = self.cpu.get_all_registers();
            let pc = self.cpu.pc;
            // Use provided display name or generate one
            let text = display_name.unwrap_or_else(|| {
                // Use the mnemonic for the instruction
                instruction.mnemonic().to_string()
            });
            (regs, pc, text)
        };
        
        // Execute the instruction
        match self.cpu.execute(instruction) {
            Ok(()) => {
                // Capture state after successful execution
                #[cfg(feature = "repl")]
                {
                    let new_registers = self.cpu.get_all_registers();
                    let new_pc = self.cpu.pc;
                    let snapshot = StateSnapshot {
                        instruction: instruction_text,
                        registers: old_registers,
                        pc: old_pc,
                        registers_after: new_registers,
                        pc_after: new_pc,
                        csr_changes: self.cpu.csr_changes.clone(),
                        memory_changes: self.cpu.memory_changes.clone(),
                    };
                    self.history.push(snapshot);
                }
                
                Ok(self.humanize_instruction(instruction))
            }
            e => Err(Error::Generic(format!("{e:?}"))),
        }
    }

    /// Converts an instruction into a human-readable description
    fn humanize_instruction(&self, instruction: Instruction) -> String {
        match instruction {
            Instruction::ADD(i) => {
                let rs1_val = self.cpu.get_register(i.rs1);
                let rs2_val = self.cpu.get_register(i.rs2);
                let rd_val = self.cpu.get_register(i.rd);
                format!("{}: Added {:?} ({}) and {:?} ({}) and stored result in {:?} ({})", 
                    instruction.mnemonic(), i.rs1, rs1_val, i.rs2, rs2_val, i.rd, rd_val)
            },
            Instruction::ADDI(i) => {
                let rs1_val = self.cpu.get_register(i.rs1);
                let rd_val = self.cpu.get_register(i.rd);
                format!("{}: Added {} to {:?} ({}) and stored result in {:?} ({})", 
                    instruction.mnemonic(), i.imm.as_i32(), i.rs1, rs1_val, i.rd, rd_val)
            },
            Instruction::SUB(i) => {
                let rs1_val = self.cpu.get_register(i.rs1);
                let rs2_val = self.cpu.get_register(i.rs2);
                let rd_val = self.cpu.get_register(i.rd);
                format!("{}: Subtracted {:?} ({}) from {:?} ({}) and stored result in {:?} ({})", 
                    instruction.mnemonic(), i.rs2, rs2_val, i.rs1, rs1_val, i.rd, rd_val)
            },
            Instruction::LW(i) => {
                let rs1_val = self.cpu.get_register(i.rs1);
                let addr = rs1_val.wrapping_add(i.imm.as_u32());
                let rd_val = self.cpu.get_register(i.rd);
                format!("{}: Loaded word from memory address 0x{:x} ({}+{}) into {:?} ({})", 
                    instruction.mnemonic(), addr, rs1_val, i.imm.as_i32(), i.rd, rd_val)
            },
            Instruction::SW(i) => {
                let rs1_val = self.cpu.get_register(i.rs1);
                let rs2_val = self.cpu.get_register(i.rs2);
                let addr = rs1_val.wrapping_add(i.imm.as_u32());
                format!("{}: Stored word from {:?} ({}) to memory address 0x{:x} ({}+{})", 
                    instruction.mnemonic(), i.rs2, rs2_val, addr, rs1_val, i.imm.as_i32())
            },
            Instruction::BEQ(i) => {
                let rs1_val = self.cpu.get_register(i.rs1);
                let rs2_val = self.cpu.get_register(i.rs2);
                if rs1_val == rs2_val {
                    format!("{}: Branch taken: {:?} ({}) equals {:?} ({})", 
                        instruction.mnemonic(), i.rs1, rs1_val, i.rs2, rs2_val)
                } else {
                    format!("{}: Branch not taken: {:?} ({}) does not equal {:?} ({})", 
                        instruction.mnemonic(), i.rs1, rs1_val, i.rs2, rs2_val)
                }
            },
            Instruction::BNE(i) => {
                let rs1_val = self.cpu.get_register(i.rs1);
                let rs2_val = self.cpu.get_register(i.rs2);
                if rs1_val != rs2_val {
                    format!("{}: Branch taken: {:?} ({}) does not equal {:?} ({})", 
                        instruction.mnemonic(), i.rs1, rs1_val, i.rs2, rs2_val)
                } else {
                    format!("{}: Branch not taken: {:?} ({}) equals {:?} ({})", 
                        instruction.mnemonic(), i.rs1, rs1_val, i.rs2, rs2_val)
                }
            },
            Instruction::JAL(i) => {
                let return_addr = self.cpu.get_register(i.rd);
                format!("{}: Jumped to PC+{} and stored return address in {:?} ({})", 
                    instruction.mnemonic(), i.imm.as_i32() * 2, i.rd, return_addr)
            },
            Instruction::JALR(i) => {
                let rs1_val = self.cpu.get_register(i.rs1);
                let return_addr = self.cpu.get_register(i.rd);
                format!("{}: Jumped to {:?} ({}) + {} and stored return address in {:?} ({})", 
                    instruction.mnemonic(), i.rs1, rs1_val, i.imm.as_i32(), i.rd, return_addr)
            },
            Instruction::LUI(i) => {
                let rd_val = self.cpu.get_register(i.rd);
                format!("{}: Loaded upper immediate {} into {:?} ({})", 
                    instruction.mnemonic(), i.imm.as_i32(), i.rd, rd_val)
            },
            Instruction::AUIPC(i) => {
                let rd_val = self.cpu.get_register(i.rd);
                format!("{}: Added upper immediate {} to PC and stored in {:?} ({})", 
                    instruction.mnemonic(), i.imm.as_i32(), i.rd, rd_val)
            },
            Instruction::AND(i) => {
                let rs1_val = self.cpu.get_register(i.rs1);
                let rs2_val = self.cpu.get_register(i.rs2);
                let rd_val = self.cpu.get_register(i.rd);
                format!("{}: Bitwise AND of {:?} ({}) and {:?} ({}) stored in {:?} ({})", 
                    instruction.mnemonic(), i.rs1, rs1_val, i.rs2, rs2_val, i.rd, rd_val)
            },
            Instruction::OR(i) => {
                let rs1_val = self.cpu.get_register(i.rs1);
                let rs2_val = self.cpu.get_register(i.rs2);
                let rd_val = self.cpu.get_register(i.rd);
                format!("{}: Bitwise OR of {:?} ({}) and {:?} ({}) stored in {:?} ({})", 
                    instruction.mnemonic(), i.rs1, rs1_val, i.rs2, rs2_val, i.rd, rd_val)
            },
            Instruction::XOR(i) => {
                let rs1_val = self.cpu.get_register(i.rs1);
                let rs2_val = self.cpu.get_register(i.rs2);
                let rd_val = self.cpu.get_register(i.rd);
                format!("{}: Bitwise XOR of {:?} ({}) and {:?} ({}) stored in {:?} ({})", 
                    instruction.mnemonic(), i.rs1, rs1_val, i.rs2, rs2_val, i.rd, rd_val)
            },
            Instruction::NOP => format!("{}: No operation", instruction.mnemonic()),
            _ => {
                format!("{}: Executed instruction", instruction.mnemonic())
            }
        }
    }

    /// Executes a [Command], which can be an instruction or an inspection
    pub fn run_command(&mut self, input: Command) -> Result<String, Error> {
        match input {
            Command::Exec(instruction) => self.execute(instruction),
            Command::ExecPseudo(pseudo) => self.execute_pseudo(pseudo),
            Command::Inspect(r) => Ok(format!(
                "{:?}: {:?} (0x{:x})",
                r,
                self.cpu.get_register(r),
                self.cpu.get_register(r)
            )),
            Command::ShowRegs => Ok(self.show_registers()),
            Command::ShowSpecificRegs(regs) => Ok(self.show_specific_registers(regs)),
            Command::ShowHelp => Ok(self.show_help()),
            #[cfg(feature = "repl")]
            Command::Undo => self.handle_undo(),
            #[cfg(feature = "repl")]
            Command::Redo => self.handle_redo(),
        }
    }

    /// Shows all register values in a formatted table
    fn show_registers(&self) -> String {
        let mut output = String::new();
        
        for i in 0..32 {
            let reg = match i {
                0 => Register::X0, 1 => Register::X1, 2 => Register::X2, 3 => Register::X3,
                4 => Register::X4, 5 => Register::X5, 6 => Register::X6, 7 => Register::X7,
                8 => Register::X8, 9 => Register::X9, 10 => Register::X10, 11 => Register::X11,
                12 => Register::X12, 13 => Register::X13, 14 => Register::X14, 15 => Register::X15,
                16 => Register::X16, 17 => Register::X17, 18 => Register::X18, 19 => Register::X19,
                20 => Register::X20, 21 => Register::X21, 22 => Register::X22, 23 => Register::X23,
                24 => Register::X24, 25 => Register::X25, 26 => Register::X26, 27 => Register::X27,
                28 => Register::X28, 29 => Register::X29, 30 => Register::X30, 31 => Register::X31,
                _ => unreachable!(),
            };
            
            let val = self.cpu.get_register(reg);
            let abi_name = match reg {
                Register::X0 => "zero", Register::X1 => "ra", Register::X2 => "sp", Register::X3 => "gp",
                Register::X4 => "tp", Register::X5 => "t0", Register::X6 => "t1", Register::X7 => "t2",
                Register::X8 => "s0", Register::X9 => "s1", Register::X10 => "a0", Register::X11 => "a1",
                Register::X12 => "a2", Register::X13 => "a3", Register::X14 => "a4", Register::X15 => "a5",
                Register::X16 => "a6", Register::X17 => "a7", Register::X18 => "s2", Register::X19 => "s3",
                Register::X20 => "s4", Register::X21 => "s5", Register::X22 => "s6", Register::X23 => "s7",
                Register::X24 => "s8", Register::X25 => "s9", Register::X26 => "s10", Register::X27 => "s11",
                Register::X28 => "t3", Register::X29 => "t4", Register::X30 => "t5", Register::X31 => "t6",
                Register::PC => "pc",
            };
            
            if i % 2 == 0 && i < 31 {
                output.push_str(&format!("x{i:2} ({abi_name:4}): 0x{val:08x}    "));
            } else {
                output.push_str(&format!("x{i:2} ({abi_name:4}): 0x{val:08x}\n"));
            }
        }
        
        output
    }

    /// Shows specific register values
    fn show_specific_registers(&self, regs: Vec<Register>) -> String {
        let mut output = String::new();
        
        for (i, reg) in regs.iter().enumerate() {
            let val = self.cpu.get_register(*reg);
            let abi_name = match reg {
                Register::X0 => "zero", Register::X1 => "ra", Register::X2 => "sp", Register::X3 => "gp",
                Register::X4 => "tp", Register::X5 => "t0", Register::X6 => "t1", Register::X7 => "t2",
                Register::X8 => "s0", Register::X9 => "s1", Register::X10 => "a0", Register::X11 => "a1",
                Register::X12 => "a2", Register::X13 => "a3", Register::X14 => "a4", Register::X15 => "a5",
                Register::X16 => "a6", Register::X17 => "a7", Register::X18 => "s2", Register::X19 => "s3",
                Register::X20 => "s4", Register::X21 => "s5", Register::X22 => "s6", Register::X23 => "s7",
                Register::X24 => "s8", Register::X25 => "s9", Register::X26 => "s10", Register::X27 => "s11",
                Register::X28 => "t3", Register::X29 => "t4", Register::X30 => "t5", Register::X31 => "t6",
                Register::PC => "pc",
            };
            
            let reg_num = match reg {
                Register::X0 => 0, Register::X1 => 1, Register::X2 => 2, Register::X3 => 3,
                Register::X4 => 4, Register::X5 => 5, Register::X6 => 6, Register::X7 => 7,
                Register::X8 => 8, Register::X9 => 9, Register::X10 => 10, Register::X11 => 11,
                Register::X12 => 12, Register::X13 => 13, Register::X14 => 14, Register::X15 => 15,
                Register::X16 => 16, Register::X17 => 17, Register::X18 => 18, Register::X19 => 19,
                Register::X20 => 20, Register::X21 => 21, Register::X22 => 22, Register::X23 => 23,
                Register::X24 => 24, Register::X25 => 25, Register::X26 => 26, Register::X27 => 27,
                Register::X28 => 28, Register::X29 => 29, Register::X30 => 30, Register::X31 => 31,
                Register::PC => 32,
            };
            
            if reg_num == 32 {
                output.push_str(&format!("PC ({abi_name:4}): 0x{val:08x}"));
            } else {
                output.push_str(&format!("x{reg_num:2} ({abi_name:4}): 0x{val:08x}"));
            }
            
            if i < regs.len() - 1 {
                output.push_str("    ");
                if (i + 1) % 2 == 0 {
                    output.push('\n');
                }
            }
        }
        
        if !output.ends_with('\n') {
            output.push('\n');
        }
        
        output
    }

    /// Shows help information
    fn show_help(&self) -> String {
        r#"Brubeck: A RISC-V REPL Help

Commands:
  /regs, /r         Show all registers
  /r x1 x2 sp       Show specific registers
  /help, /h         Show this help
  
Instructions:
  ADDI x1, x0, 42   Add immediate: x1 = x0 + 42
  ADD x1, x2, x3    Add: x1 = x2 + x3
  LW x1, 4(x2)      Load word: x1 = memory[x2 + 4]
  SW x1, 4(x2)      Store word: memory[x2 + 4] = x1
  BEQ x1, x2, 8     Branch if equal: if x1 == x2, jump PC + 8
  JAL x1, 16        Jump and link: x1 = PC + 4, jump PC + 16
  
Register inspection:
  x1, x2, sp, ra    Show individual register value
  PC                Show program counter
  
Pseudo-instructions:
  MV x1, x2         Move: x1 = x2
  LI x1, 100        Load immediate: x1 = 100
  NOP               No operation
  
Number formats:
  42                Decimal
  0x2a              Hexadecimal  
  0b101010          Binary"#.to_string()
    }

    /// Executes a pseudo-instruction by expanding it and running the real instructions
    pub fn execute_pseudo(
        &mut self,
        pseudo: PseudoInstruction,
    ) -> Result<String, Error> {
        // Get a nice display name for the pseudo-instruction
        let pseudo_name = format!("{pseudo:?}"); // We'll improve this later
        
        let instructions = pseudo
            .expand()
            .map_err(|e| Error::Generic(format!("Failed to expand pseudo-instruction: {e}")))?;

        let mut results = Vec::new();
        for inst in instructions {
            // Execute with the pseudo-instruction name for history
            match self.execute_with_tracking(inst, Some(pseudo_name.clone())) {
                Ok(result) => results.push(result),
                Err(e) => return Err(e),
            }
        }

        if results.len() == 1 {
            Ok(format!("Pseudo-instruction: {}", results[0]))
        } else {
            Ok(format!(
                "Pseudo-instruction expanded to: {}",
                results.join("; ")
            ))
        }
    }

    /// Gets the current program counter value
    pub fn get_pc(&self) -> u32 {
        self.cpu.pc
    }
    
    /// Handles the /undo command
    #[cfg(feature = "repl")]
    fn handle_undo(&mut self) -> Result<String, Error> {
        match self.history.undo() {
            Some(snapshot) => {
                // Restore CPU state
                self.cpu.set_all_registers(&snapshot.registers);
                self.cpu.pc = snapshot.pc;
                
                // Restore memory changes
                self.cpu.restore_memory(&snapshot.memory_changes);
                
                // Restore CSR changes
                self.cpu.restore_csrs(&snapshot.csr_changes);
                
                Ok(format!("Undid: {}", snapshot.instruction))
            }
            None => Err(Error::Generic("Nothing to undo".to_string())),
        }
    }
    
    /// Handles the /redo command
    #[cfg(feature = "repl")]
    fn handle_redo(&mut self) -> Result<String, Error> {
        match self.history.redo() {
            Some(snapshot) => {
                // Restore to the state AFTER the instruction
                self.cpu.set_all_registers(&snapshot.registers_after);
                self.cpu.pc = snapshot.pc_after;
                
                // Apply the memory changes
                for delta in &snapshot.memory_changes {
                    if (delta.address as usize) < self.cpu.memory.len() {
                        self.cpu.memory[delta.address as usize] = delta.new_value;
                    }
                }
                
                // Apply the CSR changes
                for &(addr, _old_val, new_val) in &snapshot.csr_changes {
                    if self.cpu.csr_exists[addr as usize] && !self.cpu.csr_readonly[addr as usize] {
                        self.cpu.csrs[addr as usize] = new_val;
                    }
                }
                
                Ok(format!("Redid: {}", snapshot.instruction))
            }
            None => Err(Error::Generic("Nothing to redo".to_string())),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Command {
    Inspect(Register),
    Exec(Instruction),
    ExecPseudo(PseudoInstruction),
    ShowRegs,
    ShowSpecificRegs(Vec<Register>),
    ShowHelp,
    #[cfg(feature = "repl")]
    Undo,
    #[cfg(feature = "repl")]
    Redo,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Register(Register),
    Instruction(Instruction),
    PseudoInstruction(PseudoInstruction),
    Value32(i32),
    OffsetRegister { offset: i32, register: Register },
}

#[derive(Debug)]
pub enum Error {
    Generic(String),
    UnrecognizedToken(String),
    UnknownInstruction {
        instruction: String,
        suggestion: Option<String>,
    },
    InvalidRegister {
        register: String,
        help: String,
    },
    WrongArgumentCount {
        instruction: String,
        expected: String,
        found: usize,
    },
    ImmediateOutOfRange {
        instruction: String,
        value: i32,
        range: String,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err_string = match self {
            Self::Generic(s) => s.to_owned(),
            
            Self::UnrecognizedToken(s) => {
                format!("Unrecognized token: '{s}'\n💡 Tip: Check for typos in instruction names, register names, or number formats")
            },
            
            Self::UnknownInstruction { instruction, suggestion } => {
                match suggestion {
                    Some(s) => format!("Unknown instruction '{instruction}'. Did you mean '{s}'?\n💡 Tip: RISC-V instructions are case-insensitive. Use 'help' for a list of supported instructions"),
                    None => format!("Unknown instruction '{instruction}'\n💡 Tip: Check the RISC-V ISA manual or use 'help' for supported instructions"),
                }
            },
            
            Self::InvalidRegister { register, help } => {
                format!("Invalid register '{register}'. {help}\n💡 Tip: Valid registers are x0-x31, or ABI names like zero, ra, sp, gp, tp, t0-t6, s0-s11, a0-a7")
            },
            
            Self::WrongArgumentCount { instruction, expected, found } => {
                let tip = match instruction.as_str() {
                    "ADD" | "SUB" | "AND" | "OR" | "XOR" | "SLL" | "SLT" | "SLTU" | "SRA" | "SRL" => 
                        "💡 Tip: R-type instructions need 3 registers: rd, rs1, rs2 (e.g., ADD x1, x2, x3)",
                    "ADDI" | "ANDI" | "ORI" | "XORI" | "SLTI" | "SLTIU" | "SLLI" | "SRAI" | "SRLI" => 
                        "💡 Tip: I-type instructions need 2 registers + immediate: rd, rs1, imm (e.g., ADDI x1, x2, 100)",
                    "LW" | "LH" | "LB" | "LHU" | "LBU" => 
                        "💡 Tip: Load instructions: LW x1, offset(base) or LW x1, base, offset",
                    "SW" | "SH" | "SB" => 
                        "💡 Tip: Store instructions: SW rs2, offset(base) or SW rs2, base, offset",
                    "BEQ" | "BNE" | "BLT" | "BGE" | "BLTU" | "BGEU" => 
                        "💡 Tip: Branch instructions need 2 registers + offset: rs1, rs2, offset",
                    "LUI" | "AUIPC" => 
                        "💡 Tip: Upper immediate instructions need register + immediate: rd, imm",
                    "JAL" => 
                        "💡 Tip: JAL needs link register + offset: rd, offset",
                    "JALR" => 
                        "💡 Tip: JALR needs link register, base register + offset: rd, rs1, offset",
                    _ => "💡 Tip: Check the RISC-V ISA manual for the correct instruction format",
                };
                format!("{instruction} expects {expected}, but {found} {} provided\n{tip}",
                    if *found == 1 { "was" } else { "were" })
            },
            
            Self::ImmediateOutOfRange { instruction, value, range } => {
                let tip = match instruction.as_str() {
                    "ADDI" | "ANDI" | "ORI" | "XORI" | "SLTI" | "SLTIU" => 
                        "💡 Tip: I-type immediates are 12-bit signed values. For larger values, use LUI + ADDI pattern",
                    "LUI" | "AUIPC" => 
                        "💡 Tip: Upper immediate instructions use 20-bit values that become the upper 20 bits of the result",
                    "SLLI" | "SRAI" | "SRLI" => 
                        "💡 Tip: Shift amounts must be 0-31 since RISC-V registers are 32 bits",
                    "BEQ" | "BNE" | "BLT" | "BGE" | "BLTU" | "BGEU" => 
                        "💡 Tip: Branch offsets are 12-bit signed values and must be even (word-aligned)",
                    "JAL" => 
                        "💡 Tip: JAL offsets are 20-bit signed values and must be even (word-aligned)",
                    _ => "💡 Tip: Different instruction types have different immediate ranges - check the RISC-V ISA manual",
                };
                format!("Immediate value {value} out of range for {instruction} (valid range: {range})\n{tip}")
            },
        };

        write!(f, "{err_string}")
    }
}

/// Parses a single line of RISC-V assembly into an executable command.
///
/// # The Four-Phase Parsing Process
/// 
/// This function implements a traditional compiler front-end with four distinct phases:
/// 
/// 1. **Normalize**: Clean up whitespace, convert to uppercase, handle punctuation
/// 2. **Tokenize**: Split input into meaningful tokens (instructions, registers, values)
/// 3. **Build Command**: Convert tokens into a structured command with validation
/// 4. **Return Result**: Provide helpful error messages if parsing fails
///
/// # Supported Input Types
/// 
/// - **Instructions**: `ADDI x1, zero, 100`, `LW x1, 4(x2)`, `JAL x1, 8`
/// - **Pseudo-instructions**: `MV x1, x2`, `LI x1, 0x1234`, `RET`
/// - **Register inspection**: `x1`, `sp`, `PC`
/// - **Multiple formats**: Hex (0x100), binary (0b1010), decimal (42)
///
/// # Examples
/// ```
/// use brubeck::interpreter::Interpreter;
/// 
/// let mut interpreter = Interpreter::new();
/// let result = interpreter.interpret("ADDI x1, zero, 100");     // Immediate instruction
/// let result = interpreter.interpret("LW x1, 4(x2)");           // Load with offset notation  
/// let result = interpreter.interpret("MV x1, x2");              // Pseudo-instruction
/// let result = interpreter.interpret("x1");                     // Register inspection
/// ```
///
/// # Educational Notes
/// 
/// This parser follows RISC-V assembly conventions:
/// - All immediates are sign-extended (even ANDI/ORI/XORI)
/// - Supports both standard `LW x1, offset(base)` and legacy `LW x1, base, offset`
/// - Validates instruction arguments and provides helpful error messages
/// - Prevents common mistakes like using PC register inappropriately
fn parse(input: &str) -> Result<Command, Error> {
    // Phase 1: Normalize input (clean whitespace, convert case, handle punctuation)
    let normalized = normalize(input);
    
    // Handle empty input - a common user mistake
    if normalized.is_empty() {
        return Err(Error::Generic("No input provided".to_owned()));
    }

    // Handle commands that start with '/'
    if let Some(first_word) = normalized.first() {
        if first_word.starts_with('/') {
            return match first_word.as_str() {
                "/REGS" | "/R" => {
                    if normalized.len() == 1 {
                        // No arguments, show all registers
                        Ok(Command::ShowRegs)
                    } else {
                        // Parse register arguments
                        let mut regs = Vec::new();
                        for arg in &normalized[1..] {
                            match parse_register(arg) {
                                Some(reg) => regs.push(reg),
                                None => return Err(Error::Generic(format!("Invalid register: {arg}"))),
                            }
                        }
                        Ok(Command::ShowSpecificRegs(regs))
                    }
                },
                "/HELP" | "/H" => Ok(Command::ShowHelp),
                #[cfg(feature = "repl")]
                "/UNDO" | "/U" => Ok(Command::Undo),
                #[cfg(feature = "repl")]
                "/REDO" => Ok(Command::Redo),
                _ => Err(Error::Generic(format!("Unknown command: {first_word}"))),
            };
        }
    }

    // Phase 2: Convert normalized string into meaningful tokens
    let mut tokens = tokenize(normalized)?;

    // Phase 3: Build a structured command from tokens with validation
    create_command_from_tokens(&mut tokens)
}

/// Creates a structured command from parsed tokens.
///
/// # Command Types
/// 
/// This function determines what type of command the user wants to execute based on 
/// the first token in the input:
/// 
/// - **Register Inspection**: `x1`, `sp`, `PC` → `Command::Inspect(register)`
/// - **Hardware Instructions**: `ADDI`, `LW`, `JAL` → `Command::Exec(instruction)`
/// - **Pseudo-instructions**: `MV`, `LI`, `RET` → `Command::ExecPseudo(pseudo)`
/// - **Invalid**: Raw numbers like `42` → Error (numbers need context)
///
/// # Token Processing
/// 
/// The function uses a "consume-first" pattern where it removes the first token
/// to determine the command type, then passes the remaining tokens to specialized
/// builders for validation and construction.
/// 
/// # Educational Notes
/// 
/// This demonstrates a common compiler pattern: dispatching based on the first
/// token to specialized handlers. Each handler knows how to validate and build
/// its specific command type.
fn create_command_from_tokens(tokens: &mut Vec<Token>) -> Result<Command, Error> {
    if tokens.is_empty() {
        return Err(Error::Generic("Empty tokens in build!".to_owned()));
    }

    // Remove and examine the first token to determine command type
    let first_token = tokens.remove(0);

    match first_token {
        // Single register = inspect that register's value
        Token::Register(register) => Ok(Command::Inspect(register)),
        
        // Raw number without context = error (user probably meant something else)
        Token::Value32(value) => Err(Error::Generic(format!("Value: {value}"))),
        
        // Hardware instruction = build and validate the full instruction
        Token::Instruction(mut i) => Ok(Command::Exec(build_instruction(&mut i, tokens)?)),
        
        // Pseudo-instruction = expand to real instruction(s)
        Token::PseudoInstruction(mut p) => Ok(Command::ExecPseudo(build_pseudo_instruction(
            &mut p, tokens,
        )?)),
        Token::OffsetRegister { offset, register } => Err(Error::Generic(format!(
            "Unexpected offset(register) syntax: {offset}({register:?})"
        ))),
    }
}

/// Builds a validated hardware instruction from tokens.
///
/// # RISC-V Instruction Format Dispatch
/// 
/// This function demonstrates how RISC-V instructions are organized by format:
/// - **R-type**: Register-register operations (ADD, SUB, AND, OR, etc.)
/// - **I-type**: Immediate operations (ADDI, ANDI, loads, JALR, etc.)
/// - **S-type**: Store operations (SB, SH, SW)
/// - **B-type**: Branch operations (BEQ, BNE, BLT, etc.)
/// - **U-type**: Upper immediate operations (LUI, AUIPC)
/// - **J-type**: Jump operations (JAL)
///
/// # Educational Notes
/// 
/// Each instruction type has its own builder function that knows how to validate
/// and construct that specific format. This demonstrates the "dispatch to specialist"
/// pattern common in compilers.
fn build_instruction(instruction: &mut Instruction, args: &[Token]) -> Result<Instruction, Error> {
    let output = match instruction {
        // R-type instructions: register-register operations
        Instruction::ADD(mut rtype) => Instruction::ADD(build_rtype(&mut rtype, args)?),
        Instruction::AND(mut rtype) => Instruction::AND(build_rtype(&mut rtype, args)?),
        Instruction::OR(mut rtype) => Instruction::OR(build_rtype(&mut rtype, args)?),
        Instruction::SLL(mut rtype) => Instruction::SLL(build_rtype(&mut rtype, args)?),
        Instruction::SLT(mut rtype) => Instruction::SLT(build_rtype(&mut rtype, args)?),
        Instruction::SLTU(mut rtype) => Instruction::SLTU(build_rtype(&mut rtype, args)?),
        Instruction::SRA(mut rtype) => Instruction::SRA(build_rtype(&mut rtype, args)?),
        Instruction::SRL(mut rtype) => Instruction::SRL(build_rtype(&mut rtype, args)?),
        Instruction::SUB(mut rtype) => Instruction::SUB(build_rtype(&mut rtype, args)?),
        Instruction::XOR(mut rtype) => Instruction::XOR(build_rtype(&mut rtype, args)?),
        
        // I-type instructions: immediate operations
        Instruction::ADDI(mut itype) => Instruction::ADDI(build_itype(&mut itype, args, "ADDI")?),
        Instruction::ANDI(mut itype) => Instruction::ANDI(build_itype(&mut itype, args, "ANDI")?),
        Instruction::ORI(mut itype) => Instruction::ORI(build_itype(&mut itype, args, "ORI")?),
        Instruction::SLTI(mut itype) => Instruction::SLTI(build_itype(&mut itype, args, "SLTI")?),
        Instruction::SLTIU(mut itype) => Instruction::SLTIU(build_itype(&mut itype, args, "SLTIU")?),
        Instruction::XORI(mut itype) => Instruction::XORI(build_itype(&mut itype, args, "XORI")?),
        Instruction::JALR(mut itype) => Instruction::JALR(build_itype(&mut itype, args, "JALR")?),
        
        // I-type shifts: special validation for 5-bit shift amounts
        Instruction::SLLI(mut itype) => Instruction::SLLI(build_shift_itype(&mut itype, args, "SLLI")?),
        Instruction::SRAI(mut itype) => Instruction::SRAI(build_shift_itype(&mut itype, args, "SRAI")?),
        Instruction::SRLI(mut itype) => Instruction::SRLI(build_shift_itype(&mut itype, args, "SRLI")?),
        
        // I-type loads: support both standard and legacy syntax
        Instruction::LB(mut itype) => Instruction::LB(build_load_itype(&mut itype, args)?),
        Instruction::LBU(mut itype) => Instruction::LBU(build_load_itype(&mut itype, args)?),
        Instruction::LH(mut itype) => Instruction::LH(build_load_itype(&mut itype, args)?),
        Instruction::LHU(mut itype) => Instruction::LHU(build_load_itype(&mut itype, args)?),
        Instruction::LW(mut itype) => Instruction::LW(build_load_itype(&mut itype, args)?),
        
        // S-type instructions: store operations
        Instruction::SB(mut stype) => Instruction::SB(build_store_stype(&mut stype, args)?),
        Instruction::SH(mut stype) => Instruction::SH(build_store_stype(&mut stype, args)?),
        Instruction::SW(mut stype) => Instruction::SW(build_store_stype(&mut stype, args)?),
        
        // B-type instructions: branch operations
        Instruction::BEQ(mut btype) => Instruction::BEQ(build_btype(&mut btype, args, "BEQ")?),
        Instruction::BGE(mut btype) => Instruction::BGE(build_btype(&mut btype, args, "BGE")?),
        Instruction::BGEU(mut btype) => Instruction::BGEU(build_btype(&mut btype, args, "BGEU")?),
        Instruction::BLT(mut btype) => Instruction::BLT(build_btype(&mut btype, args, "BLT")?),
        Instruction::BLTU(mut btype) => Instruction::BLTU(build_btype(&mut btype, args, "BLTU")?),
        Instruction::BNE(mut btype) => Instruction::BNE(build_btype(&mut btype, args, "BNE")?),
        
        // U-type instructions: upper immediate operations  
        Instruction::AUIPC(mut utype) => Instruction::AUIPC(build_utype(&mut utype, args, "AUIPC")?),
        Instruction::LUI(mut utype) => Instruction::LUI(build_utype(&mut utype, args, "LUI")?),
        
        // J-type instructions: jump operations
        Instruction::JAL(mut jtype) => Instruction::JAL(build_jtype(&mut jtype, args, "JAL")?),
        
        // System instructions: no arguments required
        Instruction::EBREAK(mut itype) => Instruction::EBREAK(build_system_itype(&mut itype, args, "EBREAK")?),
        Instruction::ECALL(mut itype) => Instruction::ECALL(build_system_itype(&mut itype, args, "ECALL")?),
        Instruction::FENCE(mut itype) => Instruction::FENCE(build_system_itype(&mut itype, args, "FENCE")?),
        
        // Special case: NOP has no arguments or variants
        Instruction::NOP => Instruction::NOP,
        
        // CSR Instructions: Control and Status Register operations
        // These provide access to processor state and control registers
        Instruction::CSRRW(mut itype) => Instruction::CSRRW(build_csr_itype(&mut itype, args, "CSRRW")?),
        Instruction::CSRRS(mut itype) => Instruction::CSRRS(build_csr_itype(&mut itype, args, "CSRRS")?),
        Instruction::CSRRC(mut itype) => Instruction::CSRRC(build_csr_itype(&mut itype, args, "CSRRC")?),
        Instruction::CSRRWI(mut itype) => Instruction::CSRRWI(build_csr_itype_imm(&mut itype, args, "CSRRWI")?),
        Instruction::CSRRSI(mut itype) => Instruction::CSRRSI(build_csr_itype_imm(&mut itype, args, "CSRRSI")?),
        Instruction::CSRRCI(mut itype) => Instruction::CSRRCI(build_csr_itype_imm(&mut itype, args, "CSRRCI")?),
    };

    Ok(output)
}

fn build_pseudo_instruction(
    pseudo: &mut PseudoInstruction,
    args: &[Token],
) -> Result<PseudoInstruction, Error> {

    let output = match pseudo {
        PseudoInstruction::MV { rd: _, rs: _ } => {
            if let [Token::Register(dest), Token::Register(src)] = args {
                PseudoInstruction::MV {
                    rd: *dest,
                    rs: *src,
                }
            } else {
                return Err(Error::Generic(format!("Invalid MV arguments: {args:?}")));
            }
        }
        PseudoInstruction::NOT { rd: _, rs: _ } => {
            if let [Token::Register(dest), Token::Register(src)] = args {
                PseudoInstruction::NOT {
                    rd: *dest,
                    rs: *src,
                }
            } else {
                return Err(Error::Generic(format!("Invalid NOT arguments: {args:?}")));
            }
        }
        PseudoInstruction::SEQZ { rd: _, rs: _ } => {
            if let [Token::Register(dest), Token::Register(src)] = args {
                PseudoInstruction::SEQZ {
                    rd: *dest,
                    rs: *src,
                }
            } else {
                return Err(Error::Generic(format!("Invalid SEQZ arguments: {args:?}")));
            }
        }
        PseudoInstruction::SNEZ { rd: _, rs: _ } => {
            if let [Token::Register(dest), Token::Register(src)] = args {
                PseudoInstruction::SNEZ {
                    rd: *dest,
                    rs: *src,
                }
            } else {
                return Err(Error::Generic(format!("Invalid SNEZ arguments: {args:?}")));
            }
        }
        PseudoInstruction::J { offset: _ } => {
            if let [Token::Value32(val)] = args {
                PseudoInstruction::J {
                    offset: *val,
                }
            } else {
                return Err(Error::Generic(format!("Invalid J arguments: {args:?}")));
            }
        }
        PseudoInstruction::JR { rs: _ } => {
            if let [Token::Register(src)] = args {
                PseudoInstruction::JR { rs: *src }
            } else {
                return Err(Error::Generic(format!("Invalid JR arguments: {args:?}")));
            }
        }
        PseudoInstruction::RET => {
            if args.is_empty() {
                PseudoInstruction::RET
            } else {
                return Err(Error::Generic(format!("RET takes no arguments, got: {args:?}")));
            }
        }
        PseudoInstruction::LI { rd: _, imm: _ } => {
            if let [Token::Register(dest), Token::Value32(val)] = args {
                PseudoInstruction::LI {
                    rd: *dest,
                    imm: *val,
                }
            } else {
                return Err(Error::Generic(format!("Invalid LI arguments: {args:?}")));
            }
        }
        PseudoInstruction::LA { .. } => {
            return Err(Error::Generic(
                "LA pseudo-instruction not yet implemented".to_string(),
            ));
        }
    };

    Ok(output)
}

fn build_utype(utype: &mut UType, args: &[Token], instruction_name: &str) -> Result<UType, Error> {
    if let [Token::Register(rd), Token::Value32(imm)] = args {
        // PC cannot be used as destination in U-type instructions
        // AUIPC reads PC implicitly but doesn't allow PC as destination
        validate_not_pc(*rd, "destination")?;
        
        // Validate immediate is in 20-bit signed range
        if *imm < -524288 || *imm > 524287 {
            return Err(Error::ImmediateOutOfRange {
                instruction: instruction_name.to_string(),
                value: *imm,
                range: "-524288 to 524287".to_string(),
            });
        }
        
        utype.rd = *rd;
        utype
            .imm
            .set_signed(*imm)
            .map_err(|e| Error::Generic(format!("{e:?}")))?;
        Ok(*utype)
    } else {
        Err(Error::WrongArgumentCount {
            instruction: instruction_name.to_string(),
            expected: "2 arguments (rd, immediate)".to_string(),
            found: args.len(),
        })
    }
}

fn build_jtype(jtype: &mut JType, args: &[Token], instruction_name: &str) -> Result<JType, Error> {
    if let [Token::Register(rd), Token::Value32(imm)] = args {
        // PC cannot be used as destination register
        validate_not_pc(*rd, "destination")?;
        
        // Validate immediate is in 20-bit signed range (actually 21-bit with bit 0 always 0)
        if *imm < -1048576 || *imm > 1048574 {
            return Err(Error::ImmediateOutOfRange {
                instruction: instruction_name.to_string(),
                value: *imm,
                range: "-1048576 to 1048574 (even values only)".to_string(),
            });
        }
        
        // Check alignment - must be even
        if *imm % 2 != 0 {
            return Err(Error::Generic(format!(
                "{instruction_name}: Jump offset {imm} must be even (2-byte aligned)"
            )));
        }
        
        jtype.rd = *rd;
        jtype
            .imm
            .set_signed(*imm)
            .map_err(|e| Error::Generic(format!("{e:?}")))?;
        Ok(*jtype)
    } else {
        Err(Error::WrongArgumentCount {
            instruction: instruction_name.to_string(),
            expected: "2 arguments (rd, offset)".to_string(),
            found: args.len(),
        })
    }
}

fn build_btype(btype: &mut BType, args: &[Token], instruction_name: &str) -> Result<BType, Error> {
    if let [Token::Register(rs1), Token::Register(rs2), Token::Value32(imm)] = args {
        // PC cannot be used as source in branch comparisons
        validate_not_pc(*rs1, "source 1")?;
        validate_not_pc(*rs2, "source 2")?;
        
        // Validate immediate is in 12-bit signed range (actually 13-bit with bit 0 always 0)
        if *imm < -4096 || *imm > 4094 {
            return Err(Error::ImmediateOutOfRange {
                instruction: instruction_name.to_string(),
                value: *imm,
                range: "-4096 to 4094 (even values only)".to_string(),
            });
        }
        
        // Check alignment - must be even
        if *imm % 2 != 0 {
            return Err(Error::Generic(format!(
                "{instruction_name}: Branch offset {imm} must be even (2-byte aligned)"
            )));
        }
        
        btype.rs1 = *rs1;
        btype.rs2 = *rs2;
        btype
            .imm
            .set_signed(*imm)
            .map_err(|e| Error::Generic(format!("{e:?}")))?;
        Ok(*btype)
    } else {
        Err(Error::WrongArgumentCount {
            instruction: instruction_name.to_string(),
            expected: "3 arguments (rs1, rs2, offset)".to_string(),
            found: args.len(),
        })
    }
}


fn build_itype(itype: &mut IType, args: &[Token], instruction_name: &str) -> Result<IType, Error> {
    if let [Token::Register(rd), Token::Register(rs1), Token::Value32(imm)] = args {
        // PC validation - most I-type instructions cannot use PC
        // Exception: JALR can have PC as implicit destination (updates PC)
        if instruction_name != "JALR" {
            validate_not_pc(*rd, "destination")?;
        }
        validate_not_pc(*rs1, "source")?;
        
        // Validate immediate is in 12-bit signed range
        if *imm < -2048 || *imm > 2047 {
            return Err(Error::ImmediateOutOfRange {
                instruction: instruction_name.to_string(),
                value: *imm,
                range: "-2048 to 2047".to_string(),
            });
        }
        
        itype.rd = *rd;
        itype.rs1 = *rs1;
        itype
            .imm
            .set_signed(*imm)
            .map_err(|e| Error::Generic(format!("{e:?}")))?;
        Ok(*itype)
    } else {
        Err(Error::WrongArgumentCount {
            instruction: instruction_name.to_string(),
            expected: "3 arguments (rd, rs1, immediate)".to_string(),
            found: args.len(),
        })
    }
}

/// Validates that the correct number of arguments are provided for an instruction.
///
/// # Educational Notes
/// 
/// This is a common compiler validation pattern - checking that the user provided
/// the right number of arguments before trying to process them. This prevents
/// confusing errors later in the parsing process.
fn validate_argument_count(instruction: &str, expected: usize, found: usize) -> Result<(), Error> {
    if found != expected {
        Err(Error::WrongArgumentCount {
            instruction: instruction.to_string(),
            expected: format!("{expected} arguments"),
            found,
        })
    } else {
        Ok(())
    }
}

/// Validates that an immediate value is within the specified range.
///
/// # Educational Notes
/// 
/// Different RISC-V instruction types have different immediate field sizes:
/// - I-type: 12 bits signed (-2048 to 2047)
/// - U-type: 20 bits signed (-524288 to 524287)
/// - J-type: 20 bits signed, even values only
/// - Shifts: 5 bits unsigned (0 to 31)
/// 
/// This function provides a centralized place to validate these ranges.
fn validate_immediate_range(instruction: &str, value: i32, min: i32, max: i32) -> Result<(), Error> {
    if value < min || value > max {
        Err(Error::ImmediateOutOfRange {
            instruction: instruction.to_string(),
            value,
            range: format!("{min} to {max}"),
        })
    } else {
        Ok(())
    }
}

/// Validates that a register is not the PC register.
///
/// # PC Register Rules
/// 
/// In RISC-V, the PC (Program Counter) register has special handling:
/// - It cannot be used as a general-purpose register operand
/// - It's only accessible via AUIPC (implicitly) or as a jump target
/// - Using PC incorrectly can lead to undefined behavior
///
/// # Educational Notes
/// 
/// This demonstrates architecture-specific validation - some registers have
/// special meanings and cannot be used in normal operations.
fn validate_not_pc(reg: Register, position: &str) -> Result<(), Error> {
    if reg == Register::PC {
        Err(Error::Generic(format!(
            "PC register cannot be used as {position} in this instruction. PC is only accessible via AUIPC or as an implicit operand in jumps."
        )))
    } else {
        Ok(())
    }
}

/// Build CSR instruction with register operand (CSRRW, CSRRS, CSRRC)
/// Syntax: CSRRW rd, csr, rs1
fn build_csr_itype(itype: &mut IType, args: &[Token], instruction_name: &str) -> Result<IType, Error> {
    if let [Token::Register(rd), Token::Value32(csr_addr), Token::Register(rs1)] = args {
        // Validate CSR address is in valid range (12-bit unsigned)
        if *csr_addr < 0 || *csr_addr > 4095 {
            return Err(Error::ImmediateOutOfRange {
                instruction: instruction_name.to_string(),
                value: *csr_addr,
                range: "0 to 4095 (12-bit unsigned CSR address)".to_string(),
            });
        }
        
        // PC register validation
        validate_not_pc(*rd, "destination")?;
        validate_not_pc(*rs1, "source")?;
        
        itype.rd = *rd;
        itype.rs1 = *rs1;
        itype.imm.set_unsigned(*csr_addr as u32)
            .map_err(|e| Error::Generic(format!("CSR address error: {e:?}")))?;
        
        Ok(*itype)
    } else {
        Err(Error::WrongArgumentCount {
            instruction: instruction_name.to_string(),
            expected: "3 arguments (rd, csr, rs1)".to_string(),
            found: args.len(),
        })
    }
}

/// Build CSR immediate instruction (CSRRWI, CSRRSI, CSRRCI)
/// Syntax: CSRRWI rd, csr, uimm5
fn build_csr_itype_imm(itype: &mut IType, args: &[Token], instruction_name: &str) -> Result<IType, Error> {
    if let [Token::Register(rd), Token::Value32(csr_addr), Token::Value32(uimm)] = args {
        // Validate CSR address is in valid range (12-bit unsigned)
        if *csr_addr < 0 || *csr_addr > 4095 {
            return Err(Error::ImmediateOutOfRange {
                instruction: instruction_name.to_string(),
                value: *csr_addr,
                range: "0 to 4095 (12-bit unsigned CSR address)".to_string(),
            });
        }
        
        // Validate immediate is 5-bit unsigned
        if *uimm < 0 || *uimm > 31 {
            return Err(Error::ImmediateOutOfRange {
                instruction: instruction_name.to_string(),
                value: *uimm,
                range: "0 to 31 (5-bit unsigned immediate)".to_string(),
            });
        }
        
        // PC register validation
        validate_not_pc(*rd, "destination")?;
        
        itype.rd = *rd;
        // For CSR immediate instructions, the immediate value goes in the rs1 field
        itype.rs1 = Register::from_u32(*uimm as u32);
        itype.imm.set_unsigned(*csr_addr as u32)
            .map_err(|e| Error::Generic(format!("CSR address error: {e:?}")))?;
        
        Ok(*itype)
    } else {
        Err(Error::WrongArgumentCount {
            instruction: instruction_name.to_string(),
            expected: "3 arguments (rd, csr, uimm5)".to_string(),
            found: args.len(),
        })
    }
}

fn build_rtype(rtype: &mut RType, args: &[Token]) -> Result<RType, Error> {
    if let [Token::Register(rd), Token::Register(rs1), Token::Register(rs2)] = args {
        // PC cannot be used in R-type instructions
        validate_not_pc(*rd, "destination")?;
        validate_not_pc(*rs1, "source 1")?;
        validate_not_pc(*rs2, "source 2")?;
        
        rtype.rd = *rd;
        rtype.rs1 = *rs1;
        rtype.rs2 = *rs2;
        Ok(*rtype)
    } else {
        Err(Error::Generic(format!("Invalid RType arguments: {args:?}")))
    }
}

/// Builds a shift instruction (SLLI, SRLI, SRAI) with validation.
///
/// # Shift Instruction Rules
/// 
/// RISC-V shift instructions have special validation requirements:
/// - Shift amount must be 0-31 (5 bits unsigned)
/// - PC register cannot be used as source or destination
/// - Shift amount is encoded in the lower 5 bits of the immediate field
///
/// # Educational Notes
/// 
/// This demonstrates instruction-specific validation - shift instructions have
/// tighter constraints than general I-type instructions because the hardware
/// shift unit only supports 5-bit shift amounts.
fn build_shift_itype(itype: &mut IType, args: &[Token], instruction_name: &str) -> Result<IType, Error> {
    // Validate we have exactly 3 arguments
    validate_argument_count(instruction_name, 3, args.len())?;
    
    if let [Token::Register(rd), Token::Register(rs1), Token::Value32(imm)] = args {
        // PC cannot be used in shift instructions
        validate_not_pc(*rd, "destination")?;
        validate_not_pc(*rs1, "source")?;
        
        // RISC-V shift instructions only support 5-bit shift amounts (0-31)
        validate_immediate_range(instruction_name, *imm, 0, 31)?;
        
        itype.rd = *rd;
        itype.rs1 = *rs1;
        itype
            .imm
            .set_signed(*imm)
            .map_err(|e| Error::Generic(format!("{e:?}")))?;
        Ok(*itype)
    } else {
        Err(Error::WrongArgumentCount {
            instruction: instruction_name.to_string(),
            expected: "3 arguments (rd, rs1, shift_amount)".to_string(),
            found: args.len(),
        })
    }
}

fn build_system_itype(itype: &mut IType, args: &[Token], instruction_name: &str) -> Result<IType, Error> {
    // System instructions (FENCE, ECALL, EBREAK) take no arguments
    if !args.is_empty() {
        return Err(Error::WrongArgumentCount {
            instruction: instruction_name.to_string(),
            expected: "no arguments".to_string(),
            found: args.len(),
        });
    }
    Ok(*itype)
}

fn build_load_itype(itype: &mut IType, args: &[Token]) -> Result<IType, Error> {
    match args {
        // Standard RISC-V syntax: LW rd, offset(rs1)
        [Token::Register(rd), Token::OffsetRegister { offset, register }] => {
            validate_not_pc(*rd, "destination")?;
            validate_not_pc(*register, "base address")?;
            
            itype.rd = *rd;
            itype.rs1 = *register;
            itype
                .imm
                .set_signed(*offset)
                .map_err(|e| Error::Generic(format!("{e:?}")))?;
            Ok(*itype)
        }
        // Legacy syntax: LW rd, rs1, offset
        [Token::Register(rd), Token::Register(rs1), Token::Value32(imm)] => {
            validate_not_pc(*rd, "destination")?;
            validate_not_pc(*rs1, "base address")?;
            
            itype.rd = *rd;
            itype.rs1 = *rs1;
            itype
                .imm
                .set_signed(*imm)
                .map_err(|e| Error::Generic(format!("{e:?}")))?;
            Ok(*itype)
        }
        _ => Err(Error::Generic(format!("Invalid load arguments: {args:?}"))),
    }
}

fn build_store_stype(stype: &mut SType, args: &[Token]) -> Result<SType, Error> {
    match args {
        // Standard RISC-V syntax: SW rs2, offset(rs1)
        [Token::Register(rs2), Token::OffsetRegister { offset, register }] => {
            validate_not_pc(*register, "base address")?;
            validate_not_pc(*rs2, "source")?;
            
            stype.rs1 = *register;
            stype.rs2 = *rs2;
            stype
                .imm
                .set_signed(*offset)
                .map_err(|e| Error::Generic(format!("{e:?}")))?;
            Ok(*stype)
        }
        // Legacy syntax: SW rs1, rs2, offset (note: this seems backwards!)
        [Token::Register(rs1), Token::Register(rs2), Token::Value32(imm)] => {
            validate_not_pc(*rs1, "base address")?;
            validate_not_pc(*rs2, "source")?;
            
            stype.rs1 = *rs1;
            stype.rs2 = *rs2;
            stype
                .imm
                .set_signed(*imm)
                .map_err(|e| Error::Generic(format!("{e:?}")))?;
            Ok(*stype)
        }
        _ => Err(Error::Generic(format!("Invalid store arguments: {args:?}"))),
    }
}

/// Converts normalized strings into typed tokens.
///
/// # Token Types
/// 
/// Each string is classified into one of these token types:
/// - **Instruction**: Hardware RISC-V instructions (ADD, ADDI, LW, etc.)
/// - **PseudoInstruction**: Assembly conveniences (MV, LI, RET, etc.)
/// - **Register**: Register names (x1, sp, zero, etc.)
/// - **Value32**: Numeric values (42, 0x100, 0b1010, etc.)
/// - **OffsetRegister**: Load/store notation (4(x2), -8(sp), etc.)
///
/// # Error Handling
/// 
/// If any string cannot be tokenized, the entire process fails with a descriptive
/// error message. This "fail-fast" approach prevents partially-parsed commands
/// from executing incorrectly.
///
/// # Educational Notes
/// 
/// This function demonstrates the `collect()` method with `Result` types - if any
/// `parse_single_token()` call returns an error, the entire `collect()` fails. This is
/// a common Rust pattern for "all or nothing" operations.
fn tokenize(input: Vec<String>) -> Result<Vec<Token>, Error> {
    input.into_iter().map(parse_single_token).collect()
}

fn suggest_instruction(unknown: &str) -> Option<String> {
    let instructions = [
        "ADD", "ADDI", "AND", "ANDI", "AUIPC", "BEQ", "BGE", "BGEU", "BLT", "BLTU", "BNE",
        "EBREAK", "ECALL", "FENCE", "JAL", "JALR", "LB", "LBU", "LH", "LHU", "LUI", "LW",
        "NOP", "OR", "ORI", "SB", "SH", "SLL", "SLLI", "SLT", "SLTI", "SLTIU", "SLTU",
        "SRA", "SRAI", "SRL", "SRLI", "SUB", "SW", "XOR", "XORI",
        // CSR Instructions
        "CSRRW", "CSRRS", "CSRRC", "CSRRWI", "CSRRSI", "CSRRCI",
        // Pseudo-instructions
        "MV", "NOT", "NEG", "SEQZ", "SNEZ", "J", "JR", "RET", "LI"
    ];
    
    // Find the most similar instruction (simple case-insensitive check for now)
    let unknown_upper = unknown.to_uppercase();
    
    // First check for exact match (case-insensitive)
    if instructions.contains(&unknown_upper.as_str()) {
        return Some(unknown_upper);
    }
    
    // Check if it starts with any known instruction
    for inst in instructions {
        if unknown_upper.starts_with(inst) || inst.starts_with(&unknown_upper) {
            return Some(inst.to_string());
        }
    }
    
    None
}

/// Parses a single normalized string into a typed token.
///
/// # Token Recognition Process
/// 
/// This function examines a string and determines what type of token it represents:
/// 1. **Offset(register) patterns**: `4(x2)`, `-8(sp)` → `OffsetRegister`
/// 2. **Hardware instructions**: `ADD`, `ADDI`, `LW` → `Instruction`
/// 3. **Pseudo-instructions**: `MV`, `LI`, `RET` → `PseudoInstruction`
/// 4. **Registers**: `x1`, `sp`, `zero` → `Register`
/// 5. **Numeric values**: `42`, `0x100`, `0b1010` → `Value32`
/// 6. **CSR names**: `MSTATUS`, `CYCLE` → `Value32` (with CSR address)
///
/// # Educational Notes
/// 
/// This function demonstrates pattern matching on string prefixes and suffixes
/// to classify tokens. It's a common technique in lexical analysis.
fn parse_single_token(input: String) -> Result<Token, Error> {
    let token = match input.as_str() {
        // registers
        "PC" => Token::Register(Register::PC),
        "X0" => Token::Register(Register::X0),
        "X1" => Token::Register(Register::X1),
        "X2" => Token::Register(Register::X2),
        "X3" => Token::Register(Register::X3),
        "X4" => Token::Register(Register::X4),
        "X5" => Token::Register(Register::X5),
        "X6" => Token::Register(Register::X6),
        "X7" => Token::Register(Register::X7),
        "X8" => Token::Register(Register::X8),
        "X9" => Token::Register(Register::X9),
        "X10" => Token::Register(Register::X10),
        "X11" => Token::Register(Register::X11),
        "X12" => Token::Register(Register::X12),
        "X13" => Token::Register(Register::X13),
        "X14" => Token::Register(Register::X14),
        "X15" => Token::Register(Register::X15),
        "X16" => Token::Register(Register::X16),
        "X17" => Token::Register(Register::X17),
        "X18" => Token::Register(Register::X18),
        "X19" => Token::Register(Register::X19),
        "X20" => Token::Register(Register::X20),
        "X21" => Token::Register(Register::X21),
        "X22" => Token::Register(Register::X22),
        "X23" => Token::Register(Register::X23),
        "X24" => Token::Register(Register::X24),
        "X25" => Token::Register(Register::X25),
        "X26" => Token::Register(Register::X26),
        "X27" => Token::Register(Register::X27),
        "X28" => Token::Register(Register::X28),
        "X29" => Token::Register(Register::X29),
        "X30" => Token::Register(Register::X30),
        "X31" => Token::Register(Register::X31),

        // ABI-named registers
        "ZERO" => Token::Register(ABI::Zero.to_register()),
        "RA" => Token::Register(ABI::RA.to_register()),
        "SP" => Token::Register(ABI::SP.to_register()),
        "GP" => Token::Register(ABI::GP.to_register()),
        "TP" => Token::Register(ABI::TP.to_register()),
        "T0" => Token::Register(ABI::T0.to_register()),
        "T1" => Token::Register(ABI::T1.to_register()),
        "T2" => Token::Register(ABI::T2.to_register()),
        "S0" => Token::Register(ABI::S0.to_register()),
        "FP" => Token::Register(ABI::FP.to_register()),
        "S1" => Token::Register(ABI::S1.to_register()),
        "A0" => Token::Register(ABI::A0.to_register()),
        "A1" => Token::Register(ABI::A1.to_register()),
        "A2" => Token::Register(ABI::A2.to_register()),
        "A3" => Token::Register(ABI::A3.to_register()),
        "A4" => Token::Register(ABI::A4.to_register()),
        "A5" => Token::Register(ABI::A5.to_register()),
        "A6" => Token::Register(ABI::A6.to_register()),
        "A7" => Token::Register(ABI::A7.to_register()),
        "S2" => Token::Register(ABI::S2.to_register()),
        "S3" => Token::Register(ABI::S3.to_register()),
        "S4" => Token::Register(ABI::S4.to_register()),
        "S5" => Token::Register(ABI::S5.to_register()),
        "S6" => Token::Register(ABI::S6.to_register()),
        "S7" => Token::Register(ABI::S7.to_register()),
        "S8" => Token::Register(ABI::S8.to_register()),
        "S9" => Token::Register(ABI::S9.to_register()),
        "S10" => Token::Register(ABI::S10.to_register()),
        "S11" => Token::Register(ABI::S11.to_register()),
        "T3" => Token::Register(ABI::T3.to_register()),
        "T4" => Token::Register(ABI::T4.to_register()),
        "T5" => Token::Register(ABI::T5.to_register()),
        "T6" => Token::Register(ABI::T6.to_register()),

        // instructions
        "ADD" => Token::Instruction(Instruction::ADD(RType::default())),
        "ADDI" => Token::Instruction(Instruction::ADDI(IType::default())),
        "AND" => Token::Instruction(Instruction::AND(RType::default())),
        "ANDI" => Token::Instruction(Instruction::ANDI(IType::default())),
        "AUIPC" => Token::Instruction(Instruction::AUIPC(UType::default())),
        "BEQ" => Token::Instruction(Instruction::BEQ(BType::default())),
        "BGE" => Token::Instruction(Instruction::BGE(BType::default())),
        "BGEU" => Token::Instruction(Instruction::BGEU(BType::default())),
        "BLT" => Token::Instruction(Instruction::BLT(BType::default())),
        "BLTU" => Token::Instruction(Instruction::BLTU(BType::default())),
        "BNE" => Token::Instruction(Instruction::BNE(BType::default())),
        "EBREAK" => Token::Instruction(Instruction::EBREAK(IType::default())),
        "ECALL" => Token::Instruction(Instruction::ECALL(IType::default())),
        "FENCE" => Token::Instruction(Instruction::FENCE(IType::default())),
        "JAL" => Token::Instruction(Instruction::JAL(JType::default())),
        "JALR" => Token::Instruction(Instruction::JALR(IType::default())),
        "LB" => Token::Instruction(Instruction::LB(IType::default())),
        "LBU" => Token::Instruction(Instruction::LBU(IType::default())),
        "LH" => Token::Instruction(Instruction::LH(IType::default())),
        "LHU" => Token::Instruction(Instruction::LHU(IType::default())),
        "LUI" => Token::Instruction(Instruction::LUI(UType::default())),
        "LW" => Token::Instruction(Instruction::LW(IType::default())),
        "NOP" => Token::Instruction(Instruction::NOP),
        "OR" => Token::Instruction(Instruction::OR(RType::default())),
        "ORI" => Token::Instruction(Instruction::ORI(IType::default())),
        "SB" => Token::Instruction(Instruction::SB(SType::default())),
        "SH" => Token::Instruction(Instruction::SH(SType::default())),
        "SLL" => Token::Instruction(Instruction::SLL(RType::default())),
        "SLLI" => Token::Instruction(Instruction::SLLI(IType::default())),
        "SLT" => Token::Instruction(Instruction::SLT(RType::default())),
        "SLTI" => Token::Instruction(Instruction::SLTI(IType::default())),
        "SLTIU" => Token::Instruction(Instruction::SLTIU(IType::default())),
        "SLTU" => Token::Instruction(Instruction::SLTU(RType::default())),
        "SRA" => Token::Instruction(Instruction::SRA(RType::default())),
        "SRAI" => Token::Instruction(Instruction::SRAI(IType::default())),
        "SRL" => Token::Instruction(Instruction::SRL(RType::default())),
        "SRLI" => Token::Instruction(Instruction::SRLI(IType::default())),
        "SUB" => Token::Instruction(Instruction::SUB(RType::default())),
        "SW" => Token::Instruction(Instruction::SW(SType::default())),
        "XOR" => Token::Instruction(Instruction::XOR(RType::default())),
        "XORI" => Token::Instruction(Instruction::XORI(IType::default())),
        
        // CSR Instructions
        "CSRRW" => Token::Instruction(Instruction::CSRRW(IType::default())),
        "CSRRS" => Token::Instruction(Instruction::CSRRS(IType::default())),
        "CSRRC" => Token::Instruction(Instruction::CSRRC(IType::default())),
        "CSRRWI" => Token::Instruction(Instruction::CSRRWI(IType::default())),
        "CSRRSI" => Token::Instruction(Instruction::CSRRSI(IType::default())),
        "CSRRCI" => Token::Instruction(Instruction::CSRRCI(IType::default())),
        
        // CSR Names (Control and Status Registers)
        "CYCLE" => Token::Value32(0xC00),      // Cycle counter (read-only)
        "TIME" => Token::Value32(0xC01),       // Timer (read-only)
        "INSTRET" => Token::Value32(0xC02),    // Instructions retired (read-only)
        "MSTATUS" => Token::Value32(0x300),    // Machine status register
        "MISA" => Token::Value32(0x301),       // Machine ISA register
        "MIE" => Token::Value32(0x304),        // Machine interrupt enable
        "MTVEC" => Token::Value32(0x305),      // Machine trap vector base address
        "MSCRATCH" => Token::Value32(0x340),   // Machine scratch register
        "MEPC" => Token::Value32(0x341),       // Machine exception program counter
        "MCAUSE" => Token::Value32(0x342),     // Machine trap cause
        "MTVAL" => Token::Value32(0x343),      // Machine trap value
        "MIP" => Token::Value32(0x344),        // Machine interrupt pending

        // Pseudo-instructions - these expand to real instructions
        "MV" => Token::PseudoInstruction(PseudoInstruction::MV {
            rd: Register::X0,
            rs: Register::X0,
        }),
        "NOT" => Token::PseudoInstruction(PseudoInstruction::NOT {
            rd: Register::X0,
            rs: Register::X0,
        }),
        "SEQZ" => Token::PseudoInstruction(PseudoInstruction::SEQZ {
            rd: Register::X0,
            rs: Register::X0,
        }),
        "SNEZ" => Token::PseudoInstruction(PseudoInstruction::SNEZ {
            rd: Register::X0,
            rs: Register::X0,
        }),
        "J" => {
            Token::PseudoInstruction(PseudoInstruction::J { offset: 0 })
        }
        "JR" => Token::PseudoInstruction(PseudoInstruction::JR {
            rs: Register::X0,
        }),
        "RET" => Token::PseudoInstruction(PseudoInstruction::RET),
        "LI" => Token::PseudoInstruction(PseudoInstruction::LI {
            rd: Register::X0,
            imm: 0,
        }),

        // everything else could be a value or offset(register)
        _ => parse_value_or_offset(input)?,
    };

    Ok(token)
}

fn parse_value_or_offset(input: String) -> Result<Token, Error> {
    // Check if it's offset(register) syntax
    if let Some(paren_pos) = input.find('(') {
        if input.ends_with(')') {
            // Extract offset and register parts
            let offset_str = &input[..paren_pos];
            let register_str = &input[paren_pos + 1..input.len() - 1];
            
            // Parse the offset as a number
            let offset = parse_number(offset_str).map_err(Error::Generic)?;
            
            // Parse the register
            let register = match parse_register(register_str) {
                Some(reg) => reg,
                None => return Err(Error::InvalidRegister {
                    register: register_str.to_string(),
                    help: "Valid registers are x0-x31 or ABI names (zero, ra, sp, etc.)".to_string(),
                }),
            };
            
            return Ok(Token::OffsetRegister { offset, register });
        }
    }
    
    // Otherwise try to parse as a regular value
    match parse_number(&input) {
        Ok(v) => Ok(Token::Value32(v)),
        Err(_) => {
            // Could be an unknown instruction
            let suggestion = suggest_instruction(&input);
            Err(Error::UnknownInstruction {
                instruction: input,
                suggestion,
            })
        }
    }
}

fn parse_number(input: &str) -> Result<i32, String> {
    // Support hex (0x), binary (0b), and decimal
    let value = if input.starts_with("0X") || input.starts_with("0x") {
        // Parse hex
        i32::from_str_radix(&input[2..], 16)
    } else if input.starts_with("0B") || input.starts_with("0b") {
        // Parse binary
        i32::from_str_radix(&input[2..], 2)
    } else {
        // Parse decimal
        input.parse::<i32>()
    };
    
    value.map_err(|_| format!("Invalid number: {input}"))
}

fn parse_register(input: &str) -> Option<Register> {
    match input {
        "PC" => Some(Register::PC),
        "X0" | "ZERO" => Some(Register::X0),
        "X1" | "RA" => Some(Register::X1),
        "X2" | "SP" => Some(Register::X2),
        "X3" | "GP" => Some(Register::X3),
        "X4" | "TP" => Some(Register::X4),
        "X5" | "T0" => Some(Register::X5),
        "X6" | "T1" => Some(Register::X6),
        "X7" | "T2" => Some(Register::X7),
        "X8" | "S0" | "FP" => Some(Register::X8),
        "X9" | "S1" => Some(Register::X9),
        "X10" | "A0" => Some(Register::X10),
        "X11" | "A1" => Some(Register::X11),
        "X12" | "A2" => Some(Register::X12),
        "X13" | "A3" => Some(Register::X13),
        "X14" | "A4" => Some(Register::X14),
        "X15" | "A5" => Some(Register::X15),
        "X16" | "A6" => Some(Register::X16),
        "X17" | "A7" => Some(Register::X17),
        "X18" | "S2" => Some(Register::X18),
        "X19" | "S3" => Some(Register::X19),
        "X20" | "S4" => Some(Register::X20),
        "X21" | "S5" => Some(Register::X21),
        "X22" | "S6" => Some(Register::X22),
        "X23" | "S7" => Some(Register::X23),
        "X24" | "S8" => Some(Register::X24),
        "X25" | "S9" => Some(Register::X25),
        "X26" | "S10" => Some(Register::X26),
        "X27" | "S11" => Some(Register::X27),
        "X28" | "T3" => Some(Register::X28),
        "X29" | "T4" => Some(Register::X29),
        "X30" | "T5" => Some(Register::X30),
        "X31" | "T6" => Some(Register::X31),
        _ => None,
    }
}

/// Normalizes raw input into a clean, tokenizable format.
///
/// # Normalization Process
/// 
/// This function prepares user input for tokenization by:
/// 1. **Converting to uppercase**: RISC-V mnemonics are case-insensitive
/// 2. **Splitting on whitespace**: Handles spaces, tabs, newlines uniformly
/// 3. **Splitting on commas**: Supports both `ADD x1, x2, x3` and `ADD x1 x2 x3`
/// 4. **Removing empty tokens**: Filters out extra spaces and empty comma-separated fields
///
/// # Input Flexibility
/// 
/// The parser accepts various input formats:
/// - `"ADDI x1, zero, 100"`     → `["ADDI", "x1", "zero", "100"]`
/// - `"addi  x1    zero  100"`  → `["ADDI", "x1", "zero", "100"]`
/// - `"LW x1, 4(x2)"`          → `["LW", "x1", "4(x2)"]`
/// - `"mov x1,x2"`             → `["MOV", "x1", "x2"]`
///
/// # Educational Notes
/// 
/// This demonstrates lexical analysis - the first phase of most compilers.
/// We're converting unstructured text into a normalized sequence of strings
/// that can be more easily processed by the tokenizer.
fn normalize(input: &str) -> Vec<String> {
    let mut output = vec![];

    // Split on whitespace first, then commas - handles mixed formats gracefully
    for ws in input.to_uppercase().split_whitespace() {
        for t in ws.split(',') {
            // Skip empty tokens (from extra spaces or trailing commas)
            if t.is_empty() {
                continue;
            }
            output.push(t.to_owned());
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_input() {
        let a = "whitespace is   weird \t and can be dumb";
        let b = "commas ,are, ok\t,too";

        assert_eq!(
            normalize(a),
            vec!["WHITESPACE", "IS", "WEIRD", "AND", "CAN", "BE", "DUMB"]
        );
        assert_eq!(normalize(b), vec!["COMMAS", "ARE", "OK", "TOO"]);
    }

    #[test]
    fn tokenize_input() {
        let a = "ADD x1, x2, x3";

        let normalized = normalize(a);
        let result = tokenize(normalized);

        assert!(result.is_ok());

        let tokens = result.unwrap();

        assert_eq!(
            tokens,
            vec![
                Token::Instruction(Instruction::ADD(RType::default())),
                Token::Register(Register::X1),
                Token::Register(Register::X2),
                Token::Register(Register::X3)
            ]
        );
    }

    #[test]
    fn parse_command() {
        let a = "ADD x1, x2, x3";
        let result = parse(a);

        assert!(result.is_ok());

        let rtype = RType {
            rd: Register::X1,
            rs1: Register::X2,
            rs2: Register::X3,
            ..Default::default()
        };

        assert_eq!(result.unwrap(), Command::Exec(Instruction::ADD(rtype)));
    }

    #[test]
    fn trivial_add() {
        let mut i = Interpreter::default();
        i.cpu.x2 = 3;
        i.cpu.x3 = 5;

        assert_eq!(i.cpu.x1, 0);

        let input = "ADD x1, x2, x3";
        assert!(i.interpret(input).is_ok());

        assert_eq!(i.cpu.x1, 8);
    }
}
