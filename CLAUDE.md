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
   - `cpu.rs` - Main CPU emulator with 32 general-purpose registers, 1 MiB memory, and CSR (Control and Status Register) support
   - `instructions.rs` - RV32I instruction definitions and opcode decoding
   - `formats.rs` - Instruction encoding formats (R, I, S, B, U, J types)
   - `registers.rs` - Register definitions with ABI names (x0-x31, zero, ra, sp, etc.)
   - `pseudo_instructions.rs` - RV32I-specific pseudo-instructions (MV, NOT, LI, etc.)

2. **REPL Interface (`src/`)**
   - `interpreter.rs` - Production-grade parser with comprehensive validation and educational error messages
   - `bin/brubeck.rs` - Binary entry point for the REPL application
   - `lib.rs` - Library entry point exposing public API

3. **Utilities**
   - `immediate.rs` - Sign extension utilities for immediate values

### Key Design Patterns

- The CPU struct maintains state with registers and memory
- Instructions are decoded using pattern matching on opcode/funct3/funct7 fields
- The interpreter handles both assembly instruction parsing and REPL commands
- Comprehensive unit tests in `rv32_i/mod.rs` validate instruction implementations

### Parser Architecture (Teaching-Focused)

The parser is designed as an educational resource demonstrating compiler front-end techniques:

**Four-Phase Parsing Process:**
1. **Normalize**: Clean input (whitespace, case conversion, punctuation)
2. **Tokenize**: Convert strings to typed tokens (instructions, registers, values)
3. **Build Commands**: Construct validated instruction objects
4. **Execute**: Run instructions on the CPU emulator

**Educational Features:**
- Comprehensive function documentation with examples
- Helper functions demonstrating common compiler patterns
- Rich error messages with contextual tips and RISC-V education
- Comments explaining "why" behind design decisions
- Single-file architecture for easy linear reading

**Validation & Error Handling:**
- PC register protection (prevents misuse)
- Immediate range validation for each instruction type
- Argument count checking with instruction-specific guidance
- Support for both standard and legacy assembly syntax

### Current Features

- **Complete RV32I instruction set implementation** (47 instructions)
- **CSR (Control and Status Register) support** with 6 CSR instructions:
  - CSRRW, CSRRS, CSRRC (register variants)
  - CSRRWI, CSRRSI, CSRRCI (immediate variants)
  - Standard CSRs: MSTATUS, MISA, CYCLE, TIME, INSTRET, MSCRATCH, MEPC, MCAUSE, etc.
- **System instructions**: FENCE, ECALL, EBREAK  
- **Common pseudo-instructions**: MV, NOT, SEQZ, SNEZ, J, JR, RET, LI
- **Multiple immediate formats**: Hex (0x), binary (0b), and decimal values
- **Production-grade parser** with comprehensive validation and educational error messages
- **Standard RISC-V assembly syntax**: Both `LW x1, 4(x2)` and legacy `LW x1, x2, 4` formats
- **Robust validation**: PC register protection, immediate range checking, argument validation
- **Memory**: 1 MiB address space with proper load/store operations

### Limitations

- No support for other RISC-V extensions (M, A, F, D, etc.)
- No support for labels or assembler directives
- REPL lacks advanced features like command history or tab completion

## Current Project: Command-Line Interface

**Status**: In progress - implementing command-line argument parsing

We are currently adding command-line argument support to Brubeck using the `clap` crate. This will allow users to configure memory size, undo/redo limits, and run scripts or one-liner commands.

### Planned CLI Arguments
- **Memory configuration**: `-m, --memory <size>` (e.g., 1M, 256k)
- **History configuration**: `--undo-limit <n>`, `--no-undo`
- **Execution modes**: `-e, --execute <commands>`, `-s, --script <file>`
- **Standard options**: `-h, --help`, `-V, --version`

### Implementation Progress
See `docs/specs/CLI_ARGS_SPEC.md` for the implementation specification.

### Related Improvements
- Adding semicolon support to parser for multi-command lines
- Automatic banner suppression in non-interactive modes
- Human-friendly memory size parsing (1k, 5M, etc.)

## Recent Improvements

### Undo/Redo Functionality

**Status**: Completed

We have implemented a comprehensive undo/redo system for the REPL that:
- Allows users to undo instruction execution with `/undo` or `/u`
- Supports redo with `/redo`
- Uses efficient delta compression for memory changes
- Maintains a configurable history (default: 1000 states)
- Only tracks successfully executed instructions
- Has comprehensive test coverage for all RV32I instructions

See `docs/specs/UNDO_REDO_SPEC.md` for the implementation specification.

### REPL Usability

**Status**: Completed based on user feedback from hands-on testing

We have implemented significant REPL usability improvements to make Brubeck more beginner-friendly and educational:

### Completed Improvements
1. **PC address prompt**: `[0x00000000]> ` shows current execution address
2. **Human-readable output**: All instructions show their mnemonic and describe what they did
   - Example: `ADDI: Added 42 to X0 (0) and stored result in X1 (42)`
3. **Instruction mnemonics**: Added `mnemonic()` method to Instruction enum for clean access
4. **Non-interactive mode**: Supports piped input for testing and scripting
   - `echo "ADDI x1, x0, 42" | brubeck` works seamlessly
5. **Colorized output**: Interactive mode uses colors (green ✅, red ❌)
6. **Terminal features**: Full terminal support via `crossterm` (optional, binary-only)

### Architecture Decisions
- **Library remains pure**: No dependencies, ready for no-std and WASM
- **Binary has rich features**: Terminal colors, TTY detection, etc. via feature flags
- **Clean separation**: All REPL enhancements are in the binary, not the library

### Still Planned
- **Command system**: `/regs`, `/memory`, `/help`, `/reset` with "/" prefix
- **Register state overview**: Show all registers at once
- **Safety confirmations**: Prevent accidental state loss

See `REPL_USABILITY_FEEDBACK.md` for the original analysis.

## Testing Approach

Tests have been reorganized into a structured hierarchy under the `tests/` directory:

### Test Organization
- **Unit Tests** (`tests/unit/`)
  - `components/` - Tests for core components like immediates
  - `instructions/` - Comprehensive tests for each instruction category
- **Integration Tests** (`tests/`)
  - `parser.rs` - Tests for the REPL parser and interpreter
  - `pseudo_instructions.rs` - Tests for pseudo-instruction parsing and expansion

### Test Coverage Status
- **RV32I Instructions**: Complete coverage (47/47 instructions)
- **CSR Instructions**: Complete coverage (6/6 instructions)
- **Pseudo-instructions**: Complete coverage (8/8 pseudo-instructions)
- **Parser Features**: Comprehensive integration tests
- **Edge Cases**: Extensive validation and error handling tests

**Total Test Count**: 350+ tests across unit, integration, and comprehensive test suites.

For detailed testing goals and coverage status, see `docs/specs/TESTING_GOALS.md` and `tests/TEST_COVERAGE.md`.

## Implementing New Instructions

When adding new RISC-V instructions, follow the systematic process documented in `INSTRUCTION_IMPLEMENTATION.md`. This guide provides:
- Step-by-step implementation checklist
- Code examples for each phase
- Current implementation status
- References to relevant specification sections

The process ensures each instruction is correctly implemented across all layers: definition, decoding, execution, parsing, and testing.