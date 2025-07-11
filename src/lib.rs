//! # Brubeck: A RISC-V REPL in Progress
//!
//! Brubeck provides a "read, evaluate, print, loop" interactive environment for
//! RISC-V assembly language.
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
//! Basic arithmetic operations are functional. More complex features are still
//! in development.
//!
//! ## Running
//!
//! `brubeck` (or `cargo run`) opens a REPL for a minimal RV32I RISC-V emulated
//! processor (single hardware thread, one mebibyte of memory).
//!
//! To see the contents of a register, just type in it's name (eg: `x2` or
//! `sp` if you prefer the [ABI](crate::rv32_i::ABI) name). Memory examination
//! is not yet available through the REPL interface.
//!
//! To execute an instruction, type in its name and arguments (eg: `nop` or `addi x2, x0, 5`).
//!
//! The complete RV32I instruction set is implemented, including system
//! instructions (FENCE, ECALL, EBREAK). Common pseudo-instructions are also
//! supported (MV, NOT, SEQZ, SNEZ, J, JR, RET, LI).
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
//! // NOP, or "No Operation"
//! let nop = Instruction::NOP;
//! let result = cpu.execute(nop);
//!
//! // Check for successful execution
//! assert!(result.is_ok());
//!
//! // PC should be incremented by the length of the NOP instruction
//! assert_eq!(cpu.pc, Instruction::LENGTH);
//!
//! // Set a register to a value, then store it in memory
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
//! // Execute the instruction
//! let result = cpu.execute(addi);
//!
//! // ... Any execution errors will be caught.
//! assert!(result.is_ok());
//!
//! // The result is a StateDelta showing what changed
//! let delta = result.unwrap();
//! assert_eq!(delta.register_changes.len(), 1);
//!
//! // Verify the register was updated
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
//! // Execute the instruction
//! let result = cpu.execute(sw);
//!
//! // ... Any execution errors will be caught.
//! assert!(result.is_ok());
//!
//! // The delta shows memory was changed
//! let delta = result.unwrap();
//! assert!(!delta.memory_changes.is_empty());
//!
//! // Let's check the address
//! assert_eq!(cpu.memory[255], 1);
//!
//! ```
//!

pub mod interpreter;
pub mod rv32_i;

pub use interpreter::Interpreter;
pub use rv32_i::{Immediate, PseudoInstruction};
