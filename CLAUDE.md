# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Brubeck is a RISC-V assembly language REPL and emulation library written in Rust. It implements the RV32I (32-bit integer) instruction set and provides both a library interface and an interactive REPL for executing RISC-V assembly instructions.

The implementation follows the official RISC-V ISA specification, available as AsciiDoc source files in `riscv-isa-manual/src/`. The RV32I base instruction set is documented in `riscv-isa-manual/src/rv32.adoc`.

## Common Development Commands

### Build and Run
- `cargo build` - Build the project in debug mode
- `cargo build --release` - Build optimized release version
- `cargo run` - Launch the Brubeck RISC-V REPL
- `cargo run --release` - Run the optimized REPL

### Testing
- `cargo test` - Run all unit tests
- `cargo test -- --nocapture` - Run tests with println! output visible
- `cargo test test_name` - Run a specific test by name
- `cargo test --test unit_components` - Run component unit tests
- `cargo test --test unit_instructions` - Run instruction unit tests
- `cargo test --test parser` - Run parser integration tests
- `cargo test --test pseudo_instructions` - Run pseudo-instruction tests

### Code Quality
- `cargo fmt` - Format code according to Rust standards
- `cargo clippy` - Run linter for code improvements
- `cargo check` - Quick error check without building

### Documentation
- `cargo doc --open` - Generate and open documentation

## Architecture Overview

The codebase is organized as both a library and binary application:

### Core Components

1. **CPU Emulation (`src/rv32_i/`)**
   - `cpu.rs` - Main CPU emulator with 32 general-purpose registers and 1 MiB memory
   - `instructions.rs` - RV32I instruction definitions and opcode decoding
   - `formats.rs` - Instruction encoding formats (R, I, S, B, U, J types)
   - `registers.rs` - Register definitions with ABI names (x0-x31, zero, ra, sp, etc.)
   - `pseudo_instructions.rs` - RV32I-specific pseudo-instructions (MV, NOT, LI, etc.)

2. **REPL Interface (`src/`)**
   - `interpreter.rs` - REPL command parsing and execution engine
   - `bin/brubeck.rs` - Binary entry point for the REPL application
   - `lib.rs` - Library entry point exposing public API

3. **Utilities**
   - `immediate.rs` - Sign extension utilities for immediate values

### Key Design Patterns

- The CPU struct maintains state with registers and memory
- Instructions are decoded using pattern matching on opcode/funct3/funct7 fields
- The interpreter handles both assembly instruction parsing and REPL commands
- Comprehensive unit tests in `rv32_i/mod.rs` validate instruction implementations

### Current Features

- Complete RV32I instruction set implementation
- System instructions: FENCE, ECALL, EBREAK  
- Common pseudo-instructions: MV, NOT, SEQZ, SNEZ, J, JR, RET, LI
- Support for hex (0x), binary (0b), and decimal immediate values
- Memory limited to 1 MiB

### Limitations

- No support for other RISC-V extensions (M, A, F, D, etc.)
- REPL is in early development with basic parsing capabilities
- Parser cannot handle negative immediate values in assembly

## Testing Approach

Tests have been reorganized into a structured hierarchy under the `tests/` directory:

### Test Organization
- **Unit Tests** (`tests/unit/`)
  - `components/` - Tests for core components like immediates
  - `instructions/` - Comprehensive tests for each instruction category
- **Integration Tests** (`tests/`)
  - `parser.rs` - Tests for the REPL parser and interpreter
  - `pseudo_instructions.rs` - Tests for pseudo-instruction parsing and expansion

### Known Issues
- **Negative Immediates**: Parser can't handle negative immediate values in assembly syntax (e.g., "ADDI x1, x0, -5" won't parse, but "LI x1, -5" works)

For detailed testing goals and coverage status, see `TESTING_GOALS.md` and `tests/TEST_COVERAGE.md`.

## Implementing New Instructions

When adding new RISC-V instructions, follow the systematic process documented in `INSTRUCTION_IMPLEMENTATION.md`. This guide provides:
- Step-by-step implementation checklist
- Code examples for each phase
- Current implementation status
- References to relevant specification sections

The process ensures each instruction is correctly implemented across all layers: definition, decoding, execution, parsing, and testing.