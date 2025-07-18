//! Integration tests for pseudo-instruction support in the interpreter
//!
//! These tests verify that pseudo-instructions are properly parsed,
//! expanded, and executed through the interpreter interface.

use brubeck::interpreter::Interpreter;
use brubeck::rv32_i::Register;

#[test]
fn test_mv_pseudo_instruction() {
    let mut interpreter = Interpreter::new();

    // Set up source value
    interpreter.interpret("ADDI x2, zero, 42").unwrap();

    // Execute MV pseudo-instruction
    let result = interpreter.interpret("MV x1, x2");
    assert!(result.is_ok(), "MV should execute successfully");

    // Verify the value was moved
    assert_eq!(
        interpreter.cpu.get_register(brubeck::rv32_i::Register::X1),
        42,
        "x1 should contain 42"
    );
}

#[test]
fn test_not_pseudo_instruction() {
    let mut interpreter = Interpreter::new();

    // Set up a value to NOT
    interpreter.interpret("ADDI x1, zero, 5").unwrap();

    // Execute NOT pseudo-instruction
    let result = interpreter.interpret("NOT x2, x1");
    assert!(result.is_ok(), "NOT should execute successfully");

    // Verify the result (NOT 5 = -6 in two's complement)
    // NOT 0x00000005 = 0xFFFFFFFA = -6 in signed interpretation = 4294967290 unsigned
    assert_eq!(
        interpreter.cpu.get_register(brubeck::rv32_i::Register::X2),
        0xfffffffa,
        "x2 should contain 0xfffffffa"
    );
}

#[test]
fn test_seqz_pseudo_instruction() {
    let mut interpreter = Interpreter::new();

    // Test with zero value
    interpreter.interpret("ADDI x1, zero, 0").unwrap();
    let result = interpreter.interpret("SEQZ x2, x1");
    assert!(result.is_ok(), "SEQZ should execute successfully");

    assert_eq!(
        interpreter.cpu.get_register(brubeck::rv32_i::Register::X2),
        0x00000001,
        "SEQZ of 0 should be 1"
    );

    // Test with non-zero value
    interpreter.interpret("ADDI x3, zero, 5").unwrap();
    interpreter.interpret("SEQZ x4, x3").unwrap();

    assert_eq!(
        interpreter.cpu.get_register(brubeck::rv32_i::Register::X4),
        0x00000000,
        "SEQZ of 5 should be 0"
    );
}

#[test]
fn test_snez_pseudo_instruction() {
    let mut interpreter = Interpreter::new();

    // Test with zero value
    interpreter.interpret("ADDI x1, zero, 0").unwrap();
    let result = interpreter.interpret("SNEZ x2, x1");
    assert!(result.is_ok(), "SNEZ should execute successfully");

    assert_eq!(
        interpreter.cpu.get_register(brubeck::rv32_i::Register::X2),
        0x00000000,
        "SNEZ of 0 should be 0"
    );

    // Test with non-zero value
    interpreter.interpret("ADDI x3, zero, 5").unwrap();
    interpreter.interpret("SNEZ x4, x3").unwrap();

    assert_eq!(
        interpreter.cpu.get_register(brubeck::rv32_i::Register::X4),
        0x00000001,
        "SNEZ of 5 should be 1"
    );
}

#[test]
fn test_j_pseudo_instruction() {
    let mut interpreter = Interpreter::new();

    // Execute J pseudo-instruction (unconditional jump)
    let result = interpreter.interpret("J 8");
    assert!(result.is_ok(), "J should execute successfully");

    // Check PC has jumped
    assert_eq!(interpreter.cpu.pc, 8, "PC should be 8 after J 8");
}

#[test]
fn test_jr_pseudo_instruction() {
    let mut interpreter = Interpreter::new();

    // Set up jump target
    interpreter.interpret("ADDI x1, zero, 100").unwrap();

    // Execute JR pseudo-instruction
    let result = interpreter.interpret("JR x1");
    assert!(result.is_ok(), "JR should execute successfully");

    // Check PC has jumped to register value
    assert_eq!(
        interpreter.cpu.pc, 100,
        "PC should be 100 (0x64) after JR x1"
    );
}

#[test]
fn test_ret_pseudo_instruction() {
    let mut interpreter = Interpreter::new();

    // Set up return address
    interpreter.interpret("ADDI ra, zero, 200").unwrap();

    // Execute RET pseudo-instruction
    let result = interpreter.interpret("RET");
    assert!(result.is_ok(), "RET should execute successfully");

    // Check PC has jumped to return address
    assert_eq!(interpreter.cpu.pc, 200, "PC should be 200 (0xc8) after RET");
}

#[test]
fn test_li_small_immediate() {
    let mut interpreter = Interpreter::new();

    // Test small immediate (fits in 12 bits)
    let result = interpreter.interpret("LI x1, 42");
    assert!(result.is_ok(), "LI with small immediate should work");

    assert_eq!(
        interpreter.cpu.get_register(brubeck::rv32_i::Register::X1),
        0x0000002a,
        "x1 should contain 42 (0x2a)"
    );
}

#[test]
fn test_li_large_immediate() {
    let mut interpreter = Interpreter::new();

    // Test large immediate (requires LUI + ADDI)
    let result = interpreter.interpret("LI x1, 0x12345");
    assert!(result.is_ok(), "LI with large immediate should work");

    assert_eq!(
        interpreter.cpu.get_register(brubeck::rv32_i::Register::X1),
        0x00012345,
        "x1 should contain 0x12345 (74565)"
    );
}

#[test]
fn test_li_negative_immediate() {
    let mut interpreter = Interpreter::new();

    // Test negative immediate
    let result = interpreter.interpret("LI x1, -1");
    assert!(result.is_ok(), "LI with -1 should work");

    assert_eq!(
        interpreter.cpu.get_register(brubeck::rv32_i::Register::X1),
        0xffffffff,
        "x1 should contain 0xffffffff (-1)"
    );
}

#[test]
fn test_pseudo_instruction_with_abi_names() {
    let mut interpreter = Interpreter::new();

    // Test pseudo-instructions with ABI register names
    interpreter.interpret("ADDI sp, zero, 1000").unwrap();
    interpreter.interpret("MV fp, sp").unwrap();

    assert_eq!(
        interpreter.cpu.get_register(brubeck::rv32_i::Register::X8),
        0x000003e8,
        "fp (x8) should contain 1000 (0x3e8)"
    );

    // Test RET which implicitly uses ra
    interpreter.interpret("LI ra, 0x2000").unwrap();
    interpreter.interpret("RET").unwrap();

    assert_eq!(
        interpreter.cpu.pc, 8192,
        "PC should be 0x2000 (8192) after RET"
    );
}

#[test]
fn test_pseudo_instruction_errors() {
    let mut interpreter = Interpreter::new();

    // Test invalid arguments
    let result = interpreter.interpret("MV x1");
    assert!(result.is_err(), "MV with missing argument should fail");

    let result = interpreter.interpret("NOT x1, x2, x3");
    assert!(result.is_err(), "NOT with too many arguments should fail");

    let result = interpreter.interpret("RET x1");
    assert!(result.is_err(), "RET with arguments should fail");

    // Test J with odd offset
    let result = interpreter.interpret("J 5");
    assert!(result.is_err(), "J with odd offset should fail");
}

#[test]
fn test_pseudo_instruction_expansion_visibility() {
    let mut interpreter = Interpreter::new();

    // The new API doesn't return instruction names, just state changes
    // We can verify the pseudo-instructions work by checking their effects

    // MV x1, x2 should copy x2 to x1
    interpreter.interpret("ADDI x2, x0, 42").unwrap(); // Set x2 = 42
    let delta = interpreter.interpret("MV x1, x2").unwrap();
    assert_ne!(delta.pc_change.0, delta.pc_change.1, "MV should change PC");
    assert_eq!(interpreter.cpu.get_register(Register::X1), 42);

    // NOT x3, x4 should invert x4 into x3
    interpreter.interpret("ADDI x4, x0, 0xFF").unwrap(); // Set x4 = 255
    let delta = interpreter.interpret("NOT x3, x4").unwrap();
    assert_ne!(delta.pc_change.0, delta.pc_change.1, "NOT should change PC");
    assert_eq!(interpreter.cpu.get_register(Register::X3) as i32, -256);

    // RET should jump to ra and increment PC
    interpreter.interpret("ADDI ra, x0, 0x100").unwrap(); // Set return address
    let delta = interpreter.interpret("RET").unwrap();
    assert_eq!(delta.pc_change.1, 0x100, "RET should jump to return address");
}
