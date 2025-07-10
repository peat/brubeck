//! Common test values with semantic meaning
//!
//! These constants are useful across both unit and integration tests
//! for creating readable, maintainable test cases.

#![allow(dead_code)]

// ==================== BASIC VALUES ====================

pub const ZERO: u32 = 0;
pub const ONE: u32 = 1;
pub const TWO: u32 = 2;
pub const TEN: u32 = 10;

// ==================== BOUNDARY VALUES ====================

pub const U32_MAX: u32 = u32::MAX;
pub const U32_MIN: u32 = u32::MIN;
pub const I32_MAX: u32 = i32::MAX as u32;
pub const I32_MIN: u32 = i32::MIN as u32;

// ==================== SIGN-RELATED VALUES ====================

pub const NEG_ONE: u32 = -1_i32 as u32;
pub const NEG_TWO: u32 = -2_i32 as u32;
pub const NEG_TEN: u32 = -10_i32 as u32;
pub const MSB_SET: u32 = 0x8000_0000; // Most significant bit set
pub const MSB_CLEAR: u32 = 0x7FFF_FFFF; // Most significant bit clear

// ==================== BIT PATTERNS ====================

pub const ALL_ONES: u32 = 0xFFFF_FFFF;
pub const ALL_ZEROS: u32 = 0x0000_0000;
pub const ALTERNATING_BITS: u32 = 0xAAAA_AAAA;
pub const ALTERNATING_BITS_INV: u32 = 0x5555_5555;
pub const BYTE_FF: u32 = 0xFF;
pub const BYTE_0F: u32 = 0x0F;
pub const BYTE_F0: u32 = 0xF0;

// ==================== MEMORY ADDRESSES ====================

pub const STACK_BASE: u32 = 0x8000_0000;
pub const HEAP_BASE: u32 = 0x1000_0000;
pub const CODE_BASE: u32 = 0x0000_1000;
pub const DATA_BASE: u32 = 0x0001_0000;
pub const TEST_ADDR: u32 = 0x100; // Common test address
pub const TEST_ADDR_2: u32 = 0x200; // Secondary test address
pub const TEST_ADDR_ALIGNED: u32 = 0x400; // 4-byte aligned address

// ==================== IMMEDIATE BOUNDARIES ====================

pub const IMM12_MAX: i32 = 2047; // Maximum 12-bit signed immediate
pub const IMM12_MIN: i32 = -2048; // Minimum 12-bit signed immediate
pub const IMM12_MAX_U: u32 = 2047; // As unsigned for comparisons
pub const IMM12_MIN_U: u32 = -2048_i32 as u32; // As unsigned

pub const IMM20_MAX: u32 = 0xFFFFF; // Maximum 20-bit unsigned immediate
pub const IMM5_MAX: u32 = 31; // Maximum 5-bit immediate (CSR immediate)

// ==================== SHIFT AMOUNTS ====================

pub const SHIFT_0: u32 = 0;
pub const SHIFT_1: u32 = 1;
pub const SHIFT_8: u32 = 8;
pub const SHIFT_16: u32 = 16;
pub const SHIFT_31: u32 = 31;
pub const SHIFT_32: u32 = 32; // Tests 5-bit masking

// ==================== COMMON TEST VALUES ====================

pub const SMALL_POS: u32 = 42;
pub const SMALL_NEG: u32 = -42_i32 as u32;
pub const MEDIUM_VAL: u32 = 1000;
pub const LARGE_VAL: u32 = 0x12345678;

// ==================== CSR ADDRESSES ====================

pub const CSR_CYCLE: u16 = 0xC00; // Read-only cycle counter
pub const CSR_TIME: u16 = 0xC01; // Read-only timer
pub const CSR_INSTRET: u16 = 0xC02; // Read-only instructions retired
pub const CSR_MSCRATCH: u16 = 0x340; // Read-write scratch register

// ==================== HELPER FUNCTIONS ====================

/// Convert signed value to unsigned for register storage
pub const fn as_u32(val: i32) -> u32 {
    val as u32
}

/// Check if a value fits in a 12-bit immediate
pub const fn fits_in_imm12(val: i32) -> bool {
    val >= IMM12_MIN && val <= IMM12_MAX
}

/// Create a test bit pattern
pub const fn test_pattern(byte: u8) -> u32 {
    let b = byte as u32;
    (b << 24) | (b << 16) | (b << 8) | b
}
