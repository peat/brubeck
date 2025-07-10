//! Instruction builder module for constructing validated RISC-V instructions
//!
//! This module contains all the builder functions that take parsed tokens and
//! construct properly validated instruction objects. Each instruction format
//! has its own builder function with specific validation rules.

use super::types::{Error, Token};
use super::validator;
use crate::rv32_i::{
    BType, IType, Instruction, JType, PseudoInstruction, RType, Register, SType, UType,
};

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
pub fn build_instruction(
    instruction: &mut Instruction,
    args: &[Token],
) -> Result<Instruction, Error> {
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
        Instruction::SLTIU(mut itype) => {
            Instruction::SLTIU(build_itype(&mut itype, args, "SLTIU")?)
        }
        Instruction::XORI(mut itype) => Instruction::XORI(build_itype(&mut itype, args, "XORI")?),
        Instruction::JALR(mut itype) => Instruction::JALR(build_itype(&mut itype, args, "JALR")?),

        // I-type shifts: special validation for 5-bit shift amounts
        Instruction::SLLI(mut itype) => {
            Instruction::SLLI(build_shift_itype(&mut itype, args, "SLLI")?)
        }
        Instruction::SRAI(mut itype) => {
            Instruction::SRAI(build_shift_itype(&mut itype, args, "SRAI")?)
        }
        Instruction::SRLI(mut itype) => {
            Instruction::SRLI(build_shift_itype(&mut itype, args, "SRLI")?)
        }

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
        Instruction::AUIPC(mut utype) => {
            Instruction::AUIPC(build_utype(&mut utype, args, "AUIPC")?)
        }
        Instruction::LUI(mut utype) => Instruction::LUI(build_utype(&mut utype, args, "LUI")?),

        // J-type instructions: jump operations
        Instruction::JAL(mut jtype) => Instruction::JAL(build_jtype(&mut jtype, args, "JAL")?),

        // System instructions: no arguments required
        Instruction::EBREAK(mut itype) => {
            Instruction::EBREAK(build_system_itype(&mut itype, args, "EBREAK")?)
        }
        Instruction::ECALL(mut itype) => {
            Instruction::ECALL(build_system_itype(&mut itype, args, "ECALL")?)
        }
        Instruction::FENCE(mut itype) => {
            Instruction::FENCE(build_system_itype(&mut itype, args, "FENCE")?)
        }

        // Special case: NOP has no arguments or variants
        Instruction::NOP => Instruction::NOP,

        // CSR Instructions: Control and Status Register operations
        // These provide access to processor state and control registers
        Instruction::CSRRW(mut itype) => {
            Instruction::CSRRW(build_csr_itype(&mut itype, args, "CSRRW")?)
        }
        Instruction::CSRRS(mut itype) => {
            Instruction::CSRRS(build_csr_itype(&mut itype, args, "CSRRS")?)
        }
        Instruction::CSRRC(mut itype) => {
            Instruction::CSRRC(build_csr_itype(&mut itype, args, "CSRRC")?)
        }
        Instruction::CSRRWI(mut itype) => {
            Instruction::CSRRWI(build_csr_itype_imm(&mut itype, args, "CSRRWI")?)
        }
        Instruction::CSRRSI(mut itype) => {
            Instruction::CSRRSI(build_csr_itype_imm(&mut itype, args, "CSRRSI")?)
        }
        Instruction::CSRRCI(mut itype) => {
            Instruction::CSRRCI(build_csr_itype_imm(&mut itype, args, "CSRRCI")?)
        }
    };

    Ok(output)
}

/// Builds a validated pseudo-instruction from tokens.
///
/// # Pseudo-instructions
///
/// These are assembly conveniences that expand to one or more real instructions:
/// - **MV rd, rs**: Copy register (ADDI rd, rs, 0)
/// - **NOT rd, rs**: Bitwise NOT (XORI rd, rs, -1)
/// - **SEQZ rd, rs**: Set if equal to zero (SLTIU rd, rs, 1)
/// - **SNEZ rd, rs**: Set if not equal to zero (SLTU rd, x0, rs)
/// - **J offset**: Unconditional jump (JAL x0, offset)
/// - **JR rs**: Jump register (JALR x0, rs, 0)
/// - **RET**: Return from function (JALR x0, x1, 0)
/// - **LI rd, imm**: Load immediate (ADDI or LUI+ADDI depending on size)
pub fn build_pseudo_instruction(
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
                PseudoInstruction::J { offset: *val }
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
                return Err(Error::Generic(format!(
                    "RET takes no arguments, got: {args:?}"
                )));
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

/// Builds a U-type instruction (LUI, AUIPC).
///
/// # U-type Format
///
/// U-type instructions contain a 20-bit immediate that forms the upper 20 bits
/// of a 32-bit value. The lower 12 bits are filled with zeros.
///
/// # Validation
///
/// - Destination register cannot be PC
/// - Immediate must fit in 20 bits signed (-524288 to 524287)
fn build_utype(utype: &mut UType, args: &[Token], instruction_name: &str) -> Result<UType, Error> {
    if let [Token::Register(rd), Token::Value32(imm)] = args {
        // PC cannot be used as destination in U-type instructions
        // AUIPC reads PC implicitly but doesn't allow PC as destination
        validator::validate_not_pc(*rd, "destination")?;

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

/// Builds a J-type instruction (JAL).
///
/// # J-type Format
///
/// JAL (Jump And Link) saves the return address and jumps to PC + offset.
/// The offset is a 20-bit signed value that must be even (2-byte aligned).
///
/// # Validation
///
/// - Destination register cannot be PC (but PC is updated implicitly)
/// - Offset must be even (instruction alignment requirement)
/// - Offset must fit in 20 bits signed (-1048576 to 1048574)
fn build_jtype(jtype: &mut JType, args: &[Token], instruction_name: &str) -> Result<JType, Error> {
    if let [Token::Register(rd), Token::Value32(imm)] = args {
        // PC cannot be used as destination register
        validator::validate_not_pc(*rd, "destination")?;

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

/// Builds a B-type instruction (branches).
///
/// # B-type Format
///
/// Branch instructions compare two registers and jump if the condition is true.
/// The offset is relative to the current PC and must be even.
///
/// # Validation
///
/// - Source registers cannot be PC
/// - Offset must be even (instruction alignment)
/// - Offset must fit in 12 bits signed (-4096 to 4094)
fn build_btype(btype: &mut BType, args: &[Token], instruction_name: &str) -> Result<BType, Error> {
    if let [Token::Register(rs1), Token::Register(rs2), Token::Value32(imm)] = args {
        // PC cannot be used as source in branch comparisons
        validator::validate_not_pc(*rs1, "source 1")?;
        validator::validate_not_pc(*rs2, "source 2")?;

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

/// Builds a standard I-type instruction.
///
/// # I-type Format
///
/// I-type instructions operate on one register and an immediate value.
/// Common uses: ADDI, ANDI, ORI, XORI, SLTI, SLTIU, JALR
///
/// # Validation
///
/// - Destination register usually cannot be PC (exception: JALR updates PC)
/// - Source register cannot be PC
/// - Immediate must fit in 12 bits signed (-2048 to 2047)
fn build_itype(itype: &mut IType, args: &[Token], instruction_name: &str) -> Result<IType, Error> {
    if let [Token::Register(rd), Token::Register(rs1), Token::Value32(imm)] = args {
        // PC validation - most I-type instructions cannot use PC
        // Exception: JALR can have PC as implicit destination (updates PC)
        if instruction_name != "JALR" {
            validator::validate_not_pc(*rd, "destination")?;
        }
        validator::validate_not_pc(*rs1, "source")?;

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

/// Build CSR instruction with register operand (CSRRW, CSRRS, CSRRC).
///
/// # CSR Register Instructions
///
/// These instructions operate on Control and Status Registers:
/// - CSRRW: Read CSR and write new value
/// - CSRRS: Read CSR and set bits
/// - CSRRC: Read CSR and clear bits
///
/// # Syntax
///
/// `CSRRW rd, csr, rs1`
///
/// # Validation
///
/// - CSR address must be 12-bit unsigned (0-4095)
/// - Registers cannot be PC
fn build_csr_itype(
    itype: &mut IType,
    args: &[Token],
    instruction_name: &str,
) -> Result<IType, Error> {
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
        validator::validate_not_pc(*rd, "destination")?;
        validator::validate_not_pc(*rs1, "source")?;

        itype.rd = *rd;
        itype.rs1 = *rs1;
        itype
            .imm
            .set_unsigned(*csr_addr as u32)
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

/// Build CSR immediate instruction (CSRRWI, CSRRSI, CSRRCI).
///
/// # CSR Immediate Instructions
///
/// These instructions use a 5-bit immediate instead of a register:
/// - CSRRWI: Write immediate to CSR
/// - CSRRSI: Set bits using immediate mask
/// - CSRRCI: Clear bits using immediate mask
///
/// # Syntax
///
/// `CSRRWI rd, csr, uimm5`
///
/// # Validation
///
/// - CSR address must be 12-bit unsigned (0-4095)
/// - Immediate must be 5-bit unsigned (0-31)
/// - Destination register cannot be PC
fn build_csr_itype_imm(
    itype: &mut IType,
    args: &[Token],
    instruction_name: &str,
) -> Result<IType, Error> {
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
        validator::validate_not_pc(*rd, "destination")?;

        itype.rd = *rd;
        // For CSR immediate instructions, the immediate value goes in the rs1 field
        itype.rs1 = Register::from_u32(*uimm as u32);
        itype
            .imm
            .set_unsigned(*csr_addr as u32)
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

/// Builds an R-type instruction.
///
/// # R-type Format
///
/// R-type instructions operate on three registers with no immediate.
/// Common uses: ADD, SUB, AND, OR, XOR, SLL, SRL, SRA, SLT, SLTU
///
/// # Validation
///
/// - All registers cannot be PC
fn build_rtype(rtype: &mut RType, args: &[Token]) -> Result<RType, Error> {
    if let [Token::Register(rd), Token::Register(rs1), Token::Register(rs2)] = args {
        // PC cannot be used in R-type instructions
        validator::validate_not_pc(*rd, "destination")?;
        validator::validate_not_pc(*rs1, "source 1")?;
        validator::validate_not_pc(*rs2, "source 2")?;

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
fn build_shift_itype(
    itype: &mut IType,
    args: &[Token],
    instruction_name: &str,
) -> Result<IType, Error> {
    // Validate we have exactly 3 arguments
    validator::validate_argument_count(instruction_name, 3, args.len())?;

    if let [Token::Register(rd), Token::Register(rs1), Token::Value32(imm)] = args {
        // PC cannot be used in shift instructions
        validator::validate_not_pc(*rd, "destination")?;
        validator::validate_not_pc(*rs1, "source")?;

        // RISC-V shift instructions only support 5-bit shift amounts (0-31)
        validator::validate_immediate_range(instruction_name, *imm, 0, 31)?;

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

/// Builds a system instruction (FENCE, ECALL, EBREAK).
///
/// # System Instructions
///
/// These instructions provide system-level functionality:
/// - FENCE: Memory ordering barrier
/// - ECALL: System call/environment call
/// - EBREAK: Breakpoint for debugging
///
/// # Validation
///
/// System instructions take no arguments.
fn build_system_itype(
    itype: &mut IType,
    args: &[Token],
    instruction_name: &str,
) -> Result<IType, Error> {
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

/// Builds a load instruction with flexible syntax support.
///
/// # Load Instruction Formats
///
/// Supports two syntax styles:
/// - Standard RISC-V: `LW rd, offset(rs1)`
/// - Legacy style: `LW rd, rs1, offset`
///
/// # Validation
///
/// - Registers cannot be PC
/// - Offset is sign-extended 12-bit value
fn build_load_itype(itype: &mut IType, args: &[Token]) -> Result<IType, Error> {
    match args {
        // Standard RISC-V syntax: LW rd, offset(rs1)
        [Token::Register(rd), Token::OffsetRegister { offset, register }] => {
            validator::validate_not_pc(*rd, "destination")?;
            validator::validate_not_pc(*register, "base address")?;

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
            validator::validate_not_pc(*rd, "destination")?;
            validator::validate_not_pc(*rs1, "base address")?;

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

/// Builds a store instruction with flexible syntax support.
///
/// # Store Instruction Formats
///
/// Supports two syntax styles:
/// - Standard RISC-V: `SW rs2, offset(rs1)`
/// - Legacy style: `SW rs1, rs2, offset`
///
/// # Validation
///
/// - Registers cannot be PC
/// - Offset is sign-extended 12-bit value
fn build_store_stype(stype: &mut SType, args: &[Token]) -> Result<SType, Error> {
    match args {
        // Standard RISC-V syntax: SW rs2, offset(rs1)
        [Token::Register(rs2), Token::OffsetRegister { offset, register }] => {
            validator::validate_not_pc(*register, "base address")?;
            validator::validate_not_pc(*rs2, "source")?;

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
            validator::validate_not_pc(*rs1, "base address")?;
            validator::validate_not_pc(*rs2, "source")?;

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
