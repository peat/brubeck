# CLAUDE.md

Guidance for Claude Code when working with this repository.

## Rules

1. **Write tests first** (TDD) before implementing features
2. **Run quality checks before commits**: `cargo fmt && cargo clippy -- -D warnings && cargo test`
3. **Keep the library pure** - no dependencies, no I/O, no formatting (see Architecture)

## Project Overview

Brubeck is a RISC-V RV32I emulator written in Rust. It provides:
- A **library** for CPU emulation (zero dependencies, WASM-ready)
- A **binary** for an interactive REPL with colors and history

Reference: RISC-V ISA Manual in `riscv-isa-manual/src/rv32.adoc`

## Quick Reference

```bash
cargo run                           # Interactive REPL
cargo run -- -e "ADDI x1, x0, 42"   # One-liner
cargo test                          # All tests
cargo test test_name                # Specific test
cargo test -- --nocapture           # Show println! output
```

### Key Directories

```
src/rv32_i/          # CPU emulation (registers, memory, instructions)
src/interpreter/     # Parsing and execution (returns StateDelta)
src/bin/             # REPL binary (formatting, commands, terminal)
src/bin/formatting/  # Color-coded output formatters
tests/unit/          # Unit tests organized by component
tests/common/        # Shared test helpers
```

### Key Files

| File | Purpose |
|------|---------|
| `src/rv32_i/cpu.rs` | CPU state: 32 registers, memory, CSRs |
| `src/interpreter/parser.rs` | 4-phase parsing pipeline |
| `src/interpreter/types.rs` | Command, Token, Error, StateDelta |
| `src/bin/repl_commands.rs` | REPL slash command handling |
| `src/bin/formatting/state_delta.rs` | Instruction result formatting |

## Architecture

### Library/Binary Separation (Critical)

The library returns **data**, the binary handles **presentation**.

```rust
// LIBRARY (src/interpreter/): Returns structured data
pub fn interpret(&mut self, input: &str) -> Result<StateDelta, Error>

// StateDelta contains:
// - register_changes: Vec<(Register, old_value, new_value)>
// - pc_change: (old_pc, new_pc)
// - memory_changes: Vec<(addr, old_byte, new_byte)>

// BINARY (src/bin/): Formats for display
let delta = interpreter.interpret("ADDI x1, x0, 100")?;
let output = formatting::state_delta::format(&delta, &interpreter.cpu);
println!("{}", output);
```

### What Goes Where

| Library (`src/`) | Binary (`src/bin/`) |
|------------------|---------------------|
| CPU emulation | Terminal I/O |
| Instruction parsing | Color formatting |
| StateDelta creation | REPL commands (`/regs`, `/memory`) |
| State history navigation | Command history (arrow keys) |
| Error types | Error formatting with tips |

### Do NOT

- Add dependencies to the library (breaks WASM/embedded use)
- Return formatted strings from library code
- Put REPL commands in the library
- Parse strings in the binary that the library should handle

## Implemented Features

### Instructions (47 RV32I + 6 CSR + 8 pseudo)

- **Arithmetic**: ADD, ADDI, SUB
- **Logical**: AND, ANDI, OR, ORI, XOR, XORI
- **Shifts**: SLL, SLLI, SRL, SRLI, SRA, SRAI
- **Compare**: SLT, SLTI, SLTU, SLTIU
- **Branch**: BEQ, BNE, BLT, BGE, BLTU, BGEU
- **Jump**: JAL, JALR
- **Load**: LB, LH, LW, LBU, LHU
- **Store**: SB, SH, SW
- **Upper**: LUI, AUIPC
- **System**: FENCE, ECALL, EBREAK, NOP
- **CSR**: CSRRW, CSRRS, CSRRC, CSRRWI, CSRRSI, CSRRCI
- **Pseudo**: MV, NOT, SEQZ, SNEZ, J, JR, RET, LI

### REPL Commands

| Command | Aliases | Description |
|---------|---------|-------------|
| `/regs` | `/r` | Show registers (with color) |
| `/memory [addr] [end]` | `/m` | Inspect memory |
| `/previous` | `/prev`, `/p` | Navigate back in history |
| `/next` | `/n` | Navigate forward in history |
| `/reset` | | Reset CPU (with confirmation) |
| `/help` | `/h` | Show help |
| `/quit` | `/q` | Exit |

## Testing

### Test Structure

```
tests/
├── unit_tests.rs              # Entry point
├── unit/
│   ├── instructions/          # Per-instruction tests
│   │   ├── arithmetic.rs
│   │   ├── branches.rs
│   │   ├── csr.rs
│   │   └── ...
│   ├── components/            # CSR state, immediates
│   └── history.rs             # State navigation
├── history_navigation.rs      # Integration tests
├── common/                    # Test helpers (mod common;)
└── ...
```

### Test Patterns

```rust
// Unit test: Direct CPU manipulation
let mut cpu = CPU::default();
cpu.set_register(Register::X1, 10);
let inst = ADDI { rd: Register::X2, rs1: Register::X1, imm: 5 };
let delta = cpu.execute(inst).unwrap();
assert_eq!(cpu.get_register(Register::X2), 15);

// Integration test: Through interpreter
let mut interp = Interpreter::new();
interp.interpret("ADDI x1, x0, 100").unwrap();
assert_eq!(interp.cpu.get_register(Register::X1), 100);

// Test state navigation
interp.previous_state().unwrap();
assert_eq!(interp.cpu.get_register(Register::X1), 0);
```

## Code Patterns

### Instruction Syntax

Both syntaxes are supported for loads/stores:
```
LW x1, 0(x2)      # Standard: offset(base)
LW x1, x2, 0      # Legacy: base, offset
```

### Error Handling

Library errors are structured with context for educational formatting:
```rust
// Parser provides suggestions via fuzzy matching
Error::UnknownInstruction { name: "ADDD", suggestion: Some("ADD") }

// Binary formats with color and tips
"Unknown instruction 'ADDD'. Did you mean 'ADD'?"
```

### Terminology

Use "previous/next" not "undo/redo" - we're navigating history, not reverting actions:
- Commands: `/previous`, `/next`
- Methods: `previous_state()`, `next_state()`
- CLI flags: `--history-limit`, `--no-state-history`

## Known Limitations

- CSR cycle counters (`CYCLE`, `TIME`, `INSTRET`) return 0 (stubs in `src/rv32_i/cpu.rs`)
- No tab completion
- No `/history` command to show execution log
- No breakpoints or watchpoints

## Common Pitfalls

1. **Store syntax**: Use `SB x2, 0(x1)` not `SB x2, x1, 0` (both work, but standard is preferred)
2. **Terminal state**: Always restore terminal mode on all exit paths
3. **Test output**: Use `cargo test -- --nocapture` to see println! in tests

## Future Work

- Structured error types in library (no string parsing for consumers)
- Tab completion
- `/history` command to show execution log
- M extension (multiply/divide)
- WASM/web version
