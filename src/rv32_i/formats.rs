//! Encoding formats for RV32I instructions.

use crate::rv32_i::Register;
use crate::Immediate;

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct RType {
    pub opcode: u8,
    pub rd: Register,
    pub funct3: u8,
    pub rs1: Register,
    pub rs2: Register,
    pub funct7: u8,
}

#[derive(Debug, Copy, Clone, PartialEq)]
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
            opcode: 0,
            rd: Register::default(),
            funct3: 0,
            rs1: Register::default(),
            imm: Immediate::new(Self::IMM_BITS),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SType {
    pub opcode: u8,
    pub imm: Immediate,
    pub funct3: u8,
    pub rs1: Register,
    pub rs2: Register,
}

impl Default for SType {
    fn default() -> Self {
        Self::new()
    }
}

impl SType {
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

#[derive(Debug, Copy, Clone, PartialEq)]
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

#[derive(Debug, Copy, Clone, PartialEq)]
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
            opcode: 0,
            rd: Register::default(),
            imm: Immediate::new(Self::IMM_BITS),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
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
            opcode: 0,
            rd: Register::default(),
            imm: Immediate::new(Self::IMM_BITS),
        }
    }
}
