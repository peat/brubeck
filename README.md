# Brubeck!

Brubeck is a **teaching-focused** RISC-V assembly language REPL and emulator library. It's designed as a playground for learning RISC-V architecture, assembly programming, and compiler construction - prioritizing educational value over performance.

The project features a **production-grade parser** with comprehensive validation and educational error messages, making it an excellent resource for students and educators.

Please follow this repo if you're interested in the project! I'm also very keen on feedback and thoughts on what you think would be awesome to see in a RISC-V assembly playground.

## Current State

### ✅ **Complete RV32I + CSR Implementation**
* **47 RV32I instructions**: All base integer instructions with comprehensive testing
* **6 CSR instructions**: CSRRW, CSRRS, CSRRC, CSRRWI, CSRRSI, CSRRCI with standard CSRs
* **8 pseudo-instructions**: MV, NOT, LI, J, JR, RET, SEQZ, SNEZ for convenience
* **System instructions**: FENCE, ECALL, EBREAK for system-level operations

### 🎓 **Teaching-Focused Parser**
* **Production-grade validation** with educational error messages
* **Standard RISC-V syntax**: `LW x1, 4(x2)` with backward compatibility
* **Comprehensive documentation** explaining compiler concepts
* **Rich error messages** with contextual tips and RISC-V education
* **Multiple formats**: Hex (0x), binary (0b), and decimal immediates

### 🧪 **Robust Testing**
* **350+ tests** covering all instructions and edge cases
* **Comprehensive validation** for immediates, registers, and instruction formats
* **Educational test structure** demonstrating testing best practices

Implementation strictly follows the RISC-V ISA specification (see `riscv-isa-manual/src/rv32.adoc`).

## Examples

### Basic Arithmetic
```
$ cargo run

Brubeck: A RISC-V REPL
Ctrl-C to quit

ADDI x1, zero, 100      # Load immediate value
=> ✅ ADDI(IType { opcode: 0, rd: X1, funct3: 0, rs1: X0, imm: Immediate { value: 100, bits: 12 } })

ADDI x2, zero, 50       # Another immediate
=> ✅ ADDI(IType { opcode: 0, rd: X2, funct3: 0, rs1: X0, imm: Immediate { value: 50, bits: 12 } })

ADD x3, x1, x2          # Add registers
=> ✅ ADD(RType { opcode: 0, rd: X3, funct3: 0, rs1: X1, rs2: X2, funct7: 0 })

x3                      # Inspect result
=> ✅ X3: 150 (0x96)
```

### Standard RISC-V Load/Store Syntax
```
ADDI x1, zero, 0x1000   # Base address
=> ✅ ADDI(IType { opcode: 0, rd: X1, funct3: 0, rs1: X0, imm: Immediate { value: 4096, bits: 12 } })

SW x3, 4(x1)            # Store word with offset
=> ✅ SW(SType { opcode: 0, funct3: 0, rs1: X1, rs2: X3, imm: Immediate { value: 4, bits: 12 } })

LW x4, 4(x1)            # Load word with offset
=> ✅ LW(IType { opcode: 0, rd: X4, funct3: 0, rs1: X1, imm: Immediate { value: 4, bits: 12 } })
```

### CSR Instructions
```
CSRRW x1, MSCRATCH, x2  # Swap register with CSR
=> ✅ CSRRW(IType { opcode: 0, rd: X1, funct3: 0, rs1: X2, imm: Immediate { value: 832, bits: 12 } })

CSRRS x1, MSTATUS, x0   # Read machine status
=> ✅ CSRRS(IType { opcode: 0, rd: X1, funct3: 0, rs1: X0, imm: Immediate { value: 768, bits: 12 } })
```

### Educational Error Messages
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
git clone https://github.com/username/brubeck.git
cd brubeck

# Run the REPL
cargo run

# Run tests
cargo test

# Build documentation
cargo doc --open
```

## Future Enhancements

* **RISC-V Extensions**: Add M (multiplication/division), A (atomic), F/D (floating-point) extensions
* **Advanced REPL**: Command history, tab completion, syntax highlighting
* **Debugging Features**: Breakpoints, step execution, execution tracing
* **Educational Tools**: Instruction encoding display, pipeline visualization
* **Assembly Features**: Labels, expressions, assembler directives (.text, .data, etc.)

## Contact

Find me on [Bluesky](https://bsky.app/profile/peat.org), [Mastodon](https://mastodon.social/@peat), [Twitter](https://twitter.com/peat), or via [email](mailto:peat@peat.org).
