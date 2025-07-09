//! Comprehensive parser test suite
//!
//! This test suite documents both current parser behavior and desired improvements.
//! Tests are organized by feature area and clearly marked as either:
//! - Working (current behavior we want to preserve)
//! - Broken (current bugs to fix)
//! - TODO (new features to implement)

use brubeck::interpreter::Interpreter;

// =============================================================================
// BASIC PARSING - These should always work
// =============================================================================

#[test]
fn test_basic_instruction_parsing() {
    let mut i = Interpreter::new();
    
    // R-Type: ADD rd, rs1, rs2
    assert!(i.interpret("ADD x1, x2, x3").is_ok());
    assert!(i.interpret("SUB x4, x5, x6").is_ok());
    assert!(i.interpret("AND x7, x8, x9").is_ok());
    assert!(i.interpret("OR x10, x11, x12").is_ok());
    assert!(i.interpret("XOR x13, x14, x15").is_ok());
    
    // I-Type with positive immediates
    assert!(i.interpret("ADDI x1, x0, 42").is_ok());
    assert!(i.interpret("SLTI x1, x2, 100").is_ok());
    assert!(i.interpret("ANDI x1, x2, 255").is_ok());
    
    // Shifts (special I-Type with 5-bit immediate)
    assert!(i.interpret("SLLI x1, x2, 5").is_ok());
    assert!(i.interpret("SRLI x1, x2, 10").is_ok());
    assert!(i.interpret("SRAI x1, x2, 3").is_ok());
}

#[test]
fn test_register_name_variations() {
    let mut i = Interpreter::new();
    
    // Numeric registers
    assert!(i.interpret("ADD x0, x1, x31").is_ok());
    
    // ABI names
    assert!(i.interpret("ADD zero, ra, t6").is_ok());
    assert!(i.interpret("ADD sp, gp, tp").is_ok());
    assert!(i.interpret("ADD a0, a7, s11").is_ok());
    
    // Mixed case (should work due to normalization)
    assert!(i.interpret("add X1, x2, X3").is_ok());
    assert!(i.interpret("AdD Zero, RA, SP").is_ok());
}

#[test]
fn test_whitespace_handling() {
    let mut i = Interpreter::new();
    
    // Extra spaces
    assert!(i.interpret("  ADD    x1,    x2,    x3  ").is_ok());
    
    // No spaces
    assert!(i.interpret("ADD x1,x2,x3").is_ok());
    
    // Mixed spacing
    assert!(i.interpret("ADD x1, x2 ,x3").is_ok());
    
    // Tab characters
    assert!(i.interpret("ADD\tx1,\tx2,\tx3").is_ok());
}

// =============================================================================
// NEGATIVE IMMEDIATES - Currently broken, need fixing
// =============================================================================

#[test]
#[should_panic(expected = "Generic")] // Documents current broken behavior
fn test_negative_immediate_basic() {
    let mut i = Interpreter::new();
    
    // This SHOULD work but currently fails
    // -1 gets parsed as 4294967295 (u32 representation)
    // Then set_unsigned(4294967295) fails on 12-bit immediate
    i.interpret("ADDI x1, x0, -1").unwrap();
}

#[test]
#[should_panic(expected = "Generic")]
fn test_negative_immediate_range() {
    let mut i = Interpreter::new();
    
    // These should all work for 12-bit signed immediates
    i.interpret("ADDI x1, x0, -2048").unwrap(); // Min 12-bit signed
    i.interpret("ADDI x1, x0, -1000").unwrap();
    i.interpret("ADDI x1, x0, -42").unwrap();
}

#[test]
fn test_negative_immediate_workarounds() {
    let mut i = Interpreter::new();
    
    // Current workaround: use positive representation of two's complement
    // -1 in 12-bit two's complement = 4095
    assert!(i.interpret("ADDI x1, x0, 4095").is_ok());
    let result = i.interpret("x1").unwrap();
    assert!(result.contains("0xffffffff")); // Should sign-extend to -1
}

// =============================================================================
// HEX AND BINARY IMMEDIATES - Surprisingly working!
// =============================================================================

#[test]
fn test_hex_immediate_lowercase() {
    let mut i = Interpreter::new();
    
    // BUG: This actually works! But only because normalize uppercases to 0XFF
    // and parse_value checks for both "0x" and "0X"
    i.interpret("ADDI x1, x0, 0xff").unwrap();
    let result = i.interpret("x1").unwrap();
    assert!(result.contains("0xff")); // Should be 255
}

#[test]
fn test_hex_immediate_uppercase() {
    let mut i = Interpreter::new();
    
    // This works because it matches the uppercase check
    i.interpret("ADDI x1, x0, 0xFF").unwrap();
    let result = i.interpret("x1").unwrap();
    assert!(result.contains("0xff"));
}

#[test]
fn test_hex_immediate_prefix_variations() {
    let mut i = Interpreter::new();
    
    // Both work due to the dual check
    i.interpret("ADDI x1, x0, 0x100").unwrap();
    i.interpret("ADDI x2, x0, 0X200").unwrap();
    
    let r1 = i.interpret("x1").unwrap();
    let r2 = i.interpret("x2").unwrap();
    assert!(r1.contains("0x100"));
    assert!(r2.contains("0x200"));
}

#[test]
fn test_binary_immediate() {
    let mut i = Interpreter::new();
    
    // Binary literals also work!
    i.interpret("ADDI x1, x0, 0b1010").unwrap(); // 10 in decimal
    i.interpret("ADDI x2, x0, 0B1111").unwrap(); // 15 in decimal
    
    let r1 = i.interpret("x1").unwrap();
    let r2 = i.interpret("x2").unwrap();
    assert!(r1.contains("0xa")); // 10 in hex
    assert!(r2.contains("0xf")); // 15 in hex
}

// =============================================================================
// LOAD/STORE SYNTAX - Standard RISC-V offset(register) notation
// =============================================================================

#[test]
#[should_panic] // TODO: Implement standard syntax
fn test_load_offset_syntax() {
    let mut i = Interpreter::new();
    
    // Standard RISC-V syntax: offset(base)
    i.interpret("LW x1, 0(x2)").unwrap();
    i.interpret("LW x1, 8(sp)").unwrap();
    i.interpret("LW x1, -4(x2)").unwrap(); // Negative offset
}

#[test]
#[should_panic] // TODO: Implement standard syntax
fn test_store_offset_syntax() {
    let mut i = Interpreter::new();
    
    // Standard RISC-V syntax for stores
    i.interpret("SW x1, 0(x2)").unwrap();
    i.interpret("SW x1, 100(sp)").unwrap();
    i.interpret("SB x1, -8(x2)").unwrap();
}

#[test]
fn test_current_load_store_syntax() {
    let mut i = Interpreter::new();
    
    // Current non-standard syntax (works but not ideal)
    assert!(i.interpret("LW x1, x2, 0").is_ok());
    assert!(i.interpret("SW x1, x2, 0").is_ok());
}

// =============================================================================
// ERROR HANDLING - Test quality of error messages
// =============================================================================

#[test]
fn test_error_unknown_instruction() {
    let mut i = Interpreter::new();
    
    let result = i.interpret("UNKNOWN x1, x2, x3");
    assert!(result.is_err());
    
    let error = result.unwrap_err().to_string();
    assert!(error.contains("Unrecognized token: 'UNKNOWN'"));
    
    // TODO: Better error should suggest similar instructions
    // e.g., "Unknown instruction 'UNKNOWN'. Did you mean 'AND'?"
}

#[test]
fn test_error_invalid_register() {
    let mut i = Interpreter::new();
    
    let result = i.interpret("ADD x1, x2, x99");
    assert!(result.is_err());
    
    let error = result.unwrap_err().to_string();
    assert!(error.contains("X99")); // Current error mentions the bad token
    
    // TODO: Better error should be:
    // "Invalid register 'x99'. Valid registers are x0-x31 or ABI names (zero, ra, sp, etc.)"
}

#[test]
fn test_error_wrong_argument_count() {
    let mut i = Interpreter::new();
    
    let result = i.interpret("ADD x1, x2"); // Missing rs2
    assert!(result.is_err());
    
    // TODO: Better error should be:
    // "ADD requires 3 arguments (rd, rs1, rs2), but only 2 were provided"
}

#[test]
fn test_error_immediate_out_of_range() {
    let mut i = Interpreter::new();
    
    // 12-bit immediate max is 4095 (unsigned) or 2047 (signed positive)
    let result = i.interpret("ADDI x1, x0, 5000");
    assert!(result.is_err());
    
    // TODO: Better error should be:
    // "Immediate value 5000 out of range for ADDI (12-bit signed: -2048 to 2047)"
}

// =============================================================================
// EDGE CASES AND BOUNDARIES
// =============================================================================

#[test]
fn test_immediate_boundaries_12bit() {
    let mut i = Interpreter::new();
    
    // Positive boundaries (currently working)
    assert!(i.interpret("ADDI x1, x0, 0").is_ok());
    assert!(i.interpret("ADDI x1, x0, 2047").is_ok()); // Max positive 12-bit signed
    assert!(i.interpret("ANDI x1, x0, 4095").is_ok()); // Max 12-bit unsigned
    
    // BUG: Parser uses set_unsigned, so 2048 is accepted even though it's too large for signed!
    // This is wrong - ADDI should only accept -2048 to 2047
    assert!(i.interpret("ADDI x1, x0, 2048").is_ok()); // BROKEN: Should fail!
    assert!(i.interpret("ADDI x1, x0, 4095").is_ok()); // BROKEN: Accepts full unsigned range
    
    // This correctly fails
    assert!(i.interpret("ANDI x1, x0, 4096").is_err()); // Too large for unsigned
}

#[test]
fn test_immediate_boundaries_20bit() {
    let mut i = Interpreter::new();
    
    // U-Type instructions have 20-bit immediates
    assert!(i.interpret("LUI x1, 0").is_ok());
    assert!(i.interpret("LUI x1, 1048575").is_ok()); // Max 20-bit
    assert!(i.interpret("LUI x1, 1048576").is_err()); // Too large
}

#[test]
fn test_shift_immediate_boundaries() {
    let mut i = Interpreter::new();
    
    // Shift immediates are 5-bit (0-31)
    assert!(i.interpret("SLLI x1, x2, 0").is_ok());
    assert!(i.interpret("SLLI x1, x2, 31").is_ok());
    
    // BUG: Parser accepts any value that fits in 12 bits!
    // Shifts should only accept 0-31 (5 bits)
    assert!(i.interpret("SLLI x1, x2, 32").is_ok()); // BROKEN: Should fail!
    assert!(i.interpret("SLLI x1, x2, 100").is_ok()); // BROKEN: Should fail!
}

// =============================================================================
// SPECIAL CASES AND CORNER CASES
// =============================================================================

#[test]
fn test_empty_input() {
    let mut i = Interpreter::new();
    
    assert!(i.interpret("").is_err());
    assert!(i.interpret("   ").is_err()); // Just whitespace
    assert!(i.interpret("\t\n").is_err()); // Just whitespace
}

#[test]
fn test_partial_instructions() {
    let mut i = Interpreter::new();
    
    // Missing all arguments
    assert!(i.interpret("ADD").is_err());
    
    // Missing some arguments
    assert!(i.interpret("ADD x1").is_err());
    assert!(i.interpret("ADD x1, x2").is_err());
    
    // Extra arguments (should also fail)
    assert!(i.interpret("ADD x1, x2, x3, x4").is_err());
}

#[test]
fn test_register_x0_behavior() {
    let mut i = Interpreter::new();
    
    // x0/zero register should always read as 0
    i.interpret("ADDI x0, x0, 100").unwrap(); // Try to write to x0
    let result = i.interpret("x0").unwrap();
    assert!(result.contains("0x0")); // Should still be 0
}

// =============================================================================
// FUTURE FEATURES - These document desired functionality
// =============================================================================

#[test]
#[should_panic] // TODO: Implement
fn test_label_support() {
    let mut i = Interpreter::new();
    
    // Define a label and jump to it
    i.interpret("start:").unwrap();
    i.interpret("ADDI x1, x1, 1").unwrap();
    i.interpret("J start").unwrap(); // Jump to label
}

#[test]
#[should_panic] // TODO: Implement
fn test_expression_evaluation() {
    let mut i = Interpreter::new();
    
    // Simple arithmetic in immediates
    i.interpret("ADDI x1, x0, 10 + 5").unwrap(); // Should load 15
    i.interpret("ADDI x2, x0, 100 - 1").unwrap(); // Should load 99
    i.interpret("LUI x3, 0x1000 >> 12").unwrap(); // Shift expression
}

#[test]
#[should_panic] // TODO: Implement
fn test_character_literals() {
    let mut i = Interpreter::new();
    
    // ASCII character literals
    i.interpret("ADDI x1, x0, 'A'").unwrap(); // Should load 65
    i.interpret("ADDI x2, x0, '\\n'").unwrap(); // Should load 10
}

#[test]
#[should_panic] // TODO: Implement
fn test_multi_line_input() {
    let mut i = Interpreter::new();
    
    // Execute multiple instructions at once
    let program = r#"
        ADDI x1, x0, 10
        ADDI x2, x0, 20
        ADD x3, x1, x2
    "#;
    
    i.interpret(program).unwrap();
    let result = i.interpret("x3").unwrap();
    assert!(result.contains("30"));
}

// =============================================================================
// PSEUDO-INSTRUCTION PARSING
// =============================================================================

#[test]
fn test_pseudo_instruction_parsing() {
    let mut i = Interpreter::new();
    
    // Basic pseudo-instructions (should work)
    assert!(i.interpret("MV x1, x2").is_ok());
    assert!(i.interpret("NOT x1, x2").is_ok());
    
    // BUG: These pseudo-instructions require specific argument patterns
    // RET takes no arguments but parser seems to expect some
    let ret_result = i.interpret("RET");
    assert!(ret_result.is_err()); // Currently broken!
    
    assert!(i.interpret("J 100").is_ok());
    
    // JR pseudo-instruction has a bug - let's investigate
    let jr_result = i.interpret("JR x1");
    assert!(jr_result.is_err()); // Currently fails!
    
    assert!(i.interpret("LI x1, 42").is_ok());
}

#[test]
fn test_pseudo_li_negative() {
    let mut i = Interpreter::new();
    
    // LI pseudo-instruction handles negative values correctly
    // It uses set_signed() for small immediates
    let result = i.interpret("LI x1, -1");
    assert!(result.is_ok());
    
    let x1_result = i.interpret("x1").unwrap();
    assert!(x1_result.contains("0xffffffff")); // -1 in hex
    
    // Test more negative values
    i.interpret("LI x2, -2048").unwrap(); // Min 12-bit signed
    let x2_result = i.interpret("x2").unwrap();
    assert!(x2_result.contains("0xfffff800")); // -2048 in hex
    
    i.interpret("LI x3, -42").unwrap();
    let x3_result = i.interpret("x3").unwrap();
    assert!(x3_result.contains("0xffffffd6")); // -42 in hex
}

#[test]
fn test_pseudo_instruction_case_insensitive() {
    let mut i = Interpreter::new();
    
    // Should work with different cases
    assert!(i.interpret("mv x1, x2").is_ok());
    assert!(i.interpret("Mv x1, x2").is_ok());
    assert!(i.interpret("MV x1, x2").is_ok());
}

// =============================================================================
// ADDITIONAL PARSER BEHAVIOR TESTS
// =============================================================================

#[test]
fn test_signed_vs_unsigned_confusion() {
    let mut i = Interpreter::new();
    
    // Test which instructions incorrectly accept out-of-range signed values
    // SLTI should accept signed immediates (-2048 to 2047)
    assert!(i.interpret("SLTI x1, x0, -2048").is_err()); // Broken: can't parse negative
    assert!(i.interpret("SLTI x1, x0, 2047").is_ok());
    assert!(i.interpret("SLTI x1, x0, 2048").is_ok()); // BUG: Should fail, too large for signed
    
    // SLTIU should accept unsigned immediates (0 to 4095 when treated as unsigned)
    // but the immediate is still sign-extended from 12 bits
    assert!(i.interpret("SLTIU x1, x0, 4095").is_ok());
    assert!(i.interpret("SLTIU x1, x0, 4096").is_err()); // Correctly fails
}

#[test]
fn test_parser_tokenization_details() {
    let mut i = Interpreter::new();
    
    // Test weird spacing
    assert!(i.interpret("ADD    x1    ,    x2    ,    x3").is_ok());
    assert!(i.interpret("ADD\t\tx1,\t\tx2,\t\tx3").is_ok());
    
    // Test that commas are optional (they're treated as separators)
    assert!(i.interpret("ADD x1 x2 x3").is_ok());
    
    // Multiple commas should work (empty tokens are filtered)
    assert!(i.interpret("ADD x1,, x2,,, x3").is_ok());
}

#[test]
fn test_immediate_representation_bugs() {
    let mut i = Interpreter::new();
    
    // Document the current behavior with large unsigned values
    // These work because they fit in 12 bits unsigned
    assert!(i.interpret("ANDI x1, x0, 4095").is_ok()); // Max 12-bit unsigned
    assert!(i.interpret("ORI x1, x0, 4095").is_ok());
    assert!(i.interpret("XORI x1, x0, 4095").is_ok());
    
    // But logically, XORI with 4095 is the same as XORI with -1
    // Let's verify the behavior
    i.interpret("XORI x1, x0, 4095").unwrap();
    i.interpret("XORI x2, x0, 1").unwrap();
    i.interpret("XOR x3, x1, x2").unwrap(); // Result should be -1 ^ 1 = -2
    
    let result = i.interpret("x3").unwrap();
    
    // XORI with 4095 sign-extends to 0xffffffff (-1)
    // XOR with 1 gives 0xfffffffe (-2)
    assert!(result.contains("0xfffffffe")); // -2 in hex
}

#[test]
fn test_special_register_behaviors() {
    let mut i = Interpreter::new();
    
    // PC register can be read
    assert!(i.interpret("PC").is_ok());
    
    // But can we use PC in instructions? 
    // BUG: PC is tokenized as a register and accepted in instructions!
    assert!(i.interpret("ADD x1, PC, x0").is_ok()); // This shouldn't work but does!
    
    // x0 always reads as zero
    i.interpret("ADDI x0, x0, 100").unwrap();
    let result = i.interpret("x0").unwrap();
    assert!(result.contains(": 0 "));
}

#[test]
fn test_error_message_quality() {
    let mut i = Interpreter::new();
    
    // Unknown instruction
    let err = i.interpret("MULH x1, x2, x3").unwrap_err().to_string();
    assert!(err.contains("MULH"));
    assert!(err.contains("Unrecognized"));
    
    // Wrong argument count
    let err = i.interpret("ADD x1, x2").unwrap_err().to_string();
    assert!(err.contains("Invalid RType arguments"));
    // Note: Error doesn't say HOW MANY arguments are needed
    
    // Invalid register
    let err = i.interpret("ADD x1, x2, x99").unwrap_err().to_string();
    assert!(err.contains("X99"));
    assert!(err.contains("Unrecognized"));
    
    // Out of range immediate
    let err = i.interpret("ADDI x1, x0, 5000").unwrap_err().to_string();
    assert!(err.contains("5000"));
    assert!(err.contains("too big"));
}

#[test]
fn test_instruction_variants_coverage() {
    let mut i = Interpreter::new();
    
    // Ensure we test all the different immediate handling paths
    // Some instructions might have special handling
    
    // JALR has offset immediate
    // First set x2 to a valid aligned address
    i.interpret("ADDI x2, x0, 0").unwrap(); // x2 = 0
    assert!(i.interpret("JALR x1, x2, 0").is_ok());
    
    // For JALR with non-zero offset, we need the result to be aligned
    // JALR jumps to (rs1 + imm) & ~1, so we need (x2 + imm) to be 4-byte aligned
    // Set x2 to 1 so that x2 + 2047 = 2048 which is 4-byte aligned
    i.interpret("ADDI x2, x0, 1").unwrap();
    assert!(i.interpret("JALR x1, x2, 2047").is_ok());
    
    // For 4095: we need x2 + 4095 to be aligned. Set x2 = 1 so result is 4096
    i.interpret("ADDI x2, x0, 1").unwrap();
    assert!(i.interpret("JALR x1, x2, 4095").is_ok()); // BROKEN: Should fail for signed immediate!
    
    // Branch instructions
    assert!(i.interpret("BEQ x1, x2, 0").is_ok());
    assert!(i.interpret("BEQ x1, x2, 4094").is_ok()); // Max even 12-bit
    
    // Upper immediate instructions
    assert!(i.interpret("LUI x1, 0").is_ok());
    assert!(i.interpret("LUI x1, 1048575").is_ok()); // Max 20-bit
    assert!(i.interpret("AUIPC x1, 1048575").is_ok());
}