//! Formatting functions for human-readable output
//!
//! This module contains functions that format CPU state, instructions, and
//! help information for display to the user.

use crate::rv32_i::{Instruction, Register, CPU};

/// Formats the result of executing an instruction with a human-readable description
pub fn format_instruction_result(instruction: &Instruction, cpu: &CPU) -> String {
    match instruction {
        Instruction::ADD(i) => {
            format!(
                "{}: Added {:?} ({}) and {:?} ({}) and stored result in {:?} ({})",
                instruction.mnemonic(),
                i.rs1,
                cpu.get_register(i.rs1),
                i.rs2,
                cpu.get_register(i.rs2),
                i.rd,
                cpu.get_register(i.rd)
            )
        }
        Instruction::ADDI(i) => {
            format!(
                "{}: Added {} to {:?} ({}) and stored result in {:?} ({})",
                instruction.mnemonic(),
                i.imm.as_i32(),
                i.rs1,
                cpu.get_register(i.rs1),
                i.rd,
                cpu.get_register(i.rd)
            )
        }
        Instruction::SUB(i) => {
            format!(
                "{}: Subtracted {:?} ({}) from {:?} ({}) and stored result in {:?} ({})",
                instruction.mnemonic(),
                i.rs2,
                cpu.get_register(i.rs2),
                i.rs1,
                cpu.get_register(i.rs1),
                i.rd,
                cpu.get_register(i.rd)
            )
        }
        Instruction::LW(i) => {
            format!(
                "{}: Loaded word from memory address 0x{:x} ({}+{}) into {:?} ({})",
                instruction.mnemonic(),
                (cpu.get_register(i.rs1) as i32 + i.imm.as_i32()) as u32,
                cpu.get_register(i.rs1),
                i.imm.as_i32(),
                i.rd,
                cpu.get_register(i.rd)
            )
        }
        Instruction::SW(i) => {
            format!(
                "{}: Stored word from {:?} ({}) to memory address 0x{:x} ({}+{})",
                instruction.mnemonic(),
                i.rs2,
                cpu.get_register(i.rs2),
                (cpu.get_register(i.rs1) as i32 + i.imm.as_i32()) as u32,
                cpu.get_register(i.rs1),
                i.imm.as_i32()
            )
        }
        Instruction::BEQ(i) => {
            if cpu.get_register(i.rs1) == cpu.get_register(i.rs2) {
                format!(
                    "{}: Branch taken: {:?} ({}) equals {:?} ({})",
                    instruction.mnemonic(),
                    i.rs1,
                    cpu.get_register(i.rs1),
                    i.rs2,
                    cpu.get_register(i.rs2)
                )
            } else {
                format!(
                    "{}: Branch not taken: {:?} ({}) does not equal {:?} ({})",
                    instruction.mnemonic(),
                    i.rs1,
                    cpu.get_register(i.rs1),
                    i.rs2,
                    cpu.get_register(i.rs2)
                )
            }
        }
        Instruction::BNE(i) => {
            if cpu.get_register(i.rs1) != cpu.get_register(i.rs2) {
                format!(
                    "{}: Branch taken: {:?} ({}) does not equal {:?} ({})",
                    instruction.mnemonic(),
                    i.rs1,
                    cpu.get_register(i.rs1),
                    i.rs2,
                    cpu.get_register(i.rs2)
                )
            } else {
                format!(
                    "{}: Branch not taken: {:?} ({}) equals {:?} ({})",
                    instruction.mnemonic(),
                    i.rs1,
                    cpu.get_register(i.rs1),
                    i.rs2,
                    cpu.get_register(i.rs2)
                )
            }
        }
        Instruction::JAL(i) => {
            format!(
                "{}: Jumped to PC+{} and stored return address in {:?} ({})",
                instruction.mnemonic(),
                i.imm.as_i32(),
                i.rd,
                cpu.get_register(i.rd)
            )
        }
        Instruction::JALR(i) => {
            format!(
                "{}: Jumped to {:?} ({}) + {} and stored return address in {:?} ({})",
                instruction.mnemonic(),
                i.rs1,
                cpu.get_register(i.rs1),
                i.imm.as_i32(),
                i.rd,
                cpu.get_register(i.rd)
            )
        }
        Instruction::LUI(i) => {
            format!(
                "{}: Loaded upper immediate {} into {:?} ({})",
                instruction.mnemonic(),
                i.imm.as_i32(),
                i.rd,
                cpu.get_register(i.rd)
            )
        }
        Instruction::AUIPC(i) => {
            format!(
                "{}: Added upper immediate {} to PC and stored in {:?} ({})",
                instruction.mnemonic(),
                i.imm.as_i32(),
                i.rd,
                cpu.get_register(i.rd)
            )
        }
        Instruction::AND(i) => {
            format!(
                "{}: Bitwise AND of {:?} ({}) and {:?} ({}) stored in {:?} ({})",
                instruction.mnemonic(),
                i.rs1,
                cpu.get_register(i.rs1),
                i.rs2,
                cpu.get_register(i.rs2),
                i.rd,
                cpu.get_register(i.rd)
            )
        }
        Instruction::OR(i) => {
            format!(
                "{}: Bitwise OR of {:?} ({}) and {:?} ({}) stored in {:?} ({})",
                instruction.mnemonic(),
                i.rs1,
                cpu.get_register(i.rs1),
                i.rs2,
                cpu.get_register(i.rs2),
                i.rd,
                cpu.get_register(i.rd)
            )
        }
        Instruction::XOR(i) => {
            format!(
                "{}: Bitwise XOR of {:?} ({}) and {:?} ({}) stored in {:?} ({})",
                instruction.mnemonic(),
                i.rs1,
                cpu.get_register(i.rs1),
                i.rs2,
                cpu.get_register(i.rs2),
                i.rd,
                cpu.get_register(i.rd)
            )
        }
        Instruction::NOP => format!("{}: No operation", instruction.mnemonic()),
        _ => {
            format!("{}: Executed instruction", instruction.mnemonic())
        }
    }
}

/// Shows all register values in a formatted table
pub fn format_all_registers(cpu: &CPU) -> String {
    let mut output = String::new();

    for i in 0..32 {
        let reg = match i {
            0 => Register::X0,
            1 => Register::X1,
            2 => Register::X2,
            3 => Register::X3,
            4 => Register::X4,
            5 => Register::X5,
            6 => Register::X6,
            7 => Register::X7,
            8 => Register::X8,
            9 => Register::X9,
            10 => Register::X10,
            11 => Register::X11,
            12 => Register::X12,
            13 => Register::X13,
            14 => Register::X14,
            15 => Register::X15,
            16 => Register::X16,
            17 => Register::X17,
            18 => Register::X18,
            19 => Register::X19,
            20 => Register::X20,
            21 => Register::X21,
            22 => Register::X22,
            23 => Register::X23,
            24 => Register::X24,
            25 => Register::X25,
            26 => Register::X26,
            27 => Register::X27,
            28 => Register::X28,
            29 => Register::X29,
            30 => Register::X30,
            31 => Register::X31,
            _ => unreachable!(),
        };

        let val = cpu.get_register(reg);
        let abi_name = get_abi_name(reg);

        if i % 2 == 0 && i < 31 {
            output.push_str(&format!("x{i:2} ({abi_name:4}): 0x{val:08x}    "));
        } else {
            output.push_str(&format!("x{i:2} ({abi_name:4}): 0x{val:08x}\n"));
        }
    }

    // Add PC at the end
    output.push_str(&format!("pc      : 0x{:08x}\n", cpu.pc));
    output
}

/// Shows specific register values
pub fn format_specific_registers(cpu: &CPU, regs: Vec<Register>) -> String {
    let mut output = String::new();

    for (idx, reg) in regs.iter().enumerate() {
        let val = cpu.get_register(*reg);
        let abi_name = get_abi_name(*reg);

        // Format register with ABI name
        let reg_str = match reg {
            Register::PC => "pc".to_string(),
            _ => {
                let reg_num = match reg {
                    Register::X0 => 0,
                    Register::X1 => 1,
                    Register::X2 => 2,
                    Register::X3 => 3,
                    Register::X4 => 4,
                    Register::X5 => 5,
                    Register::X6 => 6,
                    Register::X7 => 7,
                    Register::X8 => 8,
                    Register::X9 => 9,
                    Register::X10 => 10,
                    Register::X11 => 11,
                    Register::X12 => 12,
                    Register::X13 => 13,
                    Register::X14 => 14,
                    Register::X15 => 15,
                    Register::X16 => 16,
                    Register::X17 => 17,
                    Register::X18 => 18,
                    Register::X19 => 19,
                    Register::X20 => 20,
                    Register::X21 => 21,
                    Register::X22 => 22,
                    Register::X23 => 23,
                    Register::X24 => 24,
                    Register::X25 => 25,
                    Register::X26 => 26,
                    Register::X27 => 27,
                    Register::X28 => 28,
                    Register::X29 => 29,
                    Register::X30 => 30,
                    Register::X31 => 31,
                    Register::PC => unreachable!(),
                };
                format!("x{reg_num:2} ({abi_name:4})")
            }
        };

        // Two registers per line, or last register
        if idx % 2 == 0 && idx < regs.len() - 1 {
            output.push_str(&format!("{reg_str}: 0x{val:08x}    "));
        } else {
            output.push_str(&format!("{reg_str}: 0x{val:08x}\n"));
        }
    }

    output
}

/// Shows help information for REPL commands
pub fn format_help() -> String {
    r#"RISC-V REPL Commands:
    
Instructions:
  <instruction>    Execute RISC-V instruction (e.g., ADDI x1, x0, 10)
  
Register Inspection:
  /r, /regs        Show all registers
  /r <regs...>     Show specific registers (e.g., /r x1 x2 sp)
  
Other Commands:
  /h, /help        Show this help message
  /p, /prev, /previous  Navigate to previous state in history
  /n, /next        Navigate to next state in history
  
Examples:
  ADDI x1, x0, 42  # Load 42 into x1
  /r x1            # Show value of x1
  /r x1 x2 x3      # Show x1, x2, and x3
  /p               # Go to previous state"#
        .to_string()
}

/// Gets the ABI name for a register
fn get_abi_name(reg: Register) -> &'static str {
    match reg {
        Register::X0 => "zero",
        Register::X1 => "ra",
        Register::X2 => "sp",
        Register::X3 => "gp",
        Register::X4 => "tp",
        Register::X5 => "t0",
        Register::X6 => "t1",
        Register::X7 => "t2",
        Register::X8 => "s0",
        Register::X9 => "s1",
        Register::X10 => "a0",
        Register::X11 => "a1",
        Register::X12 => "a2",
        Register::X13 => "a3",
        Register::X14 => "a4",
        Register::X15 => "a5",
        Register::X16 => "a6",
        Register::X17 => "a7",
        Register::X18 => "s2",
        Register::X19 => "s3",
        Register::X20 => "s4",
        Register::X21 => "s5",
        Register::X22 => "s6",
        Register::X23 => "s7",
        Register::X24 => "s8",
        Register::X25 => "s9",
        Register::X26 => "s10",
        Register::X27 => "s11",
        Register::X28 => "t3",
        Register::X29 => "t4",
        Register::X30 => "t5",
        Register::X31 => "t6",
        Register::PC => "pc",
    }
}
