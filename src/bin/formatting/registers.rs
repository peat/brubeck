//! Register display formatter

use brubeck::rv32_i::{Register, CPU};

/// Formats all registers in a table format
pub fn format_registers(cpu: &CPU, use_abi_names: bool) -> String {
    let mut output = String::new();

    // Format general purpose registers in two columns
    for i in 0..32 {
        let reg = register_from_index(i);
        let val = cpu.get_register(reg);
        let abi_name = get_abi_name(reg);

        let reg_str = if use_abi_names && abi_name != "----" {
            format!("x{i:2} ({abi_name:4})")
        } else {
            format!("x{i:2}      ")
        };

        let val_str = format!("0x{val:08x} ({val:11})", val = val as i32);

        if i % 2 == 0 && i < 31 {
            // Left column
            output.push_str(&format!("{reg_str}: {val_str}  "));
        } else {
            // Right column or last register
            output.push_str(&format!("{reg_str}: {val_str}\n"));
        }
    }

    // Add PC
    output.push_str(&format!("pc       : 0x{:08x}\n", cpu.pc));

    output
}

/// Formats specific registers
pub fn format_specific_registers(cpu: &CPU, registers: &[Register]) -> String {
    let mut output = String::new();

    for reg in registers {
        let val = cpu.get_register(*reg);
        let abi_name = get_abi_name(*reg);

        let reg_str = if abi_name != "----" {
            format!("{:?} ({})", reg, abi_name)
        } else {
            format!("{:?}", reg)
        };

        output.push_str(&format!(
            "{}: 0x{:08x} ({})\n",
            reg_str,
            val,
            val as i32
        ));
    }

    // Add PC if requested
    if registers.contains(&Register::PC) {
        output.push_str(&format!("PC: 0x{:08x}\n", cpu.pc));
    }

    output
}

/// Formats a single register value
pub fn format_register_value(reg: Register, value: u32) -> String {
    let abi_name = get_abi_name(reg);
    
    if abi_name != "----" {
        format!("{:?} ({}): 0x{:08x} ({})", reg, abi_name, value, value as i32)
    } else {
        format!("{:?}: 0x{:08x} ({})", reg, value, value as i32)
    }
}

/// Get register from index
fn register_from_index(i: u32) -> Register {
    match i {
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
        _ => panic!("Invalid register index: {}", i),
    }
}

/// Get ABI name for a register
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
        Register::PC => "----",
    }
}