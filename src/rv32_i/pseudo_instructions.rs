//! RV32I pseudo-instructions
//!
//! This module provides support for common RISC-V pseudo-instructions that
//! expand to RV32I base instructions. Pseudo-instructions are assembly
//! mnemonics that expand to one or more real instructions, making assembly
//! code more readable and convenient.
//!
//! All pseudo-instructions in this module are specific to the RV32I base
//! integer instruction set and only expand to RV32I instructions.
//!
//! Reference: RISC-V ISA Manual mentions these throughout, and they are
//! documented in the RISC-V Assembly Programmer's Manual.

use super::{
    formats::{IType, JType, RType, UType},
    instructions::Instruction,
    registers::Register,
};
use crate::Immediate;

/// Represents an RV32I pseudo-instruction
///
/// These pseudo-instructions are specific to the RV32I base integer
/// instruction set and expand only to RV32I instructions.
#[derive(Debug, Clone, PartialEq)]
pub enum PseudoInstruction {
    /// MV rd, rs - Move register
    /// Expands to: ADDI rd, rs, 0
    MV { rd: Register, rs: Register },

    /// NOT rd, rs - Bitwise NOT
    /// Expands to: XORI rd, rs, -1
    NOT { rd: Register, rs: Register },

    /// SEQZ rd, rs - Set if equal to zero
    /// Expands to: SLTIU rd, rs, 1
    SEQZ { rd: Register, rs: Register },

    /// SNEZ rd, rs - Set if not equal to zero
    /// Expands to: SLTU rd, x0, rs
    SNEZ { rd: Register, rs: Register },

    /// J offset - Unconditional jump
    /// Expands to: JAL x0, offset
    J { offset: i32 },

    /// JR rs - Jump register (indirect jump)
    /// Expands to: JALR x0, rs, 0
    JR { rs: Register },

    /// RET - Return from subroutine
    /// Expands to: JALR x0, x1, 0
    RET,

    /// LI rd, imm - Load immediate
    /// This is more complex and may expand to multiple instructions:
    /// - For small immediates: ADDI rd, x0, imm
    /// - For larger immediates: LUI + ADDI sequence
    LI { rd: Register, imm: i32 },

    /// LA rd, symbol - Load address (placeholder for now)
    /// In a real assembler, this would load the address of a symbol
    LA { rd: Register, symbol: String },
}

impl PseudoInstruction {
    /// Expand a pseudo-instruction into real RISC-V instructions
    pub fn expand(&self) -> Result<Vec<Instruction>, String> {
        match self {
            PseudoInstruction::MV { rd, rs } => {
                // MV rd, rs => ADDI rd, rs, 0
                let mut inst = IType {
                    rd: *rd,
                    rs1: *rs,
                    imm: Immediate::new(12),
                    ..Default::default()
                };
                inst.imm
                    .set_signed(0)
                    .map_err(|_| "Failed to set immediate")?;
                Ok(vec![Instruction::ADDI(inst)])
            }

            PseudoInstruction::NOT { rd, rs } => {
                // NOT rd, rs => XORI rd, rs, -1
                let mut inst = IType {
                    rd: *rd,
                    rs1: *rs,
                    imm: Immediate::new(12),
                    ..Default::default()
                };
                inst.imm
                    .set_signed(-1)
                    .map_err(|_| "Failed to set immediate")?;
                Ok(vec![Instruction::XORI(inst)])
            }

            PseudoInstruction::SEQZ { rd, rs } => {
                // SEQZ rd, rs => SLTIU rd, rs, 1
                let mut inst = IType {
                    rd: *rd,
                    rs1: *rs,
                    imm: Immediate::new(12),
                    ..Default::default()
                };
                inst.imm
                    .set_unsigned(1)
                    .map_err(|_| "Failed to set immediate")?;
                Ok(vec![Instruction::SLTIU(inst)])
            }

            PseudoInstruction::SNEZ { rd, rs } => {
                // SNEZ rd, rs => SLTU rd, x0, rs
                let inst = RType {
                    rd: *rd,
                    rs1: Register::X0,
                    rs2: *rs,
                    ..Default::default()
                };
                Ok(vec![Instruction::SLTU(inst)])
            }

            PseudoInstruction::J { offset } => {
                // J offset => JAL x0, offset
                let mut inst = JType {
                    rd: Register::X0,
                    ..Default::default()
                };

                // JAL offset is in multiples of 2
                if offset % 2 != 0 {
                    return Err("Jump offset must be even".to_string());
                }
                let encoded_offset = offset / 2;

                inst.imm = Immediate::new(20);
                inst.imm
                    .set_signed(encoded_offset)
                    .map_err(|_| "Jump offset out of range")?;
                Ok(vec![Instruction::JAL(inst)])
            }

            PseudoInstruction::JR { rs } => {
                // JR rs => JALR x0, rs, 0
                let mut inst = IType {
                    rd: Register::X0,
                    rs1: *rs,
                    imm: Immediate::new(12),
                    ..Default::default()
                };
                inst.imm
                    .set_signed(0)
                    .map_err(|_| "Failed to set immediate")?;
                Ok(vec![Instruction::JALR(inst)])
            }

            PseudoInstruction::RET => {
                // RET => JALR x0, x1, 0
                let mut inst = IType {
                    rd: Register::X0,
                    rs1: Register::X1, // Return address register
                    imm: Immediate::new(12),
                    ..Default::default()
                };
                inst.imm
                    .set_signed(0)
                    .map_err(|_| "Failed to set immediate")?;
                Ok(vec![Instruction::JALR(inst)])
            }

            PseudoInstruction::LI { rd, imm } => {
                // LI is complex - it depends on the immediate value
                self.expand_li(*rd, *imm)
            }

            PseudoInstruction::LA { .. } => {
                // LA (Load Address) requires a full assembler with symbol table support
                Err("LA pseudo-instruction requires symbol resolution".to_string())
            }
        }
    }

    /// Expand LI (load immediate) pseudo-instruction
    /// This is complex because it needs to handle various immediate sizes
    fn expand_li(&self, rd: Register, imm: i32) -> Result<Vec<Instruction>, String> {
        // If immediate fits in 12 bits (signed), use ADDI
        if (-2048..=2047).contains(&imm) {
            let mut inst = IType {
                rd,
                rs1: Register::X0,
                imm: Immediate::new(12),
                ..Default::default()
            };
            inst.imm
                .set_signed(imm)
                .map_err(|_| "Failed to set immediate")?;
            return Ok(vec![Instruction::ADDI(inst)]);
        }

        // For larger values, we need LUI + ADDI
        // Extract upper 20 bits and lower 12 bits
        let lower = imm & 0xFFF;
        let upper = (imm >> 12) & 0xFFFFF;

        // If lower 12 bits form a negative number when sign-extended,
        // we need to adjust the upper bits
        let (upper, lower) = if lower & 0x800 != 0 {
            // Lower will be negative when sign-extended
            let lower = lower | 0xFFFFF000_u32 as i32; // Sign extend
            let upper = ((imm.wrapping_sub(lower)) >> 12) & 0xFFFFF;
            (upper, lower)
        } else {
            (upper, lower)
        };

        let mut instructions = Vec::new();

        // LUI rd, upper
        let mut lui = UType {
            rd,
            imm: Immediate::new(20),
            ..Default::default()
        };
        lui.imm
            .set_unsigned(upper as u32)
            .map_err(|_| "Failed to set LUI immediate")?;
        instructions.push(Instruction::LUI(lui));

        // ADDI rd, rd, lower (if lower is non-zero)
        if lower != 0 {
            let mut addi = IType {
                rd,
                rs1: rd,
                imm: Immediate::new(12),
                ..Default::default()
            };
            addi.imm
                .set_signed(lower)
                .map_err(|_| "Failed to set ADDI immediate")?;
            instructions.push(Instruction::ADDI(addi));
        }

        Ok(instructions)
    }

    /// Check if this pseudo-instruction is valid
    pub fn validate(&self) -> Result<(), String> {
        match self {
            PseudoInstruction::MV { rd, rs } => {
                if rd == rs {
                    Err("MV with same source and destination is redundant".to_string())
                } else {
                    Ok(())
                }
            }
            PseudoInstruction::J { offset } => {
                if offset % 2 != 0 {
                    Err("Jump offset must be even".to_string())
                } else if *offset < -1048576 || *offset > 1048575 {
                    Err("Jump offset out of range for JAL".to_string())
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }
}

/// Helper to identify if an instruction sequence might be a pseudo-instruction
/// This is useful for disassembly
pub fn detect_pseudo_instruction(inst: &Instruction) -> Option<PseudoInstruction> {
    match inst {
        // ADDI rd, rs, 0 => MV rd, rs
        Instruction::ADDI(i) if i.imm.as_i32() == 0 && i.rd != i.rs1 => {
            Some(PseudoInstruction::MV {
                rd: i.rd,
                rs: i.rs1,
            })
        }

        // XORI rd, rs, -1 => NOT rd, rs
        Instruction::XORI(i) if i.imm.as_i32() == -1 => Some(PseudoInstruction::NOT {
            rd: i.rd,
            rs: i.rs1,
        }),

        // SLTIU rd, rs, 1 => SEQZ rd, rs
        Instruction::SLTIU(i) if i.imm.as_u32() == 1 => Some(PseudoInstruction::SEQZ {
            rd: i.rd,
            rs: i.rs1,
        }),

        // SLTU rd, x0, rs => SNEZ rd, rs
        Instruction::SLTU(r) if r.rs1 == Register::X0 => Some(PseudoInstruction::SNEZ {
            rd: r.rd,
            rs: r.rs2,
        }),

        // JAL x0, offset => J offset
        Instruction::JAL(j) if j.rd == Register::X0 => {
            let offset = j.imm.as_i32() * 2; // JAL offset is in multiples of 2
            Some(PseudoInstruction::J { offset })
        }

        // JALR x0, rs, 0 => JR rs (or RET if rs is x1)
        Instruction::JALR(i) if i.rd == Register::X0 && i.imm.as_i32() == 0 => {
            if i.rs1 == Register::X1 {
                Some(PseudoInstruction::RET)
            } else {
                Some(PseudoInstruction::JR { rs: i.rs1 })
            }
        }

        // ADDI rd, x0, imm => Part of LI (for small immediates)
        Instruction::ADDI(i) if i.rs1 == Register::X0 => Some(PseudoInstruction::LI {
            rd: i.rd,
            imm: i.imm.as_i32(),
        }),

        _ => None,
    }
}

// Tests have been moved to tests/unit/instructions/pseudo.rs
