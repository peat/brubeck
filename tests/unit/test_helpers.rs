//! Common test helpers and utilities for RISC-V instruction testing
//!
//! This module provides reusable patterns and helpers to make tests
//! more readable, maintainable, and educational.

#![allow(dead_code)]

use brubeck::rv32_i::{cpu::CPU, instructions::Instruction, registers::Register};

/// Common test values with semantic meaning
#[allow(dead_code)]
pub mod values {

    // Basic values
    pub const ZERO: u32 = 0;
    pub const ONE: u32 = 1;
    pub const TWO: u32 = 2;
    pub const TEN: u32 = 10;

    // Boundary values
    pub const U32_MAX: u32 = u32::MAX;
    pub const U32_MIN: u32 = u32::MIN;
    pub const I32_MAX: u32 = i32::MAX as u32;
    pub const I32_MIN: u32 = i32::MIN as u32;

    // Sign-related values
    pub const NEG_ONE: u32 = -1_i32 as u32;
    pub const NEG_TWO: u32 = -2_i32 as u32;
    pub const MSB_SET: u32 = 0x8000_0000;
    pub const MSB_CLEAR: u32 = 0x7FFF_FFFF;

    // Common patterns
    pub const ALL_ONES: u32 = 0xFFFF_FFFF;
    pub const ALL_ZEROS: u32 = 0x0000_0000;
    pub const ALTERNATING_BITS: u32 = 0xAAAA_AAAA;
    pub const ALTERNATING_BITS_INV: u32 = 0x5555_5555;

    // Memory addresses (typical values)
    pub const STACK_BASE: u32 = 0x8000_0000;
    pub const HEAP_BASE: u32 = 0x1000_0000;
    pub const CODE_BASE: u32 = 0x0000_1000;
    pub const TEST_ADDR: u32 = 1024; // Arbitrary but consistent

    // Immediate boundaries
    pub const IMM12_MAX: i32 = 2047; // Maximum 12-bit signed immediate
    pub const IMM12_MIN: i32 = -2048; // Minimum 12-bit signed immediate
    pub const IMM20_MAX: u32 = 0xFFFFF; // Maximum 20-bit unsigned immediate

    // Shift amounts
    pub const SHIFT_0: u32 = 0;
    pub const SHIFT_1: u32 = 1;
    pub const SHIFT_31: u32 = 31;
    pub const SHIFT_32: u32 = 32; // Tests 5-bit masking
}

/// Builder pattern for setting up CPU state
pub struct CpuBuilder {
    cpu: CPU,
}

impl CpuBuilder {
    pub fn new() -> Self {
        Self {
            cpu: CPU::default(),
        }
    }

    /// Set a single register value
    pub fn with_register(mut self, reg: Register, value: u32) -> Self {
        self.cpu.set_register(reg, value);
        self
    }

    /// Set multiple register values
    pub fn with_registers(mut self, regs: &[(Register, u32)]) -> Self {
        for (reg, val) in regs {
            self.cpu.set_register(*reg, *val);
        }
        self
    }

    /// Set PC to a specific value
    pub fn with_pc(mut self, pc: u32) -> Self {
        self.cpu.pc = pc;
        self
    }

    /// Write a byte to memory
    pub fn with_memory_byte(mut self, addr: u32, value: u8) -> Self {
        self.cpu.memory[addr as usize] = value;
        self
    }

    /// Write a word to memory in little-endian format
    pub fn with_memory_word_le(mut self, addr: u32, value: u32) -> Self {
        let addr = addr as usize;
        self.cpu.memory[addr] = (value & 0xFF) as u8;
        self.cpu.memory[addr + 1] = ((value >> 8) & 0xFF) as u8;
        self.cpu.memory[addr + 2] = ((value >> 16) & 0xFF) as u8;
        self.cpu.memory[addr + 3] = ((value >> 24) & 0xFF) as u8;
        self
    }

    /// Write a test pattern to memory
    pub fn with_memory_pattern(mut self, start_addr: u32, pattern: &[u8]) -> Self {
        let start = start_addr as usize;
        self.cpu.memory[start..start + pattern.len()].copy_from_slice(pattern);
        self
    }

    /// Set a CSR value
    pub fn with_csr(mut self, csr: u16, value: u32) -> Self {
        self.cpu
            .write_csr(csr, value)
            .expect("Failed to set CSR in test");
        self
    }

    pub fn build(self) -> CPU {
        self.cpu
    }
}

/// Extension trait for CPU to add test-specific assertions
pub trait CpuAssertions {
    /// Assert a single register value with context
    fn assert_register(&self, reg: Register, expected: u32, context: &str);

    /// Assert multiple register values
    fn assert_registers(&self, expected: &[(Register, u32)]);

    /// Assert PC value
    fn assert_pc(&self, expected: u32, context: &str);

    /// Assert memory word (little-endian)
    fn assert_memory_word_le(&self, addr: u32, expected: u32, context: &str);

    /// Assert memory bytes
    fn assert_memory_bytes(&self, addr: u32, expected: &[u8], context: &str);
}

impl CpuAssertions for CPU {
    fn assert_register(&self, reg: Register, expected: u32, context: &str) {
        let actual = self.get_register(reg);
        assert_eq!(
            actual,
            expected,
            "{}: Register {:?} = {:#010x} (expected {:#010x}, diff: {})",
            context,
            reg,
            actual,
            expected,
            if actual > expected {
                format!("+{:#x}", actual - expected)
            } else {
                format!("-{:#x}", expected - actual)
            }
        );
    }

    fn assert_registers(&self, expected: &[(Register, u32)]) {
        for (reg, val) in expected {
            self.assert_register(*reg, *val, &format!("Register {reg:?} check"));
        }
    }

    fn assert_pc(&self, expected: u32, context: &str) {
        assert_eq!(
            self.pc, expected,
            "{}: PC = {:#010x} (expected {:#010x})",
            context, self.pc, expected
        );
    }

    fn assert_memory_word_le(&self, addr: u32, expected: u32, context: &str) {
        let addr = addr as usize;
        let actual = u32::from_le_bytes([
            self.memory[addr],
            self.memory[addr + 1],
            self.memory[addr + 2],
            self.memory[addr + 3],
        ]);
        assert_eq!(
            actual, expected,
            "{context}: Memory[{addr:#x}] = {actual:#010x} (expected {expected:#010x})"
        );
    }

    fn assert_memory_bytes(&self, addr: u32, expected: &[u8], context: &str) {
        let start = addr as usize;
        let actual = &self.memory[start..start + expected.len()];
        assert_eq!(
            actual,
            expected,
            "{}: Memory[{:#x}..{:#x}] mismatch",
            context,
            addr,
            addr + expected.len() as u32
        );
    }
}

/// Helper for executing instructions with better error reporting
pub trait ExecuteWithContext {
    fn execute_expect(&mut self, inst: Instruction, context: &str);
    fn execute_and_assert_ok(&mut self, inst: Instruction) -> &mut Self;
}

impl ExecuteWithContext for CPU {
    fn execute_expect(&mut self, inst: Instruction, context: &str) {
        if let Err(e) = self.execute(inst) {
            panic!("{context}: Execution failed with {e:?}");
        }
    }

    fn execute_and_assert_ok(&mut self, inst: Instruction) -> &mut Self {
        self.execute(inst)
            .expect("Instruction execution should succeed");
        self
    }
}

/// Helper for creating test tables with descriptive test cases
pub struct TestCase<T> {
    pub inputs: T,
    pub expected: u32,
    pub description: &'static str,
}

/// Macro for creating readable test tables
#[macro_export]
macro_rules! test_cases {
    ($($inputs:expr => $expected:expr ; $desc:expr),* $(,)?) => {
        vec![
            $(TestCase {
                inputs: $inputs,
                expected: $expected,
                description: $desc,
            }),*
        ]
    };
}

/// Helper for documenting instruction encoding
pub fn document_encoding(instruction: &str, format: &str, encoding_bits: &str) {
    println!("Instruction: {instruction}");
    println!("Format: {format}");
    println!("Encoding: {encoding_bits}");
    println!();
}

/// Memory visualization helper for debugging
pub fn visualize_memory(cpu: &CPU, start_addr: u32, length: usize) {
    println!("Memory dump starting at {start_addr:#010x}:");
    println!("Address  | +0  +1  +2  +3  +4  +5  +6  +7  | ASCII");
    println!("---------+--------------------------------+--------");

    let start = start_addr as usize;
    for offset in (0..length).step_by(8) {
        print!("{:#08x} |", start_addr + offset as u32);

        // Hex values
        for i in 0..8 {
            if offset + i < length {
                print!(" {:02x}", cpu.memory[start + offset + i]);
            } else {
                print!("   ");
            }
        }
        print!(" | ");

        // ASCII representation
        for i in 0..8 {
            if offset + i < length {
                let byte = cpu.memory[start + offset + i];
                if (0x20..=0x7E).contains(&byte) {
                    print!("{}", byte as char);
                } else {
                    print!(".");
                }
            }
        }
        println!();
    }
}
