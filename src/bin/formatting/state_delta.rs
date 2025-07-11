//! Formatter for StateDelta - showing CPU state changes

use brubeck::rv32_i::StateDelta;


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