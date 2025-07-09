# Brubeck!

The goal of Brubeck is to create a REPL for RISC-V assembly language, and a very easy to use emulator library -- a playground for learning, not a high performance emulator.

This is a _very_ early prototype. See the [brubeck crate documentation](https://docs.rs/brubeck/) for more information about running  the REPL and working with the library.

Please follow this repo if you're interested in the project! I'm also very keen on feedback and thoughts on what you think would be awesome to see in a RISC-V assembly playground.

## Current State

* Emulator covers the complete RV32I instruction set, including `EBREAK`, `ECALL`, and `FENCE` instructions.
* Interpreter can evaluate instructions (eg: `ADD x1, x2, x3`), pseudo-instructions (eg: `LI x1, -5`), and inspect registers.
* Parser supports hex (0x), binary (0b), and decimal immediate values.
* Implementation follows the RISC-V ISA specification (see `riscv-isa-manual/src/rv32.adoc`).

## Example

The example below demonstrates using the `ADDI` and `ADD` instructions as well as inspecting the state of registers.

```
$ cargo run

Brubeck: A RISC-V REPL
Ctrl-C to quit

ADDI x1, x0, 5
=> ✅ ADDI(IType { opcode: 0, rd: X1, funct3: 0, rs1: X0, imm: Immediate { value: 5, bits: 12 } })
x1
=> ✅ X1: 5 (0x5)
ADDI x2, x0, 3
=> ✅ ADDI(IType { opcode: 0, rd: X2, funct3: 0, rs1: X0, imm: Immediate { value: 3, bits: 12 } })
x2
=> ✅ X2: 3 (0x3)
ADD x3, x2, x1
=> ✅ ADD(RType { opcode: 0, rd: X3, funct3: 0, rs1: X2, rs2: X1, funct7: 0 })
x3
=> ✅ X3: 8 (0x8)
```

## TODO

* Add CSR (Control and Status Register) instructions
* Add memory inspection commands
* Improve parser error messages and validation

## Contact

Find me on [Bluesky](https://bsky.app/profile/peat.org), [Mastodon](https://mastodon.social/@peat), [Twitter](https://twitter.com/peat), or via [email](mailto:peat@peat.org).
