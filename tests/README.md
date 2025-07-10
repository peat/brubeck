# Brubeck Test Organization

This document describes the test structure and organization for the Brubeck project.

## Overview

Tests are organized into three main categories:

1. **Unit Tests** (`tests/unit/`) - Fine-grained tests for individual components
2. **Integration Tests** (`tests/*.rs`) - End-to-end tests of the interpreter and REPL
3. **Common Infrastructure** (`tests/common/`) - Shared test utilities and helpers

## Directory Structure

```
tests/
├── README.md                 # This file
├── TEST_COVERAGE.md         # Detailed test coverage report
│
├── common/                  # Shared test infrastructure
│   ├── mod.rs              # Module declarations
│   ├── assertions.rs       # Custom assertion helpers
│   ├── context.rs          # Test context builders
│   └── values.rs           # Common test constants
│
├── unit/                    # Unit tests
│   ├── mod.rs              # Module declarations
│   ├── test_helpers.rs     # Unit test specific helpers
│   ├── repl_history.rs     # REPL command history tests
│   ├── cli.rs              # CLI functionality tests
│   ├── history.rs          # Undo/redo history tests
│   │
│   ├── components/         # Component tests
│   │   ├── immediate.rs    # Immediate value tests
│   │   └── csr.rs          # CSR functionality tests
│   │
│   └── instructions/       # Instruction tests
│       ├── arithmetic.rs   # ADD, ADDI, SUB
│       ├── branches.rs     # BEQ, BNE, BLT, BGE, BLTU, BGEU
│       ├── comparison.rs   # SLT, SLTI, SLTU, SLTIU
│       ├── csr.rs          # CSR instructions
│       ├── jumps.rs        # JAL, JALR
│       ├── loads_stores.rs # Load/Store instructions
│       ├── logical.rs      # AND, OR, XOR, ANDI, ORI, XORI
│       ├── misc.rs         # NOP and other misc
│       ├── pseudo.rs       # Pseudo-instruction tests
│       ├── shifts.rs       # Shift instructions
│       ├── system.rs       # FENCE, ECALL, EBREAK
│       └── upper_immediate.rs # LUI, AUIPC
│
├── unit_tests.rs           # Unit test entry point
├── cli_args.rs             # CLI argument parsing tests
├── cli_history.rs          # CLI history flag tests
├── parser.rs               # Parser integration tests
├── pseudo_instructions.rs  # Pseudo-instruction integration tests
└── undo_redo.rs           # Undo/redo integration tests
```

## Test Categories

### Unit Tests

Unit tests focus on individual components and instructions in isolation:

- **Component Tests**: Test individual components like immediate values, CSRs
- **Instruction Tests**: Test each RV32I instruction's behavior
- **Helper Tests**: Test CLI parsing, history management, etc.

Unit tests use the builder pattern and custom assertions for clarity:

```rust
let mut cpu = CpuBuilder::new()
    .with_register(Register::X1, 10)
    .with_register(Register::X2, 20)
    .build();

cpu.execute_expect(ADD(add_inst), "ADD should succeed");
cpu.assert_register(Register::X3, 30, "10 + 20 = 30");
```

### Integration Tests

Integration tests verify end-to-end functionality:

- **Parser Tests**: Test instruction parsing and command handling
- **Undo/Redo Tests**: Test state management across multiple operations
- **CLI Tests**: Test command-line argument handling
- **Pseudo-instruction Tests**: Test expansion and execution

### Common Infrastructure

Shared test utilities used across both unit and integration tests:

- **values.rs**: Common constants (NEG_ONE, IMM12_MAX, etc.)
- **assertions.rs**: Custom assertion helpers
- **context.rs**: Test context builders and utilities

## Running Tests

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --test unit_tests

# Run specific test categories
cargo test --test unit_tests components::immediate
cargo test --test unit_tests instructions::arithmetic

# Run integration tests
cargo test --test parser
cargo test --test undo_redo

# Run with output visible
cargo test -- --nocapture
```

## Writing New Tests

### Adding a Unit Test

1. Choose the appropriate file in `tests/unit/`
2. Use the test helpers and builder pattern
3. Include descriptive test names and comments
4. Test both success and error cases

Example:
```rust
#[test]
fn test_addi_sign_extension() {
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, 0)
        .build();
    
    // Test negative immediate sign extension
    let inst = /* create ADDI instruction */;
    cpu.execute_expect(inst, "ADDI with negative immediate");
    cpu.assert_register(Register::X1, expected_value, "Sign extension check");
}
```

### Adding an Integration Test

1. Create a new file in `tests/` or add to existing file
2. Test end-to-end scenarios
3. Use the Interpreter directly
4. Test complex workflows

Example:
```rust
#[test]
fn test_complex_program_execution() {
    let mut interpreter = Interpreter::default();
    
    // Execute a series of instructions
    assert!(interpreter.interpret("ADDI x1, x0, 10").is_ok());
    assert!(interpreter.interpret("ADDI x2, x0, 20").is_ok());
    assert!(interpreter.interpret("ADD x3, x1, x2").is_ok());
    
    // Verify final state
    assert_eq!(interpreter.cpu().get_register(Register::X3), 30);
}
```

## Test Coverage

See `TEST_COVERAGE.md` for detailed coverage information including:
- Instruction coverage (all 47 RV32I instructions)
- CSR instruction coverage
- Pseudo-instruction coverage
- Edge cases and error conditions

## Best Practices

1. **Use descriptive test names** that explain what is being tested
2. **Include comments** explaining the test's purpose and any RISC-V specifics
3. **Test edge cases** including overflow, underflow, and boundary conditions
4. **Use test helpers** to reduce boilerplate and improve readability
5. **Group related tests** using nested modules when appropriate
6. **Verify both success and failure** paths
7. **Keep tests focused** - one concept per test

## Maintenance

When adding new features:
1. Add corresponding unit tests in the appropriate file
2. Add integration tests for end-to-end scenarios
3. Update TEST_COVERAGE.md with new test information
4. Run all tests to ensure no regressions