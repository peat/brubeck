//! Register display formatter

use brubeck::rv32_i::{Register, StateDelta, CPU};
use crossterm::style::Stylize;

/// Formats all registers in a table format with optional coloring based on changes
///
/// This function displays all 32 general-purpose registers and the PC in a two-column
/// layout. When a StateDelta is provided, it will color-code the output:
/// - **Green**: Registers that changed in the last instruction
/// - **Dark gray**: Registers with zero values
/// - **Normal**: All other registers
///
/// # Arguments
/// * `cpu` - The CPU state to display
/// * `use_abi_names` - Whether to show ABI names (ra, sp, etc.) alongside register numbers
/// * `last_delta` - Optional state changes from the last instruction for highlighting
pub fn format_registers_with_colors(
    cpu: &CPU,
    use_abi_names: bool,
    last_delta: Option<&StateDelta>,
) -> String {
    let mut output = String::new();

    // Build a set of changed registers for quick lookup
    let changed_regs: std::collections::HashSet<Register> = last_delta
        .map(|delta| {
            delta
                .register_changes
                .iter()
                .map(|(reg, _, _)| *reg)
                .collect()
        })
        .unwrap_or_default();

    // Format general purpose registers in two columns
    // Left column: x0-x15, Right column: x16-x31
    for row in 0..16 {
        // Left column (x0-x15)
        let i = row;
        let reg = register_from_index(i);
        let val = cpu.get_register(reg);
        let abi_name = get_abi_name(reg);

        let reg_str = if use_abi_names && abi_name != "----" {
            format!("x{i:2} ({abi_name:4})")
        } else {
            format!("x{i:2}      ")
        };

        // Format the value with color
        let val_str = if val == 0 {
            // Gray for zero values
            format!("0x{val:08x} ({val:11})", val = val as i32)
                .dark_grey()
                .to_string()
        } else if changed_regs.contains(&reg) {
            // Green for changed values
            format!("0x{val:08x} ({val:11})", val = val as i32)
                .green()
                .to_string()
        } else {
            // Normal color for others
            format!("0x{val:08x} ({val:11})", val = val as i32)
        };

        output.push_str(&format!("{reg_str}: {val_str}  "));

        // Right column (x16-x31)
        let i = row + 16;
        let reg = register_from_index(i);
        let val = cpu.get_register(reg);
        let abi_name = get_abi_name(reg);

        let reg_str = if use_abi_names && abi_name != "----" {
            format!("x{i:2} ({abi_name:4})")
        } else {
            format!("x{i:2}      ")
        };

        // Format the value with color
        let val_str = if val == 0 {
            // Gray for zero values
            format!("0x{val:08x} ({val:11})", val = val as i32)
                .dark_grey()
                .to_string()
        } else if changed_regs.contains(&reg) {
            // Green for changed values
            format!("0x{val:08x} ({val:11})", val = val as i32)
                .green()
                .to_string()
        } else {
            // Normal color for others
            format!("0x{val:08x} ({val:11})", val = val as i32)
        };

        output.push_str(&format!("{reg_str}: {val_str}\n"));
    }

    // Add PC with coloring if it changed
    let pc_changed = last_delta
        .map(|delta| delta.pc_change.0 != delta.pc_change.1)
        .unwrap_or(false);

    let pc_str = if pc_changed {
        format!("0x{:08x}", cpu.pc).green().to_string()
    } else {
        format!("0x{:08x}", cpu.pc)
    };

    output.push_str(&format!("pc       : {pc_str}\n"));

    output
}

/// Formats specific registers
pub fn format_specific_registers(cpu: &CPU, registers: &[Register]) -> String {
    let mut output = String::new();

    for reg in registers {
        let val = cpu.get_register(*reg);
        let abi_name = get_abi_name(*reg);

        let reg_str = if abi_name != "----" {
            format!("{} ({})", format_register_name(*reg), abi_name)
        } else {
            format_register_name(*reg)
        };

        output.push_str(&format!("{}: 0x{:08x} ({})\n", reg_str, val, val as i32));
    }

    // Add PC if requested
    if registers.contains(&Register::PC) {
        output.push_str(&format!("PC: 0x{:08x}\n", cpu.pc));
    }

    output
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
        _ => panic!("Invalid register index: {i}"),
    }
}

/// Format register name consistently
fn format_register_name(reg: Register) -> String {
    match reg {
        Register::X0 => "x 0".to_string(),
        Register::X1 => "x 1".to_string(),
        Register::X2 => "x 2".to_string(),
        Register::X3 => "x 3".to_string(),
        Register::X4 => "x 4".to_string(),
        Register::X5 => "x 5".to_string(),
        Register::X6 => "x 6".to_string(),
        Register::X7 => "x 7".to_string(),
        Register::X8 => "x 8".to_string(),
        Register::X9 => "x 9".to_string(),
        Register::X10 => "x10".to_string(),
        Register::X11 => "x11".to_string(),
        Register::X12 => "x12".to_string(),
        Register::X13 => "x13".to_string(),
        Register::X14 => "x14".to_string(),
        Register::X15 => "x15".to_string(),
        Register::X16 => "x16".to_string(),
        Register::X17 => "x17".to_string(),
        Register::X18 => "x18".to_string(),
        Register::X19 => "x19".to_string(),
        Register::X20 => "x20".to_string(),
        Register::X21 => "x21".to_string(),
        Register::X22 => "x22".to_string(),
        Register::X23 => "x23".to_string(),
        Register::X24 => "x24".to_string(),
        Register::X25 => "x25".to_string(),
        Register::X26 => "x26".to_string(),
        Register::X27 => "x27".to_string(),
        Register::X28 => "x28".to_string(),
        Register::X29 => "x29".to_string(),
        Register::X30 => "x30".to_string(),
        Register::X31 => "x31".to_string(),
        Register::PC => "pc".to_string(),
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
