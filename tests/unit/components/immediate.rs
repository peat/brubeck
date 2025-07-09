//! Unit tests for immediate value handling
//!
//! These tests verify sign extension, bounds checking, and the behavior
//! of immediate values used throughout the RISC-V ISA.

use brubeck::Immediate;

#[test]
fn test_always_sign_extend() {
    // Test that unsigned values are sign-extended when retrieved as signed
    let mut imm12 = Immediate::new(12);
    
    // Small positive value
    imm12.set_unsigned(1).unwrap();
    assert_eq!(imm12.as_u32(), 1);
    assert_eq!(imm12.as_i32(), 1);
    
    // Value with bit 11 set (would be negative if sign-extended)
    imm12.set_unsigned(0b1111_1111_1111).unwrap(); // All 12 bits set
    assert_eq!(imm12.as_u32(), 0xFFFF_FFFF); // Sign-extended
    assert_eq!(imm12.as_i32(), -1); // Interpreted as -1
}

#[test]
fn test_min_max() {
    // Test 12-bit immediate boundaries
    let mut imm12 = Immediate::new(12);
    
    // Maximum positive value for signed 12-bit: 2047 (0x7FF)
    assert!(imm12.set_signed(2047).is_ok());
    assert_eq!(imm12.as_i32(), 2047);
    
    // Minimum negative value for signed 12-bit: -2048 (0x800)
    assert!(imm12.set_signed(-2048).is_ok());
    assert_eq!(imm12.as_i32(), -2048);
    
    // Out of range values should fail
    assert!(imm12.set_signed(2048).is_err());
    assert!(imm12.set_signed(-2049).is_err());
}

#[test]
fn test_set_signed() {
    let mut imm12 = Immediate::new(12);
    
    // Positive value
    let result = imm12.set_signed(42);
    assert!(result.is_ok());
    assert_eq!(imm12.as_i32(), 42);
    assert_eq!(imm12.as_u32(), 42);
    
    // Negative value
    let result = imm12.set_signed(-42);
    assert!(result.is_ok());
    assert_eq!(imm12.as_i32(), -42);
    assert_eq!(imm12.as_u32() as i32, -42);
    
    // Maximum negative
    let result = imm12.set_signed(-2048);
    assert!(result.is_ok());
    assert_eq!(imm12.as_i32(), -2048);
}

#[test]
fn test_get_signed() {
    // Test retrieving values as signed
    let mut imm12 = Immediate::new(12);
    
    // Set a value that has bit 11 set
    imm12.set_unsigned(0x800).unwrap(); // Bit 11 set
    assert_eq!(imm12.as_i32(), -2048); // Should be negative when interpreted as signed
    
    // Set a value with bit 11 clear
    imm12.set_unsigned(0x7FF).unwrap(); // Bit 11 clear
    assert_eq!(imm12.as_i32(), 2047); // Should be positive
}

#[test]
fn test_get_unsigned() {
    // Test retrieving values as unsigned (always sign-extended to 32 bits)
    let mut imm12 = Immediate::new(12);
    
    // Small positive value
    imm12.set_unsigned(42).unwrap();
    assert_eq!(imm12.as_u32(), 42);
    
    // Value with high bit set
    imm12.set_unsigned(0xFFF).unwrap(); // All 12 bits set
    assert_eq!(imm12.as_u32(), 0xFFFF_FFFF); // Sign-extended to 32 bits
}

#[test]
fn test_different_bit_widths() {
    // Test immediates of different sizes
    
    // 5-bit immediate (used in shift instructions)
    let mut imm5 = Immediate::new(5);
    assert!(imm5.set_unsigned(31).is_ok());
    assert!(imm5.set_unsigned(32).is_err()); // Too large for 5 bits
    
    // 20-bit immediate (used in U-type instructions)
    let mut imm20 = Immediate::new(20);
    assert!(imm20.set_unsigned(0xFFFFF).is_ok()); // Maximum 20-bit value
    assert!(imm20.set_unsigned(0x100000).is_err()); // Too large for 20 bits
    
    // When retrieved, should be sign-extended
    imm20.set_unsigned(0x80000).unwrap(); // Bit 19 set
    assert_eq!(imm20.as_i32(), -524288); // Negative when sign-extended
}

#[test]
fn test_bounds_checking() {
    let mut imm12 = Immediate::new(12);
    
    // Test unsigned bounds
    assert_eq!(imm12.unsigned_max(), 0xFFF); // 12 bits all set
    assert!(imm12.set_unsigned(0xFFF).is_ok());
    assert!(imm12.set_unsigned(0x1000).is_err());
    
    // Test signed bounds
    assert_eq!(imm12.signed_max(), 2047); // 0x7FF
    assert_eq!(imm12.signed_min(), -2048); // 0x800 as signed
}