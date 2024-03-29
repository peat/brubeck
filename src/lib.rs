//! # Brubeck: A RISC-V REPL in Progress
//!
//! Brubeck provides a "read, evaluate, print, loop" interactive environment for
//! RISC-V assembly language. What could be more fun?!
//!
//! From Wikipedia: _"[Dave Brubeck](https://en.wikipedia.org/wiki/Dave_Brubeck)
//! was an American jazz pianist and composer. Often regarded as a foremost
//! exponent of cool jazz, Brubeck's work is characterized by unusual time
//! signatures and superimposing contrasting rhythms, meters, and tonalities.
//! [His song] "Take Five" became the highest-selling jazz single of all time."_
//!
//! ## HERE BE DRAGONS
//!
//! Don't expect this to work out of the box. There is a lot of nuance in the
//! behavior of an ISA, particularly with error modes, addressing, etc.
//!
//! The very basic stuff works great. Go ahead and ADD and SUB to your hart's
//! content. Don't rely on it for anything else quite yet.
//!
//! ## Running
//!
//! `brubeck` (or `cargo run`) opens a REPL for a minimal RV32I RISC-V emulated
//! processor (single hardware thread, one mebibyte of memory).
//!
//! To see the contents of a register, just type in it's name (eg: `x2` or
//! `sp` if you prefer the [ABI](crate::rv32_i::ABI) name). To examine a
//! region in memory ... well, that's TBD.
//!
//! To execute an instruction, type in its name and arguments (eg: `nop` or `addi x2, x0, 5`).
//!
//! The majority of the RV32I instruction set is implemented, with a couple of
//! exceptions (eg: EBREAK, ECALL, EFENCE).
//!
//! For information about the implementation, see the [Interpreter](crate::interpreter).
//!
//! ## The Library
//!
//! The goal of the library is simplicity and observabilty, not performance.
//!
//! Dive into [`CPU`](crate::rv32_i::CPU) to see how it works, particularly
//! the `execute()` function.
//!
//! ```
//! use brubeck::rv32_i::*;
//!
//! let mut cpu = CPU::default();
//!
//! // NOP, or "No Operation" ... the simplest instruction!
//! let nop = Instruction::NOP;
//! let result = cpu.execute(nop);
//!
//! // successful execution is ok!
//! assert!(result.is_ok());
//!
//! // PC should be incremented by the length of the NOP instruction
//! assert_eq!(cpu.pc, Instruction::LENGTH);
//!
//! // Let's do something more exciting: set a register to a value, then
//! // store it in memory!
//!
//! // The xType formats represent the instruction data components.
//! let mut addi_data = IType::default();
//!
//! // ... Set the destination register
//! addi_data.rd = Register::X1;
//!
//! // ... Set the addend register; the ABI Zero register (x0) contains zero
//! addi_data.rs1 = ABI::Zero.to_register();
//!
//! // ... Set the 12-bit immediate value to load.
//! let result = addi_data.imm.set_unsigned(1);
//!
//! // ... Immediate values are bounds checked.
//! assert!(result.is_ok());
//!
//! // ... Create the instruction.
//! let addi = Instruction::ADDI(addi_data);
//!
//! // ... Aaaaand execute it!
//! let result = cpu.execute(addi);
//!
//! // ... Any execution errors will be caught.
//! assert!(result.is_ok());
//!
//! // ... The target register responds appropriately to ADDI!
//! assert_eq!(cpu.x1, 0b0000_0000_0000_0000_0000_0000_0000_0001);
//!
//! // And now we store it in memory ...
//!
//! // ... Put the address directly into register x2
//! cpu.x2 = 255;
//!
//! // ... Now set up the SW instruction.
//! let mut sw_data = SType::default();
//!
//! // ... rs2 indicates the register containing the source value
//! sw_data.rs2 = Register::X1;
//!
//! // ... rs1 indicates the register containing the address
//! sw_data.rs1 = Register::X2;
//!
//! // ... Create the instruction.
//! let sw = Instruction::SW(sw_data);
//!
//! // ... Aaaaand execute it!
//! let result = cpu.execute(sw);
//!
//! // ... Any execution errors will be caught.
//! assert!(result.is_ok());
//!
//! // Let's check the address
//! assert_eq!(cpu.memory[255], 1);
//!
//! ```
//!

/// Provides immediate value checks, conversions, etc.
mod immediate;

pub mod interpreter;
pub mod rv32_i;

pub use immediate::Immediate;
pub use interpreter::Interpreter;
