# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Important Instructions

1. **Always use PROJECT_STATUS.md** to track current work and planned tasks. Do not create multiple tracking files - consolidate everything in PROJECT_STATUS.md.
2. **Always write comprehensive tests first** before implementing new features or making changes. Follow TDD (Test-Driven Development) practices.

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
- `cargo clippy -- -D warnings` - Ensure no clippy warnings (CI requirement)

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

2. **Interpreter System (`src/interpreter/`)** - Modular architecture for parsing and execution
   - `parser.rs` (643 lines) - Four-phase parsing pipeline with educational error messages
   - `builder.rs` (819 lines) - Instruction building and validation logic
   - `executor.rs` (252 lines) - Command execution and state management
   - `formatter.rs` (403 lines) - Human-readable output formatting
   - `validator.rs` (174 lines) - Common validation functions
   - `types.rs` (128 lines) - Shared types (Command, Token, Error)
   - `interpreter.rs` (188 lines) - Main interpreter orchestration

3. **REPL Infrastructure (`src/`)**
   - `cli.rs` (232 lines) - Command-line argument parsing with clap
   - `history.rs` (286 lines) - Undo/redo state management with delta compression
   - `bin/brubeck.rs` (233 lines) - Binary entry point with terminal features
   - `lib.rs` (30 lines) - Library entry point exposing public API
   - `interpreter.rs` (188 lines) - Main interpreter orchestration

4. **Utilities**
   - `immediate.rs` - Sign extension utilities for immediate values

### Key Design Patterns

- The CPU struct maintains state with registers and memory
- Instructions are decoded using pattern matching on opcode/funct3/funct7 fields
- The interpreter is split into focused modules for maintainability
- Commands use `/` prefix to distinguish from instructions
- Comprehensive unit tests validate all instruction implementations

### Parser Architecture (Teaching-Focused)

The parser is designed as an educational resource demonstrating compiler front-end techniques:

**Modular Architecture:**
- **Parser Module**: Four-phase parsing (normalize → tokenize → build → execute)
- **Builder Module**: Type-specific instruction builders with validation
- **Executor Module**: Command dispatch and state tracking
- **Formatter Module**: Human-readable output generation
- **Validator Module**: Reusable validation logic

**Educational Features:**
- Comprehensive function documentation with examples
- Helper functions demonstrating compiler patterns
- Rich error messages with contextual tips
- Comments explaining design decisions
- Clear module boundaries for learning

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
- **Memory**: Configurable size (default 1 MiB) with proper load/store operations
- **Command system**: `/regs`, `/help`, `/undo`, `/redo` with aliases
- **CLI support**: Script files, one-liners, memory configuration

### Limitations

- No support for other RISC-V extensions (M, A, F, D, etc.)
- No support for labels or assembler directives
- REPL lacks advanced features like command history or tab completion

## Recent Major Refactoring

**Status**: Completed - interpreter split into modular architecture

The interpreter.rs file (previously 1785 lines, now 188 lines) has been refactored into 6 focused modules:

### New Interpreter Architecture (`src/interpreter/`)
- **`parser.rs`** (643 lines) - Complete parsing pipeline
- **`builder.rs`** (819 lines) - Instruction building and validation
- **`executor.rs`** (252 lines) - Command execution and state management
- **`formatter.rs`** (403 lines) - Human-readable output formatting
- **`validator.rs`** (174 lines) - Input validation functions
- **`types.rs`** (128 lines) - Common types (Command, Token, Error)

### Key Improvements
- **Better separation of concerns**: Each module has a single responsibility
- **Removed direct register inspection**: No more confusing `x1` commands, use `/regs` instead
- **Clean code**: Zero clippy warnings, proper Rust idioms throughout
- **Educational structure**: Demonstrates good software architecture practices

See `REFACTORING_SUMMARY.md` for complete details.

## Recent Major Features

### Command-Line Interface

**Status**: Completed

Brubeck now has a comprehensive CLI using `clap`:
- **Memory configuration**: `-m, --memory <size>` (e.g., 1M, 256k)
- **History configuration**: `--undo-limit <n>`, `--no-undo`
- **Execution modes**: `-e, --execute <commands>`, `-s, --script <file>`
- **Automatic mode detection**: Banner/prompt suppression for non-interactive use
- **Human-friendly parsing**: Memory sizes like "1k", "5M", "1GB"

### Undo/Redo Functionality

**Status**: Completed

We have implemented a comprehensive undo/redo system for the REPL that:
- Allows users to undo instruction execution with `/undo` or `/u`
- Supports redo with `/redo`
- Uses efficient delta compression for memory changes
- Maintains a configurable history (default: 1000 states)
- Only tracks successfully executed instructions
- Has comprehensive test coverage for all RV32I instructions

Implementation uses `src/history.rs` for state management with efficient delta compression.

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

### Command System Implementation
- **Completed**: `/regs` (alias `/r`), `/help` (alias `/h`), `/undo` (alias `/u`), `/redo`
- **Direct register inspection removed**: Must use `/regs x1` instead of `x1`
- **Flexible syntax**: `/regs x1 x2 sp` shows specific registers

### Current Development

See `PROJECT_STATUS.md` for the consolidated task list and roadmap.

Key priorities:
1. Rename history navigation commands for clarity
2. Add `/memory` command for debugging
3. Add `/reset` command with safety confirmation
4. Enhance error messages with educational content

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

For test coverage details, see `tests/TEST_COVERAGE.md`.

## Implementing New Instructions

When adding new RISC-V instructions, follow the systematic process documented in `INSTRUCTION_IMPLEMENTATION.md`. This guide provides:
- Step-by-step implementation checklist
- Code examples for each phase
- Current implementation status
- References to relevant specification sections

The process ensures each instruction is correctly implemented across all layers: definition, decoding, execution, parsing, and testing.

## Code Quality Requirements

**IMPORTANT**: Before marking any task as complete, you MUST run the following commands to ensure code quality:

1. `cargo fmt` - Format code according to Rust standards
2. `cargo clippy -- -D warnings` - Ensure no clippy warnings
3. `cargo test` - Run all tests to ensure nothing is broken

Only proceed with marking a task complete if all three commands pass successfully. If any fail, fix the issues before continuing.

## Git Workflow

**IMPORTANT**: Follow these Git practices for all development work:

1. **Commit frequently** - Make small, focused commits rather than large ones
   - Easier to review and understand changes
   - Simpler to cherry-pick specific changes
   - Reduces merge conflicts with other contributors

2. **Write clear commit messages** - Describe what changed and why

3. **Use branches for larger features** - For tasks requiring multiple commits:
   - Create a feature branch for the work
   - Make regular commits as you progress
   - Keep commits atomic and focused

4. **Never commit without testing** - Always run `cargo fmt`, `cargo clippy`, and `cargo test` before committing

Example workflow:
```bash
# After making a small, focused change
cargo fmt
cargo clippy -- -D warnings
cargo test
git add -p  # Review changes
git commit -m "Add validation for CSR addresses"

# For larger features
git checkout -b feature/memory-inspection
# ... make changes, commit frequently ...
git push origin feature/memory-inspection
```