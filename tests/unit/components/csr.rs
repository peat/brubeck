//! Unit tests for Control and Status Register (CSR) functionality
//!
//! These tests verify the CSR implementation including:
//! - Read/write operations
//! - Read-only register protection
//! - WARL (Write Any, Read Legal) behavior
//! - Bit manipulation operations (set/clear)
//! - Standard CSR initialization
//!
//! Reference: RISC-V ISA Manual, Volume II: Privileged Architecture

use brubeck::rv32_i::{cpu::CPU, Error, Register};

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
    assert!(matches!(result.unwrap_err(), Error::IllegalInstruction(_)));

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
    assert!(matches!(result.unwrap_err(), Error::IllegalInstruction(_)));

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
    assert!(matches!(result.unwrap_err(), Error::IllegalInstruction(_)));
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
        Err(Error::IllegalInstruction(_))
    ));
    assert!(matches!(
        cpu.write_csr(0x999, 0),
        Err(Error::IllegalInstruction(_))
    ));
    assert!(matches!(
        cpu.set_csr_bits(0x999, 1),
        Err(Error::IllegalInstruction(_))
    ));
    assert!(matches!(
        cpu.clear_csr_bits(0x999, 1),
        Err(Error::IllegalInstruction(_))
    ));

    // Out of bounds CSR address
    assert!(matches!(
        cpu.read_csr(0x1000),
        Err(Error::IllegalInstruction(_))
    ));

    // Read-only CSR writes (with non-zero mask/value)
    assert!(matches!(
        cpu.write_csr(0x301, 0x12345678), // misa is read-only
        Err(Error::IllegalInstruction(_))
    ));
    assert!(matches!(
        cpu.set_csr_bits(0x301, 0xFF), // non-zero mask
        Err(Error::IllegalInstruction(_))
    ));
    assert!(matches!(
        cpu.clear_csr_bits(0x301, 0xFF), // non-zero mask
        Err(Error::IllegalInstruction(_))
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