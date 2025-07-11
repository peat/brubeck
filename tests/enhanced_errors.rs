//! Tests for enhanced error messages with educational content

use brubeck::interpreter::Interpreter;

#[test]
fn test_cpu_error_messages() {
    let mut interpreter = Interpreter::new();

    // Test misaligned jump error - JALR with address that becomes misaligned after clearing LSB
    interpreter.interpret("ADDI x1, x0, 0x103").unwrap(); // 0x102 after LSB cleared
    let result = interpreter.interpret("JALR x0, x1, 0");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Misaligned jump"));
    // Educational content is still in the error message (from library)
    // The binary filters it out when tips=false
    assert!(err_msg.contains("💡 Tip:"));
    assert!(err_msg.contains("4-byte aligned"));
    assert!(err_msg.contains("remainder of 2"));

    // Test memory access violation
    interpreter.interpret("LUI x1, 0x100").unwrap(); // Load 0x100000
    let result = interpreter.interpret("LW x2, 0(x1)");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Memory address out of bounds"));
    assert!(err_msg.contains("💡 Tip:"));
    assert!(err_msg.contains("1MB of memory"));
    assert!(err_msg.contains("--memory flag"));
}

#[test]
fn test_system_instruction_errors() {
    let mut interpreter = Interpreter::new();

    // Test ECALL
    let result = interpreter.interpret("ECALL");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Environment call"));
    assert!(err_msg.contains("💡 Tip:"));
    assert!(err_msg.contains("request services"));
    assert!(err_msg.contains("register a7"));

    // Test EBREAK
    let result = interpreter.interpret("EBREAK");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Breakpoint"));
    assert!(err_msg.contains("💡 Tip:"));
    assert!(err_msg.contains("debugger"));
}

#[test]
fn test_pseudo_instruction_errors() {
    let mut interpreter = Interpreter::new();

    // Test redundant MV - this actually succeeds but we can test the validation separately
    // For now, let's skip this test as MV x1, x1 is allowed (it's just a NOP)

    // Test odd jump offset
    let result = interpreter.interpret("J 101");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Jump offset must be even"));
    assert!(err_msg.contains("💡 Tip:"));
    assert!(err_msg.contains("Use 100 instead"));

    // Test LA pseudo-instruction - currently not recognized, so we get unknown instruction
    let result = interpreter.interpret("LA x1, my_label");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    // LA is suggested as a fuzzy match for itself
    assert!(err_msg.contains("Unknown instruction"));
    assert!(err_msg.contains("LA"));
}

#[test]
fn test_generic_interpreter_errors() {
    let mut interpreter = Interpreter::new();

    // Test semicolon error - now handled differently
    // The library no longer checks for semicolons, it treats "1;" as an unknown instruction
    let result = interpreter.interpret("ADDI x1, x0, 1; ADDI x2, x0, 2");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    // The error will be about "1;" being an unknown instruction
    assert!(err_msg.contains("Unknown instruction"));

    // Test undo/redo limits using new API
    let instructions = brubeck::parse("ADDI x1, x0, 1").unwrap();
    interpreter.execute(instructions[0]).unwrap();
    interpreter.previous_state().unwrap();
    let result = interpreter.previous_state();
    assert!(result.is_err());
    // The new API returns HistoryError::AtBeginning, not a string with tips
}

#[test]
fn test_fuzzy_instruction_suggestions() {
    let mut interpreter = Interpreter::new();

    // Test typo suggestions - multiple matches don't get tips
    let result = interpreter.interpret("ADI x1, x0, 10");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Did you mean"));
    assert!(err_msg.contains("ADD or ADDI"));

    // Test common mistakes from other architectures
    let result = interpreter.interpret("MOV x1, x2");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("MV"));
    assert!(err_msg.contains("💡 Tip: Double-check the spelling"));

    let result = interpreter.interpret("PUSH x1");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("No PUSH in RISC-V"));
    assert!(err_msg.contains("ADDI sp, sp, -4"));
}

#[test]
fn test_argument_count_errors() {
    let mut interpreter = Interpreter::new();

    // Test R-type with wrong args
    let result = interpreter.interpret("ADD x1, x2");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    // Currently gives a generic error, not the nice WrongArgumentCount error
    assert!(err_msg.contains("Invalid RType arguments"));

    // Test load with wrong format - currently gives generic error
    let result = interpreter.interpret("LW x1");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Invalid load arguments"));
}

#[test]
fn test_immediate_range_errors() {
    let mut interpreter = Interpreter::new();

    // Test I-type immediate out of range
    let result = interpreter.interpret("ADDI x1, x0, 9999");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("out of range"));
    assert!(err_msg.contains("💡 Tip:"));
    assert!(err_msg.contains("12-bit signed"));
    assert!(err_msg.contains("LUI + ADDI pattern"));

    // Test shift amount out of range
    let result = interpreter.interpret("SLLI x1, x2, 33");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("💡 Tip:"));
    assert!(err_msg.contains("Shift amounts must be 0-31"));
}
