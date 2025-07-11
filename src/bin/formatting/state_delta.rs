//! Formatter for StateDelta - showing CPU state changes

use brubeck::rv32_i::{Register, StateDelta};

/// Formats a StateDelta into a human-readable string showing what changed
pub fn format_state_delta(delta: &StateDelta) -> String {
    let mut output = String::new();
    let mut changes = Vec::new();

    // Format register changes
    for (reg, old, new) in &delta.register_changes {
        // Special handling for PC to show as hex
        if *reg == Register::PC {
            changes.push(format!("PC: 0x{:08x} → 0x{:08x}", old, new));
        } else {
            // Show register name and values
            changes.push(format!(
                "{:?}: 0x{:08x} → 0x{:08x} ({})",
                reg,
                old,
                new,
                *new as i32
            ));
        }
    }

    // Format memory changes
    for mem_delta in &delta.memory_changes {
        if mem_delta.old_data.len() == 1 {
            changes.push(format!(
                "[0x{:08x}]: 0x{:02x} → 0x{:02x}",
                mem_delta.addr, mem_delta.old_data[0], mem_delta.new_data[0]
            ));
        } else {
            changes.push(format!(
                "[0x{:08x}]: {} bytes changed",
                mem_delta.addr, mem_delta.old_data.len()
            ));
        }
    }

    // Format CSR changes
    for (csr, old, new) in &delta.csr_changes {
        changes.push(format!(
            "CSR[0x{:03x}]: 0x{:08x} → 0x{:08x}",
            csr, old, new
        ));
    }

    // Join all changes with newlines
    if changes.is_empty() {
        output.push_str("No state changes");
    } else {
        output.push_str(&changes.join("\n"));
    }

    output
}

/// Formats a StateDelta in a compact single-line format
pub fn format_state_delta_compact(delta: &StateDelta) -> String {
    let mut parts = Vec::new();

    // Count changes by type
    let reg_count = delta.register_changes.len();
    let mem_count = delta.memory_changes.len();
    let csr_count = delta.csr_changes.len();

    if reg_count > 0 {
        parts.push(format!("{} register{}", reg_count, if reg_count == 1 { "" } else { "s" }));
    }
    if mem_count > 0 {
        parts.push(format!("{} memory location{}", mem_count, if mem_count == 1 { "" } else { "s" }));
    }
    if csr_count > 0 {
        parts.push(format!("{} CSR{}", csr_count, if csr_count == 1 { "" } else { "s" }));
    }

    if parts.is_empty() {
        "No changes".to_string()
    } else {
        format!("Changed: {}", parts.join(", "))
    }
}