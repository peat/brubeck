//! Tests for CSR (Control and Status Register) instructions
//!
//! CSR instructions provide atomic read-modify-write operations on control
//! and status registers. The RISC-V ISA defines 6 CSR instructions:
//! - CSRRW: Atomic Read/Write CSR
//! - CSRRS: Atomic Read and Set Bits in CSR
//! - CSRRC: Atomic Read and Clear Bits in CSR
//! - CSRRWI: Immediate variant of CSRRW
//! - CSRRSI: Immediate variant of CSRRS
//! - CSRRCI: Immediate variant of CSRRC
//!
//! Reference: RISC-V ISA Manual, Chapter 9 "Zicsr" Extension
//! https://github.com/riscv/riscv-isa-manual/blob/main/src/zicsr.adoc

use crate::unit::test_helpers::*;
use brubeck::rv32_i::{CPU, Error, Register, IType, Instruction};

/// Tests for CSRRW (Atomic Read/Write CSR) instruction
mod csrrw {
    use super::*;

    #[test]
    fn basic_read_write() {
        /// CSRRW atomically swaps values between a CSR and integer register
        /// Old CSR value → rd, rs1 → CSR
        let mut cpu = CpuBuilder::new()
            .with_csr(0x340, 0x12345678)  // mscratch = 0x12345678
            .with_register(Register::X1, 0xABCDEF00)
            .build();

        // CSRRW x2, mscratch, x1
        // Should: mscratch (0x12345678) → x2, x1 (0xABCDEF00) → mscratch
        let result = cpu.execute(Instruction::CSRRW(
            IType::new_with_imm(Register::X2, Register::X1, 0x340)
        ));

        assert!(result.is_ok());
        assert_eq!(cpu.get_register(Register::X2), 0x12345678, "rd should contain old CSR value");
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0xABCDEF00, "CSR should contain rs1 value");
    }

    #[test]
    fn write_without_read() {
        /// When rd=x0, CSRRW should not read the CSR (avoiding side effects)
        /// Only rs1 → CSR
        let mut cpu = CpuBuilder::new()
            .with_csr(0x340, 0x12345678)
            .with_register(Register::X1, 0xABCDEF00)
            .build();

        // CSRRW x0, mscratch, x1
        // Should: x1 (0xABCDEF00) → mscratch, no read
        let result = cpu.execute(Instruction::CSRRW(
            IType::new_with_imm(Register::X0, Register::X1, 0x340)
        ));

        assert!(result.is_ok());
        assert_eq!(cpu.get_register(Register::X0), 0, "x0 should remain zero");
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0xABCDEF00, "CSR should contain rs1 value");
    }

    #[test]
    fn write_zero_from_x0() {
        /// CSRRW with rs1=x0 writes zero to the CSR
        let mut cpu = CpuBuilder::new()
            .with_csr(0x340, 0x12345678)
            .build();

        // CSRRW x1, mscratch, x0
        // Should: mscratch (0x12345678) → x1, x0 (0) → mscratch
        let result = cpu.execute(Instruction::CSRRW(
            IType::new_with_imm(Register::X1, Register::X0, 0x340)
        ));

        assert!(result.is_ok());
        assert_eq!(cpu.get_register(Register::X1), 0x12345678, "rd should contain old CSR value");
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0, "CSR should be cleared");
    }

    #[test]
    fn invalid_csr_address() {
        /// Accessing a non-existent CSR should raise an illegal instruction exception
        let mut cpu = CPU::default();

        // CSRRW x1, 0xFFF, x2 (CSR 0xFFF doesn't exist)
        let result = cpu.execute(Instruction::CSRRW(
            IType::new_with_imm(Register::X1, Register::X2, 0xFFF)
        ));

        assert!(matches!(result, Err(Error::IllegalInstruction(_))));
    }

    #[test]
    fn read_only_csr() {
        /// Writing to a read-only CSR should raise an illegal instruction exception
        let mut cpu = CPU::default();
        
        // Cycle counter (0xC00) is read-only
        // CSRRW x1, cycle, x2
        let result = cpu.execute(Instruction::CSRRW(
            IType::new_with_imm(Register::X1, Register::X2, 0xC00)
        ));

        assert!(matches!(result, Err(Error::IllegalInstruction(_))));
    }
}

/// Tests for CSRRS (Atomic Read and Set Bits in CSR) instruction
mod csrrs {
    use super::*;

    #[test]
    fn basic_set_bits() {
        /// CSRRS atomically sets bits in a CSR based on rs1
        /// Old CSR value → rd, CSR | rs1 → CSR
        let mut cpu = CpuBuilder::new()
            .with_csr(0x340, 0x12340000)  // mscratch = 0x12340000
            .with_register(Register::X1, 0x00005678)
            .build();

        // CSRRS x2, mscratch, x1
        // Should: mscratch (0x12340000) → x2, 0x12340000 | 0x00005678 → mscratch
        let result = cpu.execute(Instruction::CSRRS(
            IType::new_with_imm(Register::X2, Register::X1, 0x340)
        ));

        assert!(result.is_ok());
        assert_eq!(cpu.get_register(Register::X2), 0x12340000, "rd should contain old CSR value");
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0x12345678, "CSR should have bits set");
    }

    #[test]
    fn read_without_write() {
        /// When rs1=x0, CSRRS should only read the CSR, not modify it
        let mut cpu = CpuBuilder::new()
            .with_csr(0x340, 0x12345678)
            .build();

        // CSRRS x1, mscratch, x0
        // Should: mscratch (0x12345678) → x1, no modification
        let result = cpu.execute(Instruction::CSRRS(
            IType::new_with_imm(Register::X1, Register::X0, 0x340)
        ));

        assert!(result.is_ok());
        assert_eq!(cpu.get_register(Register::X1), 0x12345678, "rd should contain CSR value");
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0x12345678, "CSR should be unchanged");
    }

    #[test]
    fn set_bits_on_read_only_csr_with_x0() {
        /// CSRRS with rs1=x0 on a read-only CSR should succeed (only reading)
        let mut cpu = CPU::default();

        // CSRRS x1, cycle, x0 (read cycle counter)
        let result = cpu.execute(Instruction::CSRRS(
            IType::new_with_imm(Register::X1, Register::X0, 0xC00)
        ));

        if let Err(e) = &result {
            println!("Error: {:?}", e);
        }
        assert!(result.is_ok());
        // We can't predict the exact cycle count, just verify it succeeded
    }

    #[test]
    fn warl_field_behavior() {
        /// WARL (Write Any Read Legal) fields should mask illegal bit writes
        /// mstatus has WARL behavior - only certain bits are writable
        let mut cpu = CpuBuilder::new()
            .with_csr(0x300, 0x00000000)  // mstatus = 0
            .with_register(Register::X1, 0xFFFFFFFF)  // Try to set all bits
            .build();

        // CSRRS x2, mstatus, x1
        let result = cpu.execute(Instruction::CSRRS(
            IType::new_with_imm(Register::X2, Register::X1, 0x300)
        ));

        assert!(result.is_ok());
        let mstatus = cpu.read_csr(0x300).unwrap();
        assert_eq!(mstatus & 0x00001888, mstatus, "Only WARL bits should be set");
    }
}

/// Tests for CSRRC (Atomic Read and Clear Bits in CSR) instruction
mod csrrc {
    use super::*;

    #[test]
    fn basic_clear_bits() {
        /// CSRRC atomically clears bits in a CSR based on rs1
        /// Old CSR value → rd, CSR & ~rs1 → CSR
        let mut cpu = CpuBuilder::new()
            .with_csr(0x340, 0x12345678)  // mscratch = 0x12345678
            .with_register(Register::X1, 0x0000FF00)
            .build();

        // CSRRC x2, mscratch, x1
        // Should: mscratch (0x12345678) → x2, 0x12345678 & ~0x0000FF00 → mscratch
        let result = cpu.execute(Instruction::CSRRC(
            IType::new_with_imm(Register::X2, Register::X1, 0x340)
        ));

        assert!(result.is_ok());
        assert_eq!(cpu.get_register(Register::X2), 0x12345678, "rd should contain old CSR value");
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0x12340078, "CSR should have bits cleared");
    }

    #[test]
    fn read_without_clear() {
        /// When rs1=x0, CSRRC should only read the CSR, not modify it
        let mut cpu = CpuBuilder::new()
            .with_csr(0x340, 0x12345678)
            .build();

        // CSRRC x1, mscratch, x0
        // Should: mscratch (0x12345678) → x1, no modification
        let result = cpu.execute(Instruction::CSRRC(
            IType::new_with_imm(Register::X1, Register::X0, 0x340)
        ));

        assert!(result.is_ok());
        assert_eq!(cpu.get_register(Register::X1), 0x12345678, "rd should contain CSR value");
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0x12345678, "CSR should be unchanged");
    }
}

/// Tests for CSRRWI (Atomic Read/Write CSR Immediate) instruction
mod csrrwi {
    use super::*;

    #[test]
    fn basic_immediate_write() {
        /// CSRRWI atomically writes a 5-bit immediate to a CSR
        /// Old CSR value → rd, zero-extended uimm → CSR
        let mut cpu = CpuBuilder::new()
            .with_csr(0x340, 0x12345678)
            .build();

        // CSRRWI x1, mscratch, 31 (0x1F - max 5-bit value)
        // Should: mscratch (0x12345678) → x1, 31 → mscratch
        let result = cpu.execute(Instruction::CSRRWI(
            IType::new_with_imm(Register::X1, Register::from_u32(31), 0x340)
        ));

        assert!(result.is_ok());
        assert_eq!(cpu.get_register(Register::X1), 0x12345678, "rd should contain old CSR value");
        assert_eq!(cpu.read_csr(0x340).unwrap(), 31, "CSR should contain immediate value");
    }

    #[test]
    fn immediate_write_without_read() {
        /// When rd=x0, CSRRWI should not read the CSR
        let mut cpu = CpuBuilder::new()
            .with_csr(0x340, 0x12345678)
            .build();

        // CSRRWI x0, mscratch, 15
        let result = cpu.execute(Instruction::CSRRWI(
            IType::new_with_imm(Register::X0, Register::from_u32(15), 0x340)
        ));

        assert!(result.is_ok());
        assert_eq!(cpu.get_register(Register::X0), 0, "x0 should remain zero");
        assert_eq!(cpu.read_csr(0x340).unwrap(), 15, "CSR should contain immediate value");
    }

    #[test]
    fn five_bit_immediate_limit() {
        /// The immediate is only 5 bits, so values > 31 should be masked
        let mut cpu = CpuBuilder::new()
            .with_csr(0x340, 0)
            .build();

        // If we somehow get a value > 31, it should be masked to 5 bits
        // This test documents the expected behavior
        // CSRRWI x1, mscratch, 0x3F (63) - should be masked to 0x1F (31)
        let result = cpu.execute(Instruction::CSRRWI(
            IType::new_with_imm(Register::X1, Register::from_u32(0x3F), 0x340)
        ));

        assert!(result.is_ok());
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0x1F, "Only 5 bits should be used");
    }
}

/// Tests for CSRRSI (Atomic Read and Set Bits in CSR Immediate) instruction
mod csrrsi {
    use super::*;

    #[test]
    fn basic_immediate_set() {
        /// CSRRSI atomically sets bits in a CSR based on a 5-bit immediate
        let mut cpu = CpuBuilder::new()
            .with_csr(0x340, 0x12340000)
            .build();

        // CSRRSI x1, mscratch, 0x18 (set bits 3 and 4)
        let result = cpu.execute(Instruction::CSRRSI(
            IType::new_with_imm(Register::X1, Register::from_u32(0x18), 0x340)
        ));

        assert!(result.is_ok());
        assert_eq!(cpu.get_register(Register::X1), 0x12340000, "rd should contain old CSR value");
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0x12340018, "CSR should have bits set");
    }

    #[test]
    fn read_without_set() {
        /// When uimm=0, CSRRSI should only read the CSR
        let mut cpu = CpuBuilder::new()
            .with_csr(0x340, 0x12345678)
            .build();

        // CSRRSI x1, mscratch, 0
        let result = cpu.execute(Instruction::CSRRSI(
            IType::new_with_imm(Register::X1, Register::from_u32(0), 0x340)
        ));

        assert!(result.is_ok());
        assert_eq!(cpu.get_register(Register::X1), 0x12345678, "rd should contain CSR value");
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0x12345678, "CSR should be unchanged");
    }

    #[test]
    fn set_on_read_only_with_zero() {
        /// CSRRSI with uimm=0 on read-only CSR should succeed
        let mut cpu = CPU::default();

        // CSRRSI x1, cycle, 0
        let result = cpu.execute(Instruction::CSRRSI(
            IType::new_with_imm(Register::X1, Register::from_u32(0), 0xC00)
        ));

        assert!(result.is_ok());
    }
}

/// Tests for CSRRCI (Atomic Read and Clear Bits in CSR Immediate) instruction
mod csrrci {
    use super::*;

    #[test]
    fn basic_immediate_clear() {
        /// CSRRCI atomically clears bits in a CSR based on a 5-bit immediate
        let mut cpu = CpuBuilder::new()
            .with_csr(0x340, 0x1234567F)
            .build();

        // CSRRCI x1, mscratch, 0x0F (clear bits 0-3)
        let result = cpu.execute(Instruction::CSRRCI(
            IType::new_with_imm(Register::X1, Register::from_u32(0x0F), 0x340)
        ));

        assert!(result.is_ok());
        assert_eq!(cpu.get_register(Register::X1), 0x1234567F, "rd should contain old CSR value");
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0x12345670, "CSR should have bits cleared");
    }

    #[test]
    fn read_without_clear() {
        /// When uimm=0, CSRRCI should only read the CSR
        let mut cpu = CpuBuilder::new()
            .with_csr(0x340, 0x12345678)
            .build();

        // CSRRCI x1, mscratch, 0
        let result = cpu.execute(Instruction::CSRRCI(
            IType::new_with_imm(Register::X1, Register::from_u32(0), 0x340)
        ));

        assert!(result.is_ok());
        assert_eq!(cpu.get_register(Register::X1), 0x12345678, "rd should contain CSR value");
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0x12345678, "CSR should be unchanged");
    }
}

/// Tests for atomic operation semantics
mod atomic_operations {
    use super::*;

    #[test]
    fn csrrs_is_atomic() {
        /// Verify that CSRRS performs atomic read-modify-write
        /// The old value returned should be before any modification
        let mut cpu = CpuBuilder::new()
            .with_csr(0x340, 0x00001234)
            .with_register(Register::X1, 0x00005678)
            .build();

        // CSRRS should return old value and set bits atomically
        let result = cpu.execute(Instruction::CSRRS(
            IType::new_with_imm(Register::X2, Register::X1, 0x340)
        ));

        assert!(result.is_ok());
        assert_eq!(cpu.get_register(Register::X2), 0x00001234, "Should return value before modification");
        // 0x00001234 | 0x00005678 = 0x0000567C
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0x0000567C, "Should have bits set");
    }

    #[test]
    fn csrrc_is_atomic() {
        /// Verify that CSRRC performs atomic read-modify-write
        let mut cpu = CpuBuilder::new()
            .with_csr(0x340, 0x0000FFFF)
            .with_register(Register::X1, 0x00000FF0)
            .build();

        // CSRRC should return old value and clear bits atomically
        let result = cpu.execute(Instruction::CSRRC(
            IType::new_with_imm(Register::X2, Register::X1, 0x340)
        ));

        assert!(result.is_ok());
        assert_eq!(cpu.get_register(Register::X2), 0x0000FFFF, "Should return value before modification");
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0x0000F00F, "Should have bits cleared");
    }
}

/// Tests for common CSR patterns and use cases
mod csr_patterns {
    use super::*;

    #[test]
    fn read_csr_idiom() {
        /// Common pattern: CSRRS rd, csr, x0 to read a CSR
        let mut cpu = CpuBuilder::new()
            .with_csr(0x340, 0xDEADBEEF)
            .build();

        // Read mscratch using CSRRS
        let result = cpu.execute(Instruction::CSRRS(
            IType::new_with_imm(Register::X5, Register::X0, 0x340)
        ));

        assert!(result.is_ok());
        assert_eq!(cpu.get_register(Register::X5), 0xDEADBEEF);
    }

    #[test]
    fn write_csr_idiom() {
        /// Common pattern: CSRRW x0, csr, rs to write a CSR
        let mut cpu = CpuBuilder::new()
            .with_register(Register::X5, 0xCAFEBABE)
            .build();

        // Write mscratch using CSRRW
        let result = cpu.execute(Instruction::CSRRW(
            IType::new_with_imm(Register::X0, Register::X5, 0x340)
        ));

        assert!(result.is_ok());
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0xCAFEBABE);
    }

    #[test]
    fn swap_with_csr() {
        /// Swap a register value with a CSR value
        let mut cpu = CpuBuilder::new()
            .with_csr(0x340, 0xAAAAAAAA)
            .with_register(Register::X10, 0x55555555)
            .build();

        // CSRRW x10, mscratch, x10 - swap x10 with mscratch
        let result = cpu.execute(Instruction::CSRRW(
            IType::new_with_imm(Register::X10, Register::X10, 0x340)
        ));

        assert!(result.is_ok());
        assert_eq!(cpu.get_register(Register::X10), 0xAAAAAAAA, "x10 should have old CSR value");
        assert_eq!(cpu.read_csr(0x340).unwrap(), 0x55555555, "CSR should have old x10 value");
    }
}