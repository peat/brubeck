# Brubeck

A RISC-V assembly REPL for learning and experimentation. Type instructions, see what happens. Step backwards if you mess up.

Brubeck implements the RV32I base integer instruction set (the 32-bit version of RISC-V). It's written in Rust and can be used as a library or through the interactive REPL.

## Quick Start

```bash
cargo run
```

```
Brubeck: A RISC-V REPL
Ctrl-C to quit

[0x00000000]> ADDI x1, zero, 100
x1: 0 → 100, PC: 0x00000000 → 0x00000004

[0x00000004]> ADD x2, x1, x1
x2: 0 → 200, PC: 0x00000004 → 0x00000008

[0x00000008]> /regs x1 x2
x1 (ra): 0x00000064 (        100)
x2 (sp): 0x000000c8 (        200)

[0x00000008]> /p
Navigated back: x2: 200 → 0, PC: 0x00000008 → 0x00000004
```

## What's Implemented

**47 RV32I instructions** — all the base integer operations: arithmetic, logic, shifts, branches, jumps, loads, stores, and upper immediates.

**6 CSR instructions** — CSRRW, CSRRS, CSRRC and their immediate variants for reading/writing control registers.

**8 pseudo-instructions** — MV, NOT, LI, J, JR, RET, SEQZ, SNEZ. These expand to real instructions but are convenient shortcuts.

The implementation follows the RISC-V ISA specification (included in `riscv-isa-manual/`).

## REPL Commands

| Command | What it does |
|---------|--------------|
| `/regs` or `/r` | Show all registers |
| `/regs x1 x2` | Show specific registers |
| `/memory` or `/m` | Show memory around PC |
| `/memory 0x100` | Show memory at address |
| `/memory 0x100 0x200` | Show memory range |
| `/previous` or `/p` | Step backward through history |
| `/next` or `/n` | Step forward through history |
| `/reset` | Clear everything and start over |
| `/help` or `/h` | Show help |
| `/quit` or `/q` | Exit |

## Command Line

```bash
# One-liner
brubeck -e "ADDI x1, x0, 42; SLLI x2, x1, 2; /r x2"

# Run a script
brubeck -s program.bru

# Verbose mode shows what each instruction does
brubeck -s program.bru --verbose

# Custom memory size
brubeck -m 64k

# See all options
brubeck --help
```

## Error Messages

Brubeck tries to be helpful when things go wrong:

```
[0x00000000]> ADDI x1, zero, 5000
Immediate value 5000 out of range for ADDI (valid range: -2048 to 2047)

[0x00000000]> ADDD x1, x0, 5
Unknown instruction 'ADDD'. Did you mean 'ADD or ADDI'?
```

Run with `--tips` for additional context about RISC-V concepts.

## Using as a Library

The emulator is a separate library with no dependencies, so you can embed it in other projects or compile to WASM.

```rust
use brubeck::{Interpreter, CPU};

let mut interp = Interpreter::new();
let delta = interp.interpret("ADDI x1, x0, 42")?;
// delta contains what changed: registers, PC, memory

let value = interp.cpu.get_register(Register::X1);
```

## Building

```bash
cargo build          # debug build
cargo build --release
cargo test           # run the tests
cargo doc --open     # browse the docs
```

## Not Yet Implemented

- M extension (multiply/divide)
- F/D extensions (floating point)
- Breakpoints and watchpoints
- Labels and assembler directives
- Tab completion

## Contact

[Bluesky](https://bsky.app/profile/peat.org) · [Mastodon](https://mastodon.social/@peat) · [Twitter](https://twitter.com/peat) · [Email](mailto:peat@peat.org)
