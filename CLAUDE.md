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

### Current Limitations

- Implements RV32I except EBREAK, ECALL, and FENCE instructions
- Memory is limited to 1 MiB
- No support for other RISC-V extensions (M, A, F, D, etc.)
- REPL is in early development with basic parsing capabilities

## Testing Approach

Tests are embedded in source files using `#[cfg(test)]` modules. The main test suite in `src/rv32_i/mod.rs` validates:
- Individual instruction execution
- Register operations
- Memory access patterns
- Edge cases and error conditions

Run tests frequently during development to ensure instruction semantics remain correct.