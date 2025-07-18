# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Important Instructions

1. **Always use PROJECT_STATUS.md** to track current work and planned tasks. Do not create multiple tracking files - consolidate everything in PROJECT_STATUS.md.
2. **Always write tests first** before implementing new features or making changes. Follow TDD (Test-Driven Development) practices.
3. **Always run quality checks** before committing: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`

## Project Overview

Brubeck is a RISC-V assembly language REPL and emulation library written in Rust. It implements the RV32I (32-bit integer) instruction set and provides both a library interface and an interactive REPL for executing RISC-V assembly instructions.

Key design principles:
- **Library remains pure**: No dependencies, ready for no-std and WASM
- **Binary has rich features**: Terminal colors, TTY detection, etc. via feature flags
- **Clean separation**: All REPL enhancements are in the binary, not the library
- **Educational focus**: Clear error messages, helpful documentation, teaching-oriented design

The implementation follows the official RISC-V ISA specification, available in `riscv-isa-manual/src/`.

## Architecture Overview

### Core Components

1. **CPU Emulation (`src/rv32_i/`)**
   - `cpu.rs` - CPU emulator with 32 registers, configurable memory, CSR support
   - `instructions.rs` - RV32I instruction definitions and opcode decoding
   - `formats.rs` - Instruction encoding formats (R, I, S, B, U, J types)
   - `registers.rs` - Register definitions with ABI names
   - `pseudo_instructions.rs` - Common pseudo-instructions (MV, NOT, LI, etc.)

2. **Interpreter System (`src/interpreter/`)** - Modular architecture
   - `parser.rs` - Four-phase parsing pipeline with validation
   - `builder.rs` - Instruction building and validation logic
   - `executor.rs` - Command execution and state management
   - `validator.rs` - Reusable validation functions
   - `types.rs` - Shared types (Command, Token, Error)
   - Returns `StateDelta` for state changes (library/binary separation)

3. **REPL Infrastructure (Binary-only)**
   - `bin/brubeck.rs` - Binary entry point with terminal features
   - `bin/formatting/` - Output formatters with color support:
     - `registers.rs` - Register display with change highlighting
     - `memory.rs` - Memory display with PC and change highlighting
     - `state_delta.rs` - Instruction execution result formatting
     - `errors.rs` - Context-aware error formatting with tips
     - `help.rs` - Help text formatting
   - `bin/repl_commands.rs` - REPL command handling
   - `cli.rs` - Command-line argument parsing with clap
   - `history.rs` - History navigation with delta compression

### Current Features

- **Complete RV32I instruction set** (47 instructions) + CSR support
- **Command system**: 
  - `/regs` (`/r`) - Show registers with color highlighting
  - `/memory` (`/m`) - Inspect memory with color highlighting
  - `/previous` (`/prev`, `/p`) - Navigate to previous state with detailed changes
  - `/next` (`/n`) - Navigate to next state with detailed changes
  - `/reset` - Reset CPU state (with confirmation)
  - `/help` (`/h`) - Show help
- **Multiple number formats**: Hex (0x), binary (0b), decimal
- **Flexible syntax**: Both standard and legacy RISC-V assembly formats
- **History navigation**: Navigate through execution history
- **Memory inspection**: View memory in hex/ASCII format
- **Educational error messages**: Context-aware help for common mistakes
- **Color-coded output**:
  - Registers: Changed values in green, zeros in dark gray
  - Memory: Changed bytes in green, zeros in dark gray, PC location highlighted
  - History navigation: Shows exact changes (e.g., "x2: 100 → 0")

## Development Workflow

### Common Commands

```bash
# Building and Running
cargo build                  # Debug build
cargo build --release        # Release build
cargo run                   # Run REPL
cargo run -- -e "ADDI x1, x0, 42"  # One-liner

# Testing
cargo test                  # Run all tests
cargo test test_name        # Run specific test
cargo test -- --nocapture   # Show println! output

# Code Quality (REQUIRED before commits)
cargo fmt                   # Format code
cargo clippy -- -D warnings # Check for issues
cargo check                # Quick compilation check
```

### Git Workflow

1. **Use feature branches** for any non-trivial changes:
   ```bash
   git checkout -b feature/new-command
   # ... make changes ...
   cargo fmt && cargo clippy -- -D warnings && cargo test
   git add -p  # Review changes
   git commit -m "Add new command implementation"
   ```

2. **Commit guidelines**:
   - Make small, atomic commits (one logical change per commit)
   - Write clear commit messages explaining what and why
   - Separate code changes from documentation updates
   - Fix linter warnings in separate commits

3. **Quality requirements**:
   - All tests must pass
   - Zero clippy warnings
   - Code must be formatted with cargo fmt

### Task Management

1. **Use the TodoWrite tool** to track progress:
   - Create todos when starting a complex task
   - Mark as `in_progress` when beginning work
   - Update to `completed` immediately when done

2. **Update PROJECT_STATUS.md** after completing tasks:
   - Mark completed items with ✅
   - Update status descriptions
   - Keep "Next Action" current

## Testing Guidelines

### Test Organization

- **Unit tests**: In `tests/unit/` for focused component testing
- **Integration tests**: In `tests/` for cross-module functionality
- **Manual testing**: Document any features requiring user interaction

### Writing Effective Tests

1. **Test real scenarios**: Use meaningful test data (e.g., "Hello" not just random bytes)
2. **Test error cases**: Invalid inputs, edge cases, boundary conditions
3. **Test the user's perspective**: What will users actually type?
4. **Keep tests focused**: One concept per test function

Example from memory command:
```rust
// Good: Tests actual usage pattern
i.interpret("ADDI x1, x0, 0x100").unwrap();
i.interpret("ADDI x2, x0, 72").unwrap();     // 'H'
i.interpret("SB x2, 0(x1)").unwrap();
let result = i.interpret("/memory 0x100");
assert!(result.unwrap().contains("48"));     // Hex for 'H'
assert!(result.unwrap().contains("H"));       // ASCII display
```

## Implementation Patterns

### Parser Design

1. **Validate early**: Check constraints in the parser when possible
2. **Provide context**: Error messages should guide users to the solution
3. **Support flexibility**: Accept multiple input formats where sensible

Example from memory command:
```rust
match normalized.len() {
    1 => Ok(Command::ShowMemory { start: None, end: None }),
    2 => {
        let addr = parse_address(&normalized[1])?;
        Ok(Command::ShowMemory { start: Some(addr), end: None })
    }
    3 => {
        let start = parse_address(&normalized[1])?;
        let end = parse_address(&normalized[2])?;
        if end <= start {
            return Err(Error::Generic("End address must be greater..."));
        }
        // ... validate range size ...
    }
    _ => Err(Error::Generic("Too many arguments..."))
}
```

### Formatting and Display

1. **Align output**: Use consistent alignment for readability
2. **Provide dual representations**: Show both raw and interpreted data
3. **Add visual separators**: Help users parse dense information
4. **Use color judiciously**: Highlight changes and important information

Example from register formatter:
```rust
// Consistent column alignment
let reg_str = if use_abi_names && abi_name != "----" {
    format!("x{i} ({abi_name})")
} else {
    format!("x{i}")
};
output.push_str(&format!("{reg_str:<10}: {val_str}"));
```

### State Management

1. **Components own their state**: Each module manages its own lifecycle
2. **Clear reset semantics**: Provide explicit reset methods
3. **Avoid external state manipulation**: Use encapsulation
4. **Return state changes**: Library returns `StateDelta`, binary handles formatting

Good example:
```rust
// Library returns data
pub fn interpret(&mut self, input: &str) -> Result<StateDelta, Error> {
    let command = parser::parse(input)?;
    let delta = executor::run_command(command, self)?;
    Ok(delta)
}

// Binary formats for display
let formatted = formatting::state_delta::format_instruction_result(&delta);
println!("{}", formatted);
```

## Code Style Guidelines

### Rust Best Practices

1. **Follow clippy suggestions**: Use modern Rust idioms
   - `format!("{var}")` not `format!("{}", var)`
   - `(0x20..=0x7E).contains(&byte)` not `byte >= 0x20 && byte <= 0x7E`

2. **Error handling**: Provide actionable error messages
3. **Documentation**: Use doc comments for public APIs
4. **Feature flags**: Keep library and binary features separate

### Documentation Standards

1. **Avoid hyperbole**: No "robust", "comprehensive", "production-grade"
2. **Be concise**: Get to the point quickly
3. **Focus on clarity**: Simple language, clear examples
4. **Document limitations**: Be honest about what doesn't work

## Debugging Tips

1. **Use debug prints in tests**: `cargo test -- --nocapture`
2. **Check instruction syntax**: Store format differs between addressing modes
3. **Verify alignment**: Memory displays often need alignment
4. **Test incrementally**: Build parser → formatter → executor → tests

## Terminal and Binary Development

### Module Organization

1. **Binary-only features**: Keep terminal code in `src/bin/` subdirectories
   ```
   src/bin/repl/
   ├── mod.rs       # Module exports
   ├── history.rs   # Command history logic
   └── input.rs     # Terminal input handling
   ```

2. **Feature separation**: Library stays pure, binary gets rich features
   ```rust
   #[cfg(feature = "repl")]
   let mut history = repl::CommandHistory::new(history_size);
   ```

### Terminal Handling Patterns

1. **Always restore terminal state**:
   ```rust
   terminal::enable_raw_mode()?;
   let result = (|| -> io::Result<String> {
       // ... terminal interaction ...
   })();
   terminal::disable_raw_mode()?;  // Always runs
   result
   ```

2. **Use crossterm for consistency**: Avoid mixing print methods
   ```rust
   execute!(stdout, Print(prompt))?;  // Good
   print!("{}", prompt);              // Avoid in raw mode
   ```

3. **Handle line endings explicitly**: Use `\r\n` for proper terminal behavior

### CLI Configuration Best Practices

1. **Validate at boundaries**: Convert CLI args to Config early
2. **Use conflicts_with**: Prevent invalid flag combinations
3. **Thread config through**: Pass only needed values, not entire CLI struct

Example:
```rust
let history_size = if cli.no_history { 0 } else { cli.history_size };
run_interactive(&mut interpreter, cli.quiet, history_size)
```

## Common Pitfalls

1. **Store instruction syntax**: Use `SB x2, 0(x1)` not `SB x2, x1, 0`
2. **Option dereferencing**: Can't dereference `Option<T>` directly
3. **Doc test format**: Use ` ```text` for non-code examples
4. **Feature flag scope**: Remember what's gated by `#[cfg(feature = "repl")]`
5. **Terminal state**: Always restore on all exit paths (success, error, panic)
6. **Event loops**: Handle Ctrl+C gracefully with proper cleanup

## References

- RISC-V ISA Manual: `riscv-isa-manual/src/rv32.adoc`
- Test organization: `tests/TEST_COVERAGE.md`
- Implementation guide: `INSTRUCTION_IMPLEMENTATION.md`
- Task tracking: `PROJECT_STATUS.md`