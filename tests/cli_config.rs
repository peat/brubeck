//! Tests for CLI configuration integration

use brubeck::interpreter::Interpreter;
use brubeck::rv32_i::Register;

#[test]
fn test_custom_memory_size() {
    // Create interpreter with 16KB of memory
    let mut interpreter = Interpreter::with_config(16 * 1024, 100);

    // Should be able to access memory up to 16KB - 4
    let result = interpreter.interpret("LUI x1, 0x3");
    assert!(result.is_ok());
    let result = interpreter.interpret("ADDI x1, x1, 0x7FC");
    assert!(result.is_ok());
    let result = interpreter.interpret("SW x1, 0(x1)");
    assert!(result.is_ok());

    // But accessing at 16KB should fail
    let result = interpreter.interpret("LUI x1, 0x4");
    assert!(result.is_ok());
    let result = interpreter.interpret("SW x1, 0(x1)");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    // The new error format includes more detail
    assert!(err_msg.contains("Memory address out of bounds"));
}

#[test]
fn test_custom_history_limit() {
    // Create interpreter with history limit of 3
    let mut interpreter = Interpreter::with_config(1024 * 1024, 3);

    // Execute 5 instructions
    interpreter.interpret("ADDI x1, x0, 1").unwrap();
    interpreter.interpret("ADDI x2, x0, 2").unwrap();
    interpreter.interpret("ADDI x3, x0, 3").unwrap();
    interpreter.interpret("ADDI x4, x0, 4").unwrap();
    interpreter.interpret("ADDI x5, x0, 5").unwrap();

    // Should be able to undo 3 times (history limit)
    assert!(interpreter.previous_state().is_ok());
    assert_eq!(interpreter.cpu().get_register(Register::X5), 0);

    assert!(interpreter.previous_state().is_ok());
    assert_eq!(interpreter.cpu().get_register(Register::X4), 0);

    assert!(interpreter.previous_state().is_ok());
    assert_eq!(interpreter.cpu().get_register(Register::X3), 0);

    // Fourth undo should fail (exceeded history limit)
    assert!(interpreter.previous_state().is_err());

    // x1 and x2 should still have their values (not in history)
    assert_eq!(interpreter.cpu().get_register(Register::X1), 1);
    assert_eq!(interpreter.cpu().get_register(Register::X2), 2);
}

#[test]
fn test_zero_history_limit() {
    // Create interpreter with history disabled
    let mut interpreter = Interpreter::with_config(1024 * 1024, 0);

    // Execute an instruction
    interpreter.interpret("ADDI x1, x0, 42").unwrap();
    assert_eq!(interpreter.cpu().get_register(Register::X1), 42);

    // Undo should fail immediately
    let result = interpreter.previous_state();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("No previous state"));
}

#[test]
fn test_default_configuration() {
    // Default interpreter should have 1MB memory and 1000 history limit
    let mut interpreter = Interpreter::new();

    // Can access near 1MB boundary
    interpreter.interpret("LUI x1, 0xFF").unwrap();
    interpreter.interpret("ADDI x1, x1, 0x7FC").unwrap();
    interpreter.interpret("SW x1, 0(x1)").unwrap();

    // Can undo many times (default is 1000)
    for i in 1..=50 {
        interpreter
            .interpret(&format!("ADDI x{}, x0, {}", i % 31 + 1, i))
            .unwrap();
    }

    // Should be able to undo all 50 instructions
    for _ in 0..50 {
        assert!(interpreter.previous_state().is_ok());
    }
}
