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
fn test_negative_immediate_basic() {
    let mut i = Interpreter::new();
    
    // This SHOULD work but currently fails
    // -1 gets parsed as 4294967295 (u32 representation)
    // Then set_unsigned(4294967295) fails on 12-bit immediate
    i.interpret("ADDI x1, x0, -1").unwrap();
}

#[test]
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
    
    // Now that negatives work, we can use -1 directly
    assert!(i.interpret("ADDI x1, x0, -1").is_ok());
    let result = i.interpret("x1").unwrap();
    assert!(result.contains("0xffffffff")); // Should sign-extend to -1
    
    // The old workaround (4095) should NOT work anymore
    // because 4095 is outside the signed 12-bit range (-2048 to 2047)
    assert!(i.interpret("ADDI x1, x0, 4095").is_err());
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
fn test_load_offset_syntax() {
    let mut i = Interpreter::new();
    
    // Initialize some registers to valid memory addresses
    i.interpret("ADDI x2, x0, 1000").unwrap(); // x2 = 1000
    i.interpret("ADDI sp, x0, 2000").unwrap(); // sp = 2000
    
    // Standard RISC-V syntax: offset(base)
    i.interpret("LW x1, 0(x2)").unwrap();
    i.interpret("LW x1, 8(sp)").unwrap();
    i.interpret("LW x1, -4(x2)").unwrap(); // Negative offset
    
    // Test all load types
    i.interpret("LB x1, 100(x2)").unwrap();
    i.interpret("LH x1, 200(x2)").unwrap();
    i.interpret("LBU x1, -8(x2)").unwrap();
    i.interpret("LHU x1, 0(x0)").unwrap();
}

#[test]
fn test_store_offset_syntax() {
    let mut i = Interpreter::new();
    
    // Initialize registers
    i.interpret("ADDI x1, x0, 42").unwrap(); // x1 = 42 (value to store)
    i.interpret("ADDI x2, x0, 1000").unwrap(); // x2 = 1000 (base address)
    i.interpret("ADDI sp, x0, 2000").unwrap(); // sp = 2000
    
    // Standard RISC-V syntax for stores
    i.interpret("SW x1, 0(x2)").unwrap();
    i.interpret("SW x1, 100(sp)").unwrap();
    i.interpret("SB x1, -8(x2)").unwrap();
    i.interpret("SH x1, 256(x2)").unwrap();
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
    assert!(error.contains("Unknown instruction 'UNKNOWN'"));
    println!("Unknown instruction error: {}", error);
    
    // Test instruction suggestion
    let result2 = i.interpret("ADDD x1, x2, x3");
    assert!(result2.is_err());
    let error2 = result2.unwrap_err().to_string();
    println!("Suggestion error: {}", error2);
    assert!(error2.contains("Did you mean"));
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
    // ANDI uses signed immediates too, so -1 becomes 0xFFFFFFFF after sign extension
    assert!(i.interpret("ANDI x1, x0, -1").is_ok()); // -1 sign-extends to 0xFFFFFFFF
    
    // Now correctly rejects values outside signed 12-bit range
    assert!(i.interpret("ADDI x1, x0, 2048").is_err()); // Too large for signed 12-bit
    assert!(i.interpret("ADDI x1, x0, 4095").is_err()); // Too large for signed 12-bit
    
    // This correctly fails
    assert!(i.interpret("ANDI x1, x0, 2048").is_err()); // Too large for signed 12-bit
}

#[test]
fn test_immediate_boundaries_20bit() {
    let mut i = Interpreter::new();
    
    // U-Type instructions have 20-bit immediates
    assert!(i.interpret("LUI x1, 0").is_ok());
    assert!(i.interpret("LUI x1, 524287").is_ok()); // Max positive 20-bit signed
    assert!(i.interpret("LUI x1, -524288").is_ok()); // Min negative 20-bit signed
    assert!(i.interpret("LUI x1, 1048575").is_err()); // Too large for signed 20-bit
    assert!(i.interpret("LUI x1, 1048576").is_err()); // Too large
}

#[test]
fn test_shift_immediate_boundaries() {
    let mut i = Interpreter::new();
    
    // Shift immediates are 5-bit (0-31)
    assert!(i.interpret("SLLI x1, x2, 0").is_ok());
    assert!(i.interpret("SLLI x1, x2, 31").is_ok());
    
    // Shift immediates must be in range 0-31
    assert!(i.interpret("SLLI x1, x2, 32").is_err()); // Out of range
    assert!(i.interpret("SLLI x1, x2, 100").is_err()); // Out of range
    assert!(i.interpret("SRLI x1, x2, -1").is_err()); // Negative not allowed
    assert!(i.interpret("SRAI x1, x2, 50").is_err()); // Out of range
    
    // Verify error message is helpful
    let result = i.interpret("SLLI x1, x2, 100");
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("100"));
    assert!(error_msg.contains("0-31"));
    assert!(error_msg.contains("SLLI"));
    println!("SLLI error: {}", error_msg);
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
    
    // Initialize x1 with a valid aligned address for RET
    i.interpret("ADDI x1, x0, 100").unwrap(); // x1 = 100 (aligned)
    
    // RET takes no arguments
    assert!(i.interpret("RET").is_ok());
    
    assert!(i.interpret("J 100").is_ok());
    
    // Initialize x5 with a valid aligned address for JR
    i.interpret("ADDI x5, x0, 200").unwrap(); // x5 = 200 (aligned)
    
    // JR takes one register argument
    assert!(i.interpret("JR x5").is_ok());
    
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
    
    // Test which instructions correctly handle signed immediates
    // SLTI should accept signed immediates (-2048 to 2047)
    assert!(i.interpret("SLTI x1, x0, -2048").is_ok()); // Now works!
    assert!(i.interpret("SLTI x1, x0, 2047").is_ok());
    assert!(i.interpret("SLTI x1, x0, 2048").is_err()); // Correctly fails, too large for signed
    
    // SLTIU also uses sign-extended immediates (then treats as unsigned for comparison)
    // So it still only accepts -2048 to 2047 in the immediate field
    assert!(i.interpret("SLTIU x1, x0, 2047").is_ok());
    assert!(i.interpret("SLTIU x1, x0, 4095").is_err()); // Too large for signed 12-bit
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
    
    // Now these fail because 4095 is outside signed 12-bit range
    assert!(i.interpret("ANDI x1, x0, 4095").is_err()); // Too large for signed 12-bit
    assert!(i.interpret("ORI x1, x0, 4095").is_err());
    assert!(i.interpret("XORI x1, x0, 4095").is_err());
    
    // Now let's verify that -1 works correctly (which is what 4095 represented)
    i.interpret("XORI x1, x0, -1").unwrap();
    i.interpret("XORI x2, x0, 1").unwrap();
    i.interpret("XOR x3, x1, x2").unwrap(); // Result should be -1 ^ 1 = -2
    
    let result = i.interpret("x3").unwrap();
    
    // XORI with -1 sign-extends to 0xffffffff
    // XOR with 1 gives 0xfffffffe (-2)
    assert!(result.contains("0xfffffffe")); // -2 in hex
}

#[test]
fn test_special_register_behaviors() {
    let mut i = Interpreter::new();
    
    // PC register can be read
    assert!(i.interpret("PC").is_ok());
    
    // PC cannot be used in regular instructions
    let err = i.interpret("ADD x1, PC, x0").unwrap_err();
    assert!(err.to_string().contains("PC register cannot be used"));
    assert!(err.to_string().contains("source 1"));
    
    // Test various instructions that should reject PC
    assert!(i.interpret("ADD PC, x1, x2").is_err()); // PC as destination
    assert!(i.interpret("ADDI PC, x0, 5").is_err()); // PC in I-type
    assert!(i.interpret("LW PC, 0(x1)").is_err()); // PC in load
    assert!(i.interpret("SW PC, 0(x1)").is_err()); // PC in store
    assert!(i.interpret("BEQ PC, x1, 8").is_err()); // PC in branch
    
    // AUIPC reads PC implicitly but shouldn't allow PC as destination
    assert!(i.interpret("AUIPC PC, 0").is_err());
    
    // JAL updates PC implicitly but shouldn't allow PC as link register
    assert!(i.interpret("JAL PC, 100").is_err());
    
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
    assert!(err.contains("Unknown instruction"));
    
    // Wrong argument count
    let err = i.interpret("ADD x1, x2").unwrap_err().to_string();
    assert!(err.contains("Invalid RType arguments"));
    // Note: Error doesn't say HOW MANY arguments are needed
    
    // Invalid register
    let err = i.interpret("ADD x1, x2, x99").unwrap_err().to_string();
    assert!(err.contains("X99"));
    assert!(err.contains("Unknown instruction")); // X99 is parsed as unknown instruction
    
    // Out of range immediate
    let err = i.interpret("ADDI x1, x0, 5000").unwrap_err().to_string();
    assert!(err.contains("5000"));
    assert!(err.contains("out of range") || err.contains("too big"));
    assert!(err.contains("ADDI"));
    assert!(err.contains("-2048 to 2047"));
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
    
    // 4095 is outside the signed 12-bit range, so this should fail
    assert!(i.interpret("JALR x1, x2, 4095").is_err()); // Too large for signed 12-bit
    
    // Branch instructions
    assert!(i.interpret("BEQ x1, x2, 0").is_ok());
    // Branch immediates are also signed 12-bit, but encoded as multiples of 2
    // So the range is -4096 to 4094 (in steps of 2)
    assert!(i.interpret("BEQ x1, x2, 4094").is_err()); // Too large for signed 12-bit
    assert!(i.interpret("BEQ x1, x2, 2046").is_ok()); // Max positive even value in signed range
    
    // Upper immediate instructions
    assert!(i.interpret("LUI x1, 0").is_ok());
    assert!(i.interpret("LUI x1, 524287").is_ok()); // Max positive 20-bit signed
    assert!(i.interpret("AUIPC x1, 524287").is_ok());
}