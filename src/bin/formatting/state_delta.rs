//! Formatter for StateDelta - showing CPU state changes

use brubeck::rv32_i::{Register, StateDelta};

/// Format register name in lowercase (x0, x1, etc.)
fn format_register_name(reg: Register) -> String {
    match reg {
        Register::X0 => "x0".to_string(),
        Register::X1 => "x1".to_string(),
        Register::X2 => "x2".to_string(),
        Register::X3 => "x3".to_string(),
        Register::X4 => "x4".to_string(),
        Register::X5 => "x5".to_string(),
        Register::X6 => "x6".to_string(),
        Register::X7 => "x7".to_string(),
        Register::X8 => "x8".to_string(),
        Register::X9 => "x9".to_string(),
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

/// Formats a StateDelta showing detailed instruction execution results
/// This is the main formatter for instruction execution output
pub fn format_instruction_result(delta: &StateDelta) -> String {
    let mut changes = Vec::new();

    // Format register changes (show the most important ones)
    let mut has_pc_change = false;
    for (reg, old, new) in &delta.register_changes {
        if *reg == Register::PC {
            changes.push(format!("PC: 0x{old:08x} → 0x{new:08x}"));
            has_pc_change = true;
        } else {
            changes.push(format!(
                "{}: {} → {}",
                format_register_name(*reg),
                *old as i32,
                *new as i32
            ));
        }
    }

    // Always show PC change from delta
    if !has_pc_change && delta.pc_change.0 != delta.pc_change.1 {
        changes.push(format!(
            "PC: 0x{:08x} → 0x{:08x}",
            delta.pc_change.0, delta.pc_change.1
        ));
    }

    // Show memory changes summary
    if !delta.memory_changes.is_empty() {
        changes.push(format!(
            "{} memory bytes changed",
            delta.memory_changes.len()
        ));
    }

    // Show CSR changes
    for (csr, old, new) in &delta.csr_changes {
        changes.push(format!("CSR[0x{csr:03x}]: 0x{old:08x} → 0x{new:08x}"));
    }

    if changes.is_empty() {
        "No state changes".to_string()
    } else {
        changes.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use brubeck::rv32_i::MemoryDelta;

    #[test]
    fn test_format_instruction_result_basic() {
        let mut delta = StateDelta::new();
        delta.pc_change = (0, 4);
        delta.register_changes.push((Register::X1, 0, 42));

        let result = format_instruction_result(&delta);
        assert!(result.contains("x1: 0 → 42"));
        assert!(result.contains("PC: 0x00000000 → 0x00000004"));
    }

    #[test]
    fn test_format_instruction_result_memory() {
        let mut delta = StateDelta::new();
        delta.pc_change = (0, 4);
        delta.memory_changes.push(MemoryDelta {
            addr: 0x100,
            old_data: vec![0x00],
            new_data: vec![0x42],
        });

        let result = format_instruction_result(&delta);
        assert!(result.contains("memory bytes changed"));
        assert!(result.contains("PC: 0x00000000 → 0x00000004"));
    }
}
