//! Unit tests for immediate value handling
//!
//! These tests verify sign extension, bounds checking, and the behavior
//! of immediate values used throughout the RISC-V ISA.
//!
//! Reference: RISC-V ISA Manual, Volume I: Unprivileged ISA, Version 20191213
//! Chapter 2 - RV32I Base Integer Instruction Set
//!
//! Key concepts:
//! - RISC-V immediates are ALWAYS sign-extended to 32 bits
//! - Different instruction types use different immediate widths:
//!   - I-type: 12-bit immediates (ADDI, LW, etc.)
//!   - S-type: 12-bit immediates (SW, etc.)
//!   - B-type: 12-bit immediates (branches)
//!   - U-type: 20-bit immediates (LUI, AUIPC)
//!   - J-type: 20-bit immediates (JAL)
//!
//! Sign extension visualization (12-bit to 32-bit):
//! ```
//! 12-bit: 0x800 = 1000_0000_0000 (bit 11 set)
//! 32-bit: 0xFFFFF800 = 1111_1111_1111_1111_1111_1000_0000_0000
//!                      ^^^^^^^^^^^^^^^^^^^^^ sign bits extended
//!
//! 12-bit: 0x7FF = 0111_1111_1111 (bit 11 clear)
//! 32-bit: 0x000007FF = 0000_0000_0000_0000_0000_0111_1111_1111
//!                      ^^^^^^^^^^^^^^^^^^^^^ zero extended
//! ```

use brubeck::Immediate;

// Import test helpers
use crate::unit::test_helpers::values;

#[test]
fn test_always_sign_extend() {
    // Critical RISC-V behavior: ALL immediates are sign-extended
    // This is true even when using set_unsigned()
    let mut imm12 = Immediate::new(12);

    // Case 1: Small positive value (bit 11 = 0)
    imm12.set_unsigned(1).unwrap();
    assert_eq!(imm12.as_u32(), 1, "Positive values stay positive");
    assert_eq!(imm12.as_i32(), 1, "Same value signed or unsigned");

    // Case 2: All 12 bits set (bit 11 = 1)
    // Binary: 1111_1111_1111 (12 bits)
    imm12.set_unsigned(0xFFF).unwrap();
    assert_eq!(
        imm12.as_u32(),
        values::NEG_ONE,
        "0xFFF sign-extends to 0xFFFFFFFF"
    );
    assert_eq!(
        imm12.as_i32(),
        -1,
        "All ones represents -1 in two's complement"
    );

    // This sign extension is why SLTIU with -1 compares against 0xFFFFFFFF!
}

#[test]
fn test_min_max() {
    // Test 12-bit immediate boundaries
    // 12-bit signed range: -2048 to 2047
    let mut imm12 = Immediate::new(12);

    // Maximum positive value for signed 12-bit
    // Binary: 0111_1111_1111 (bit 11 = 0, rest = 1)
    assert!(imm12.set_signed(2047).is_ok());
    assert_eq!(imm12.as_i32(), 2047);
    assert_eq!(imm12.as_u32(), 2047, "Positive values unchanged");

    // Minimum negative value for signed 12-bit
    // Binary: 1000_0000_0000 (only bit 11 set)
    assert!(imm12.set_signed(-2048).is_ok());
    assert_eq!(imm12.as_i32(), -2048);
    assert_eq!(imm12.as_u32(), 0xFFFFF800, "Sign extends to 0xFFFFF800");

    // Out of range values should fail
    assert!(imm12.set_signed(2048).is_err(), "2048 > max (2047)");
    assert!(imm12.set_signed(-2049).is_err(), "-2049 < min (-2048)");
}

#[test]
fn test_set_signed() {
    // Setting signed values and observing sign extension
    let mut imm12 = Immediate::new(12);

    // Positive value - no sign extension needed
    let result = imm12.set_signed(42);
    assert!(result.is_ok());
    assert_eq!(imm12.as_i32(), 42);
    assert_eq!(imm12.as_u32(), 42, "Positive unchanged");

    // Negative value - requires sign extension
    let result = imm12.set_signed(-42);
    assert!(result.is_ok());
    assert_eq!(imm12.as_i32(), -42);
    assert_eq!(
        imm12.as_u32(),
        0xFFFFFFD6,
        "-42 = 0xFFFFFFD6 when sign-extended"
    );

    // Maximum negative for 12-bit
    let result = imm12.set_signed(-2048);
    assert!(result.is_ok());
    assert_eq!(imm12.as_i32(), -2048);
    assert_eq!(
        imm12.as_u32(),
        0xFFFFF800,
        "-2048 = 0xFFFFF800 (bit 11 propagated)"
    );
}

#[test]
fn test_get_signed() {
    // Demonstrates how bit 11 determines sign in 12-bit immediates
    let mut imm12 = Immediate::new(12);

    // Case 1: Bit 11 set = negative number
    // 0x800 = 1000_0000_0000 (only bit 11 set)
    imm12.set_unsigned(0x800).unwrap();
    assert_eq!(
        imm12.as_i32(),
        -2048,
        "Bit 11 set means negative in two's complement"
    );

    // Case 2: Bit 11 clear = positive number
    // 0x7FF = 0111_1111_1111 (all bits except 11)
    imm12.set_unsigned(0x7FF).unwrap();
    assert_eq!(imm12.as_i32(), 2047, "Bit 11 clear means positive");

    // This is why 12-bit signed range is -2048 to 2047
}

#[test]
fn test_get_unsigned() {
    // RISC-V quirk: as_u32() still sign-extends!
    // This catches many people off guard
    let mut imm12 = Immediate::new(12);

    // Small positive value - no surprises
    imm12.set_unsigned(42).unwrap();
    assert_eq!(imm12.as_u32(), 42, "Small values unchanged");

    // All 12 bits set - here's the surprise!
    imm12.set_unsigned(0xFFF).unwrap();
    assert_eq!(
        imm12.as_u32(),
        values::NEG_ONE,
        "0xFFF becomes 0xFFFFFFFF, not 0x00000FFF!"
    );

    // This is why ANDI with 0xFFF doesn't mask to 12 bits
    // You get ANDI rd, rs1, -1 instead!
}

#[test]
fn test_different_bit_widths() {
    // Different RISC-V instruction formats use different immediate widths

    // 5-bit immediate (shift amount in SLLI, SRLI, SRAI)
    let mut imm5 = Immediate::new(5);
    assert!(imm5.set_unsigned(31).is_ok(), "31 is max shift");
    assert!(imm5.set_unsigned(32).is_err(), "32 > 5-bit max");

    // Even 5-bit immediates are sign-extended!
    imm5.set_unsigned(31).unwrap(); // Binary: 11111
    assert_eq!(
        imm5.as_u32(),
        values::NEG_ONE,
        "Even 5-bit 0x1F sign-extends to 0xFFFFFFFF!"
    );
    assert_eq!(
        imm5.as_i32(),
        -1,
        "31 with bit 4 set = -1 when sign-extended"
    );

    // 20-bit immediate (LUI, AUIPC)
    let mut imm20 = Immediate::new(20);
    assert!(imm20.set_unsigned(values::IMM20_MAX).is_ok());
    assert!(imm20.set_unsigned(0x100000).is_err(), "Exceeds 20 bits");

    // Bit 19 determines sign for 20-bit immediates
    imm20.set_unsigned(0x80000).unwrap(); // Bit 19 set
    assert_eq!(
        imm20.as_i32(),
        -524288,
        "Bit 19 set = negative when sign-extended"
    );
    assert_eq!(
        imm20.as_u32(),
        0xFFF80000,
        "0x80000 sign-extends to 0xFFF80000"
    );
}

#[test]
fn test_bounds_checking() {
    // Verify bounds checking for different immediate sizes
    let mut imm12 = Immediate::new(12);

    // Unsigned bounds for 12-bit
    assert_eq!(imm12.unsigned_max(), 0xFFF, "2^12 - 1 = 4095");
    assert!(imm12.set_unsigned(0xFFF).is_ok(), "Max value OK");
    assert!(imm12.set_unsigned(0x1000).is_err(), "4096 > max");

    // Signed bounds for 12-bit
    assert_eq!(imm12.signed_max(), 2047, "2^11 - 1");
    assert_eq!(imm12.signed_min(), -2048, "-2^11");

    // Edge case: Can we set the exact min/max?
    assert!(imm12.set_signed(2047).is_ok(), "Max signed OK");
    assert!(imm12.set_signed(-2048).is_ok(), "Min signed OK");

    // Common mistake: thinking unsigned max is positive
    imm12.set_unsigned(0xFFF).unwrap();
    assert_eq!(
        imm12.as_i32(),
        -1,
        "Unsigned max 0xFFF is -1 when sign-extended!"
    );
}
