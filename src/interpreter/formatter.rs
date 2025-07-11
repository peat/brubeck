//! Formatting functions for human-readable output
//!
//! This module contains functions that format instruction execution results.

use crate::rv32_i::StateDelta;

/// Formats the result of executing an instruction using StateDelta information
pub fn format_instruction_result(instruction_name: &str, delta: &StateDelta) -> String {
    let mut result = format!("{instruction_name}: ");

    // Format register changes
    if !delta.register_changes.is_empty() {
        let reg_changes: Vec<String> = delta
            .register_changes
            .iter()
            .map(|(reg, old, new)| format!("{reg:?}: {old} -> {new}"))
            .collect();
        result.push_str(&format!("Registers: [{}] ", reg_changes.join(", ")));
    }

    // Format PC change
    if delta.pc_change.0 != delta.pc_change.1 {
        result.push_str(&format!(
            "PC: {} -> {} ",
            delta.pc_change.0, delta.pc_change.1
        ));
    }

    // Format memory changes
    if !delta.memory_changes.is_empty() {
        result.push_str(&format!(
            "Memory: {} bytes changed ",
            delta.memory_changes.len()
        ));
    }

    // Format CSR changes
    if !delta.csr_changes.is_empty() {
        result.push_str(&format!("CSR: {} changed ", delta.csr_changes.len()));
    }

    result.trim_end().to_string()
}
