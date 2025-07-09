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

#[derive(Default)]
pub struct Interpreter {
    cpu: CPU,
}

impl Interpreter {
    /// Creates a new Interpreter with 1 mebibyte of memory.
    pub fn new() -> Self {
        Self {
            cpu: CPU::default(), // initializes with 1 mebibyte of memory
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
        match self.cpu.execute(instruction) {
            Ok(()) => Ok(format!("{instruction:?}")),
            e => Err(Error::Generic(format!("{e:?}"))),
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
        }
    }

    /// Executes a pseudo-instruction by expanding it and running the real instructions
    pub fn execute_pseudo(
        &mut self,
        pseudo: PseudoInstruction,
    ) -> Result<String, Error> {
        let instructions = pseudo
            .expand()
            .map_err(|e| Error::Generic(format!("Failed to expand pseudo-instruction: {e}")))?;

        let mut results = Vec::new();
        for inst in instructions {
            match self.cpu.execute(inst) {
                Ok(()) => results.push(format!("{inst:?}")),
                Err(e) => return Err(Error::Generic(format!("{e:?}"))),
            }
        }

        Ok(format!(
            "Pseudo {:?} expanded to: {}",
            pseudo,
            results.join(", ")
        ))
    }
}

#[derive(Debug, PartialEq)]
pub enum Command {
    Inspect(Register),
    Exec(Instruction),
    ExecPseudo(PseudoInstruction),
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
            Self::UnrecognizedToken(s) => format!("Unrecognized token: '{s}'"),
            Self::UnknownInstruction { instruction, suggestion } => {
                match suggestion {
                    Some(s) => format!("Unknown instruction '{instruction}'. Did you mean '{s}'?"),
                    None => format!("Unknown instruction '{instruction}'"),
                }
            },
            Self::InvalidRegister { register, help } => {
                format!("Invalid register '{register}'. {help}")
            },
            Self::WrongArgumentCount { instruction, expected, found } => {
                format!("{instruction} expects {expected}, but {found} {} provided",
                    if *found == 1 { "was" } else { "were" })
            },
            Self::ImmediateOutOfRange { instruction, value, range } => {
                format!("Immediate value {value} out of range for {instruction} (valid range: {range})")
            },
        };

        write!(f, "{err_string}")
    }
}

fn parse(input: &str) -> Result<Command, Error> {
    // clean up whitespace, punctuation, capitalization, etc ...
    let normalized = normalize(input);
    
    // Handle empty input
    if normalized.is_empty() {
        return Err(Error::Generic("No input provided".to_owned()));
    }

    // convert the normalized input into recognized tokens
    let mut tokens = tokenize(normalized)?;

    // build a command from those tokens
    build_command(&mut tokens)
}

fn build_command(tokens: &mut Vec<Token>) -> Result<Command, Error> {
    if tokens.is_empty() {
        return Err(Error::Generic("Empty tokens in build!".to_owned()));
    }

    let first_token = tokens.remove(0);

    match first_token {
        Token::Register(register) => Ok(Command::Inspect(register)),
        Token::Value32(value) => Err(Error::Generic(format!("Value: {value}"))),
        Token::Instruction(mut i) => Ok(Command::Exec(build_instruction(&mut i, tokens)?)),
        Token::PseudoInstruction(mut p) => Ok(Command::ExecPseudo(build_pseudo_instruction(
            &mut p, tokens,
        )?)),
        Token::OffsetRegister { offset, register } => Err(Error::Generic(format!(
            "Unexpected offset(register) syntax: {}({:?})", offset, register
        ))),
    }
}

fn build_instruction(instruction: &mut Instruction, args: &[Token]) -> Result<Instruction, Error> {
    let output = match instruction {
        // build instructions
        Instruction::ADD(mut rtype) => Instruction::ADD(build_rtype(&mut rtype, args)?),
        Instruction::ADDI(mut itype) => Instruction::ADDI(build_itype(&mut itype, args, "ADDI")?),
        Instruction::AND(mut rtype) => Instruction::AND(build_rtype(&mut rtype, args)?),
        Instruction::ANDI(mut itype) => Instruction::ANDI(build_itype(&mut itype, args, "ANDI")?),
        Instruction::AUIPC(mut utype) => Instruction::AUIPC(build_utype(&mut utype, args, "AUIPC")?),
        Instruction::BEQ(mut btype) => Instruction::BEQ(build_btype(&mut btype, args, "BEQ")?),
        Instruction::BGE(mut btype) => Instruction::BGE(build_btype(&mut btype, args, "BGE")?),
        Instruction::BGEU(mut btype) => Instruction::BGEU(build_btype(&mut btype, args, "BGEU")?),
        Instruction::BLT(mut btype) => Instruction::BLT(build_btype(&mut btype, args, "BLT")?),
        Instruction::BLTU(mut btype) => Instruction::BLTU(build_btype(&mut btype, args, "BLTU")?),
        Instruction::BNE(mut btype) => Instruction::BNE(build_btype(&mut btype, args, "BNE")?),
        Instruction::EBREAK(mut itype) => Instruction::EBREAK(build_system_itype(&mut itype, args, "EBREAK")?),
        Instruction::ECALL(mut itype) => Instruction::ECALL(build_system_itype(&mut itype, args, "ECALL")?),
        Instruction::FENCE(mut itype) => Instruction::FENCE(build_system_itype(&mut itype, args, "FENCE")?),
        Instruction::JAL(mut jtype) => Instruction::JAL(build_jtype(&mut jtype, args, "JAL")?),
        Instruction::JALR(mut itype) => Instruction::JALR(build_itype(&mut itype, args, "JALR")?),
        Instruction::LB(mut itype) => Instruction::LB(build_load_itype(&mut itype, args)?),
        Instruction::LBU(mut itype) => Instruction::LBU(build_load_itype(&mut itype, args)?),
        Instruction::LH(mut itype) => Instruction::LH(build_load_itype(&mut itype, args)?),
        Instruction::LHU(mut itype) => Instruction::LHU(build_load_itype(&mut itype, args)?),
        Instruction::LUI(mut utype) => Instruction::LUI(build_utype(&mut utype, args, "LUI")?),
        Instruction::LW(mut itype) => Instruction::LW(build_load_itype(&mut itype, args)?),
        Instruction::NOP => Instruction::NOP,
        Instruction::OR(mut rtype) => Instruction::OR(build_rtype(&mut rtype, args)?),
        Instruction::ORI(mut itype) => Instruction::ORI(build_itype(&mut itype, args, "ORI")?),
        Instruction::SB(mut stype) => Instruction::SB(build_store_stype(&mut stype, args)?),
        Instruction::SH(mut stype) => Instruction::SH(build_store_stype(&mut stype, args)?),
        Instruction::SLL(mut rtype) => Instruction::SLL(build_rtype(&mut rtype, args)?),
        Instruction::SLLI(mut itype) => Instruction::SLLI(build_shift_itype(&mut itype, args, "SLLI")?),
        Instruction::SLT(mut rtype) => Instruction::SLT(build_rtype(&mut rtype, args)?),
        Instruction::SLTI(mut itype) => Instruction::SLTI(build_itype(&mut itype, args, "SLTI")?),
        Instruction::SLTIU(mut itype) => Instruction::SLTIU(build_itype(&mut itype, args, "SLTIU")?),
        Instruction::SLTU(mut rtype) => Instruction::SLTU(build_rtype(&mut rtype, args)?),
        Instruction::SRA(mut rtype) => Instruction::SRA(build_rtype(&mut rtype, args)?),
        Instruction::SRAI(mut itype) => Instruction::SRAI(build_shift_itype(&mut itype, args, "SRAI")?),
        Instruction::SRL(mut rtype) => Instruction::SRL(build_rtype(&mut rtype, args)?),
        Instruction::SRLI(mut itype) => Instruction::SRLI(build_shift_itype(&mut itype, args, "SRLI")?),
        Instruction::SUB(mut rtype) => Instruction::SUB(build_rtype(&mut rtype, args)?),
        Instruction::SW(mut stype) => Instruction::SW(build_store_stype(&mut stype, args)?),
        Instruction::XOR(mut rtype) => Instruction::XOR(build_rtype(&mut rtype, args)?),
        Instruction::XORI(mut itype) => Instruction::XORI(build_itype(&mut itype, args, "XORI")?),
        
        // CSR Instructions (TODO: implement proper parsing)
        Instruction::CSRRW(mut itype) => Instruction::CSRRW(build_itype(&mut itype, args, "CSRRW")?),
        Instruction::CSRRS(mut itype) => Instruction::CSRRS(build_itype(&mut itype, args, "CSRRS")?),
        Instruction::CSRRC(mut itype) => Instruction::CSRRC(build_itype(&mut itype, args, "CSRRC")?),
        Instruction::CSRRWI(mut itype) => Instruction::CSRRWI(build_itype(&mut itype, args, "CSRRWI")?),
        Instruction::CSRRSI(mut itype) => Instruction::CSRRSI(build_itype(&mut itype, args, "CSRRSI")?),
        Instruction::CSRRCI(mut itype) => Instruction::CSRRCI(build_itype(&mut itype, args, "CSRRCI")?),
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
                    offset: *val as i32,
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
                "{}: Jump offset {} must be even (2-byte aligned)",
                instruction_name, imm
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
                "{}: Branch offset {} must be even (2-byte aligned)",
                instruction_name, imm
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

fn validate_not_pc(reg: Register, position: &str) -> Result<(), Error> {
    if reg == Register::PC {
        Err(Error::Generic(format!(
            "PC register cannot be used as {} in this instruction. PC is only accessible via AUIPC or as an implicit operand in jumps.",
            position
        )))
    } else {
        Ok(())
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

fn build_shift_itype(itype: &mut IType, args: &[Token], instruction_name: &str) -> Result<IType, Error> {
    if let [Token::Register(rd), Token::Register(rs1), Token::Value32(imm)] = args {
        // PC cannot be used in shift instructions
        validate_not_pc(*rd, "destination")?;
        validate_not_pc(*rs1, "source")?;
        
        // Validate shift amount is in range 0-31
        if *imm < 0 || *imm > 31 {
            return Err(Error::ImmediateOutOfRange {
                instruction: instruction_name.to_string(),
                value: *imm,
                range: "0-31".to_string(),
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

fn tokenize(input: Vec<String>) -> Result<Vec<Token>, Error> {
    input.into_iter().map(tokenize_one).collect()
}

fn suggest_instruction(unknown: &str) -> Option<String> {
    let instructions = [
        "ADD", "ADDI", "AND", "ANDI", "AUIPC", "BEQ", "BGE", "BGEU", "BLT", "BLTU", "BNE",
        "EBREAK", "ECALL", "FENCE", "JAL", "JALR", "LB", "LBU", "LH", "LHU", "LUI", "LW",
        "NOP", "OR", "ORI", "SB", "SH", "SLL", "SLLI", "SLT", "SLTI", "SLTIU", "SLTU",
        "SRA", "SRAI", "SRL", "SRLI", "SUB", "SW", "XOR", "XORI",
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

fn tokenize_one(input: String) -> Result<Token, Error> {
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
    
    value.map_err(|_| format!("Invalid number: {}", input))
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

fn normalize(input: &str) -> Vec<String> {
    let mut output = vec![];

    // split on whitespace and commas, uppercase
    for ws in input.to_uppercase().split_whitespace() {
        for t in ws.split(',') {
            // ignore empty tokens
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
