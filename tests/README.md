# Brubeck Tests

This directory contains the comprehensive test suite for Brubeck, organized to support both correctness verification and educational purposes.

## Quick Start

Run all tests:
```bash
cargo test
```

Run tests for a specific category:
```bash
cargo test --test unit_instructions
cargo test --test integration_parser
```

## Organization

Tests are organized by purpose and scope:

- **`unit/`** - Test individual components in isolation
  - `instructions/` - Each instruction's behavior
  - `formats/` - Instruction encoding/decoding
  - `components/` - CPU, memory, immediates

- **`integration/`** - Test components working together
  - `sequences/` - Multi-instruction programs
  - `parser/` - Assembly parsing pipeline
  - `repl/` - Complete REPL sessions

- **`compliance/`** - Verify RISC-V specification compliance
  - `spec_examples/` - Examples from the spec
  - `coverage/` - Systematic instruction testing

- **`errors/`** - Test error handling and robustness
  - `invalid_input/` - Parser error handling
  - `runtime/` - Execution errors
  - `boundaries/` - Edge cases and limits

- **`educational/`** - Test learning effectiveness
  - `error_messages/` - Message clarity and helpfulness
  - `diagnostics/` - Execution traces and debugging

## Status

See [TEST_COVERAGE.md](TEST_COVERAGE.md) for current coverage status and gaps.

## Contributing

When adding tests:
1. Place them in the appropriate category
2. Include comments explaining what ISA behavior is being tested
3. Reference the RISC-V specification where applicable
4. Consider what the test teaches about RISC-V

See [TESTING_GOALS.md](../TESTING_GOALS.md) for our testing philosophy and best practices.