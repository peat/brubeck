# Brubeck Testing Goals

This document describes our aspirational testing strategy for Brubeck. It represents our "big hairy audacious goal" for comprehensive, educational testing that serves both correctness and learning.

## Overview
Brubeck is an educational RISC-V emulator designed to help users understand how the RISC-V ISA works. Our testing strategy prioritizes:
1. **Correctness** - Exact compliance with the RISC-V specification
2. **Robustness** - Graceful handling of edge cases and errors
3. **Usability** - Clear, educational feedback that aids learning

## Testing Philosophy

### Why We Test
- **Specification Compliance**: Every instruction must behave exactly as the RISC-V spec defines
- **Educational Value**: Tests serve as executable documentation and learning examples
- **Confidence in Learning**: Users need to trust that what they learn from Brubeck is correct
- **Debugging Aid**: When something goes wrong, tests help users understand why

### What We Test
1. **Instruction Semantics**: Does each instruction do exactly what the spec says?
2. **Edge Cases**: What happens at the boundaries of valid behavior?
3. **Error Conditions**: How do we handle invalid inputs or operations?
4. **State Consistency**: Is the CPU state always valid and predictable?
5. **User Experience**: Are error messages helpful and educational?

## Test Categories

### 1. Unit Tests
**Purpose**: Verify individual components work correctly in isolation

#### a. Instruction Tests
- **What**: Each RV32I instruction implementation
- **Why**: Ensure spec compliance at the most granular level
- **How**: Test normal operation, edge cases, and special behaviors
- **Example**: Testing that `ADDI` correctly sign-extends 12-bit immediates

#### b. Format Tests
- **What**: Instruction encoding/decoding for each format (R, I, S, B, U, J)
- **Why**: Correct encoding is fundamental to instruction execution
- **How**: Encode/decode round trips, invalid encoding rejection
- **Example**: Verifying B-format encodes branch offsets in multiples of 2

#### c. Component Tests
- **What**: CPU state management, memory model, immediate values
- **Why**: These are the building blocks that instructions depend on
- **How**: Test invariants, boundaries, and state transitions
- **Example**: Ensuring x0 register always reads as zero

### 2. Integration Tests
**Purpose**: Verify components work correctly together

#### a. Instruction Sequences
- **What**: Multi-instruction programs
- **Why**: Real programs use instructions in combination
- **How**: Test common patterns (loops, function calls, etc.)
- **Example**: Function prologue/epilogue sequences

#### b. Parser Integration
- **What**: String input → tokenization → instruction building → execution
- **Why**: This is how users interact with Brubeck
- **How**: Test valid and invalid assembly syntax
- **Example**: "ADD x1, x2, x3" produces correct instruction and result

#### c. REPL Sessions
- **What**: Complete user interactions
- **Why**: Tests the full user experience
- **How**: Simulate realistic REPL sessions
- **Example**: Loading values, performing calculations, inspecting results

### 3. Specification Compliance Tests
**Purpose**: Ensure Brubeck matches the RISC-V specification exactly

#### a. Spec Example Tests
- **What**: Examples directly from the RISC-V specification
- **Why**: These are the authoritative examples
- **How**: Implement each spec example as a test
- **Example**: Overflow handling examples from the spec

#### b. Instruction Coverage
- **What**: Every defined behavior for each instruction
- **Why**: Complete compliance requires complete coverage
- **How**: Systematic testing of all documented behaviors
- **Example**: All addressing modes for load/store instructions

#### c. Cross-Validation Tests (Future)
- **What**: Compare Brubeck output with reference simulators
- **Why**: External validation increases confidence
- **How**: Run same programs on Brubeck and Spike/QEMU
- **Example**: Comparing register states after complex programs

### 4. Error Handling Tests
**Purpose**: Ensure errors are caught and reported helpfully

#### a. Invalid Input Tests
- **What**: Malformed assembly, invalid instructions
- **Why**: Users will make mistakes while learning
- **How**: Test parser error messages and recovery
- **Example**: "ADD x1, x2" (missing operand) produces clear error

#### b. Runtime Error Tests
- **What**: Memory violations, misaligned access, invalid operations
- **Why**: Understanding limits is part of learning the ISA
- **How**: Trigger each error condition and verify message
- **Example**: Jump to misaligned address explains alignment requirements

#### c. Boundary Tests
- **What**: Maximum values, memory limits, immediate ranges
- **Why**: Edge cases reveal implementation details
- **How**: Test at and beyond all limits
- **Example**: ADDI with immediate of 2048 (out of range)

### 5. Educational Tests
**Purpose**: Ensure Brubeck effectively teaches RISC-V concepts

#### a. Error Message Quality
- **What**: Clarity and educational value of error messages
- **Why**: Errors are learning opportunities
- **How**: Verify messages explain what, why, and how to fix
- **Example**: "Immediate value 4096 exceeds 12-bit signed range (-2048 to 2047)"

#### b. Diagnostic Output Tests
- **What**: Instruction traces, state dumps, execution explanations
- **Why**: Visibility into execution helps understanding
- **How**: Verify output is accurate and comprehensible
- **Example**: Showing how branch addresses are calculated

#### c. Documentation Tests
- **What**: Code examples in documentation
- **Why**: Documentation should be accurate and runnable
- **How**: Extract and run all documentation examples
- **Example**: README examples produce shown output

## Test Organization

### File Structure
```
tests/
├── unit/
│   ├── instructions/      # One file per instruction type
│   ├── formats/          # Encoding/decoding tests
│   └── components/       # CPU, memory, immediate tests
├── integration/
│   ├── sequences/        # Multi-instruction tests
│   ├── parser/          # Assembly parsing tests
│   └── repl/            # Full REPL interaction tests
├── compliance/
│   ├── spec_examples/   # Direct from specification
│   └── coverage/        # Systematic instruction coverage
├── errors/
│   ├── invalid_input/   # Parser error handling
│   ├── runtime/         # Execution error handling
│   └── boundaries/      # Limit testing
└── educational/
    ├── error_messages/  # Message quality tests
    └── diagnostics/     # Output clarity tests
```

### Test Helpers
- **CPU State Assertions**: Compare entire CPU state with clear diffs
- **Instruction Builders**: Easily create test instructions
- **Spec Reference Helpers**: Link tests to specification sections
- **Educational Assertions**: Verify error messages meet quality standards

## Testing Best Practices

### For Contributors
1. **Every instruction needs tests**: No instruction is too simple to test
2. **Test the why, not just the what**: Comments should explain the ISA behavior being tested
3. **Use descriptive test names**: `test_addi_sign_extends_negative_immediate` not `test_addi_2`
4. **Include spec references**: Link to relevant sections of the RISC-V manual
5. **Consider the learner**: What would help someone understand this behavior?

### For Maintainers
1. **Tests are documentation**: They show how Brubeck should behave
2. **Failed tests should teach**: Error output should explain the concept
3. **Coverage matters**: Systematic testing prevents gaps
4. **External validation**: Regular cross-checking with reference simulators
5. **Test maintenance**: Keep tests up to date with spec changes

## Future Enhancements

### Cross-Validation Framework
- Automated comparison with Spike/QEMU
- Binary instruction encoding/decoding verification
- Trace comparison tools

### Property-Based Testing
- Invariant checking (x0 always zero, PC alignment)
- Instruction property verification (commutativity, etc.)
- State machine properties

### Fuzzing
- Random instruction generation
- Parser fuzzing for robustness
- Memory access pattern fuzzing

### Educational Test Modes
- "Tutorial mode" that explains test failures in detail
- Interactive test exploration
- Test-driven learning exercises