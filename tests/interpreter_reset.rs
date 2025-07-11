//! Tests for the Interpreter reset method

use brubeck::{parse, Interpreter};

#[test]
fn test_reset_clears_registers() {
    let mut interpreter = Interpreter::new();

    // Execute some instructions to change state
    let instructions = parse("ADDI x1, x0, 42").unwrap();
    interpreter.execute(instructions[0]).unwrap();

    let instructions = parse("ADDI x2, x0, 99").unwrap();
    interpreter.execute(instructions[0]).unwrap();

    // Verify registers are set
    assert_eq!(
        interpreter.cpu.get_register(brubeck::rv32_i::Register::X1),
        42
    );
    assert_eq!(
        interpreter.cpu.get_register(brubeck::rv32_i::Register::X2),
        99
    );
    assert_eq!(interpreter.cpu.pc, 8); // Two instructions executed

    // Reset
    interpreter.reset();

    // Verify everything is back to initial state
    assert_eq!(
        interpreter.cpu.get_register(brubeck::rv32_i::Register::X1),
        0
    );
    assert_eq!(
        interpreter.cpu.get_register(brubeck::rv32_i::Register::X2),
        0
    );
    assert_eq!(interpreter.cpu.pc, 0);
}

#[test]
fn test_reset_clears_history() {
    let mut interpreter = Interpreter::new();

    // Execute some instructions
    let instructions = parse("ADDI x1, x0, 10").unwrap();
    interpreter.execute(instructions[0]).unwrap();

    let instructions = parse("ADDI x2, x0, 20").unwrap();
    interpreter.execute(instructions[0]).unwrap();

    // Navigate back in history
    interpreter.previous_state().unwrap();
    assert_eq!(
        interpreter.cpu.get_register(brubeck::rv32_i::Register::X2),
        0
    );

    // Reset
    interpreter.reset();

    // Try to navigate back - should fail because history is cleared
    let result = interpreter.previous_state();
    assert!(result.is_err());
}

#[test]
fn test_reset_preserves_configuration() {
    // Create interpreter with custom configuration
    let mut interpreter = Interpreter::with_config(2048, 100);

    // Execute some instructions
    let instructions = parse("ADDI x1, x0, 42").unwrap();
    interpreter.execute(instructions[0]).unwrap();

    // Reset
    interpreter.reset();

    // Configuration should be preserved (memory size, history limit)
    // We can't directly test these, but we can verify the interpreter still works
    let instructions = parse("ADDI x3, x0, 99").unwrap();
    let delta = interpreter.execute(instructions[0]).unwrap();
    assert_eq!(delta.register_changes.len(), 1);
}
