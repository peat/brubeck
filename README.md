# Brubeck

Brubeck is a RISC-V assembly language REPL and emulator library designed for learning RISC-V architecture and assembly programming. It includes a parser with validation and helpful error messages.

Feedback and suggestions for this RISC-V assembly playground are welcome.

## Current State

### ✅ **Complete RV32I + CSR Implementation**
* **47 RV32I instructions**: All base integer instructions with comprehensive testing
* **6 CSR instructions**: CSRRW, CSRRS, CSRRC, CSRRWI, CSRRSI, CSRRCI with standard CSRs
* **8 pseudo-instructions**: MV, NOT, LI, J, JR, RET, SEQZ, SNEZ for convenience
* **System instructions**: FENCE, ECALL, EBREAK for system-level operations

### 🎓 **Features**
* Parser with helpful error messages
* Standard RISC-V syntax: `LW x1, 4(x2)` with backward compatibility
* Documentation explaining implementation concepts
* **Multiple formats**: Hex (0x), binary (0b), and decimal immediates
* **History navigation** (`/previous`, `/next`): Step back and forward with detailed change info
* **Enhanced output formatting**:
  - Color-coded registers: Changed values in green, zeros in gray
  - Color-coded memory: Changed bytes in green, PC location highlighted
  - Detailed history navigation shows exact changes (e.g., "x2: 100 → 0")
  - Consistent columnar alignment for easy reading
* **Context-aware error messages**: Helpful tips for common mistakes
* CLI support: Script files, one-liners, and execution traces

### 🧪 **Testing**
* 350+ tests covering all instructions and edge cases
* Validation for immediates, registers, and instruction formats
* Structured test organization

Implementation strictly follows the RISC-V ISA specification (see `riscv-isa-manual/src/rv32.adoc`).

## REPL Commands

* `/regs` or `/r` - Show all registers (changed values in green, zeros in gray)
* `/regs x1 x2` - Show specific registers
* `/memory` or `/m` - Show memory around PC (changed bytes in green, PC highlighted)
* `/memory 0x100` - Show memory starting at address
* `/memory 0x100 0x200` - Show memory range
* `/previous` or `/p` - Navigate to previous state (shows what changed)
* `/next` or `/n` - Navigate to next state (shows what changed)  
* `/reset` - Reset CPU state (with confirmation)
* `/help` or `/h` - Show help
* `/quit` - Exit REPL

## Examples

### Interactive REPL
```
$ cargo run

Brubeck: A RISC-V REPL
Ctrl-C to quit

[0x00000000]> ADDI x1, zero, 100
● X1: 0 → 100, PC: 0x00000000 → 0x00000004

[0x00000004]> ADDI x2, zero, 50
● X2: 0 → 50, PC: 0x00000004 → 0x00000008

[0x00000008]> ADD x3, x1, x2
● X3: 0 → 150, PC: 0x00000008 → 0x0000000c

[0x0000000c]> /regs x3
● x3 (gp): 0x00000096 (        150)

[0x0000000c]> /p
● Navigated back: x3: 150 → 0, PC: 0x0000000c → 0x00000008

[0x00000008]> /regs x3
● x3 (gp): 0x00000000 (          0)
```

### Command-Line Usage
```bash
# Quick calculations
$ brubeck -e "ADDI x1, x0, 42; SLLI x2, x1, 2; /r x2"
x2 (sp): 0x000000a8 (        168)

# Run a script file
$ brubeck -s program.bru
X3: 500 (0x1f4)

# Verbose mode for learning
$ brubeck -s program.bru --verbose
ADDI x1, x0, 100     # 0x00000000 ADDI: Added 100 to X0 (0) and stored result in X1 (100)
SLLI x2, x1, 2       # 0x00000004 SLLI: x2 ← x1 << 2 = 400
ADD x3, x1, x2       # 0x00000008 ADD: x3 ← x1 + x2 = 100 + 400 = 500
x3 (gp): 0x000001f4 (        500)

# Custom memory size
$ brubeck -m 64k

# Disable history tracking for minimal overhead
$ brubeck --no-undo
```

### Error Messages
```
ADDI x1, zero, 5000     # Out of range immediate
=> ❌ Immediate value 5000 out of range for ADDI (valid range: -2048 to 2047)
💡 Tip: I-type immediates are 12-bit signed values. For larger values, use LUI + ADDI pattern

SLLI x1, x2, 50         # Invalid shift amount
=> ❌ Immediate value 50 out of range for SLLI (valid range: 0-31)
💡 Tip: Shift amounts must be 0-31 since RISC-V registers are 32 bits
```

## Getting Started

```bash
# Clone the repository
git clone https://github.com/peat/brubeck.git
cd brubeck

# Run the REPL
cargo run

# Run with custom memory
cargo run -- -m 256k

# Execute one-liners
cargo run -- -e "LI x1, 0x1234; /regs x1"

# Run tests
cargo test

# Build documentation
cargo doc --open
```

## Command-Line Options

```
Usage: brubeck [OPTIONS]

Options:
  -m, --memory <SIZE>      Memory size (e.g., 1M, 256k, 1024) [default: 1M]
      --undo-limit <N>     Maximum history depth [default: 1000]
      --no-undo            Disable history navigation
  -e, --execute <CMDS>     Execute commands and exit (semicolon-separated)
  -s, --script <FILE>      Execute script file and exit
  -q, --quiet              Suppress banner and descriptions (REPL only)
  -v, --verbose            Show instruction trace (script/execute only)
  -h, --help               Print help
  -V, --version            Print version
```

## Future Enhancements

* **RISC-V Extensions**: Add M (multiplication/division), A (atomic), F/D (floating-point) extensions
* **Advanced REPL**: Tab completion, syntax highlighting
* **Debugging Features**: Breakpoints, step execution, execution tracing
* **Educational Tools**: Instruction encoding display, pipeline visualization
* **Assembly Features**: Labels, expressions, assembler directives (.text, .data, etc.)

## Contact

Find me on [Bluesky](https://bsky.app/profile/peat.org), [Mastodon](https://mastodon.social/@peat), [Twitter](https://twitter.com/peat), or via [email](mailto:peat@peat.org).
