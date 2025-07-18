//! Formatter for StateDelta - showing CPU state changes

use brubeck::rv32_i::{Register, StateDelta};

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
            changes.push(format!("{reg:?}: {} → {}", *old as i32, *new as i32));
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

/// Formats a StateDelta in a compact single-line format
pub fn format_state_delta_compact(delta: &StateDelta) -> String {
    let mut parts = Vec::new();

    // Count changes by type
    let reg_count = delta.register_changes.len();
    let mem_count = delta.memory_changes.len();
    let csr_count = delta.csr_changes.len();

    if reg_count > 0 {
        parts.push(format!(
            "{} register{}",
            reg_count,
            if reg_count == 1 { "" } else { "s" }
        ));
    }
    if mem_count > 0 {
        parts.push(format!(
            "{} memory location{}",
            mem_count,
            if mem_count == 1 { "" } else { "s" }
        ));
    }
    if csr_count > 0 {
        parts.push(format!(
            "{} CSR{}",
            csr_count,
            if csr_count == 1 { "" } else { "s" }
        ));
    }

    if parts.is_empty() {
        "No changes".to_string()
    } else {
        format!("Changed: {}", parts.join(", "))
    }
}