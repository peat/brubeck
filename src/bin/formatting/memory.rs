//! Memory display formatter

use brubeck::rv32_i::{StateDelta, CPU};
use crossterm::style::Stylize;
use std::collections::HashSet;

/// Formats a memory range with optional coloring for changed bytes
///
/// Displays memory in a traditional hex dump format with 16 bytes per line.
/// When a StateDelta is provided, it will color-code the output:
/// - **Yellow background**: Current PC location (instruction pointer)
/// - **Green**: Bytes that were modified by the last instruction
/// - **Dark gray**: Zero bytes
/// - **Yellow address**: Lines containing the PC
///
/// # Arguments
/// * `cpu` - The CPU state containing memory to display
/// * `start` - Optional start address (defaults to PC - 32, aligned to 16 bytes)
/// * `end` - Optional end address (defaults to start + 64 bytes)
/// * `last_delta` - Optional state changes from the last instruction for highlighting
///
/// # Examples
/// - `/memory` - Shows 64 bytes around the current PC
/// - `/memory 0x100` - Shows 64 bytes starting at address 0x100
/// - `/memory 0x100 0x200` - Shows memory from 0x100 to 0x200
pub fn format_memory_range_with_colors(
    cpu: &CPU,
    start: Option<u32>,
    end: Option<u32>,
    last_delta: Option<&StateDelta>,
) -> String {
    // Default to showing 64 bytes around PC if no range specified
    let (start_addr, end_addr) = match (start, end) {
        (Some(s), Some(e)) => (s, e),
        (Some(s), None) => (s, s + 64),
        (None, None) => {
            let pc = cpu.pc;
            let start = pc.saturating_sub(32) & !0xF; // Align to 16-byte boundary
            (start, start + 64)
        }
        (None, Some(_)) => {
            return "Error: End address specified without start address".to_string();
        }
    };

    // Validate range
    if end_addr <= start_addr {
        return "Error: End address must be greater than start address".to_string();
    }
    if end_addr - start_addr > 1024 {
        return "Error: Memory range too large (max 1024 bytes)".to_string();
    }

    // Build set of changed addresses for quick lookup
    let changed_addrs: HashSet<u32> = last_delta
        .map(|delta| {
            delta
                .memory_changes
                .iter()
                .flat_map(|md| {
                    // Each MemoryDelta can affect multiple bytes
                    (md.addr..md.addr + md.new_data.len() as u32).collect::<Vec<_>>()
                })
                .collect()
        })
        .unwrap_or_default();

    let mut output = String::new();
    output.push_str("Address    00 01 02 03 04 05 06 07 | 08 09 0A 0B 0C 0D 0E 0F  ASCII\n");
    output.push_str(
        "--------   ----------------------- + -----------------------  ----------------\n",
    );

    // Align start to 16-byte boundary
    let aligned_start = start_addr & !0xF;
    let aligned_end = (end_addr + 15) & !0xF;

    for addr in (aligned_start..aligned_end).step_by(16) {
        // Address column - highlight if PC is in this line
        let addr_str = if cpu.pc >= addr && cpu.pc < addr + 16 {
            format!("0x{addr:08x}").yellow().to_string()
        } else {
            format!("0x{addr:08x}")
        };
        output.push_str(&addr_str);
        output.push(' ');

        let mut ascii_chars = Vec::new();

        // Hex bytes
        for i in 0..16 {
            let byte_addr = addr + i;

            // Add separator at byte 8
            if i == 8 {
                output.push_str("| ");
            }

            // Only show bytes within the requested range and memory bounds
            if byte_addr >= start_addr
                && byte_addr < end_addr
                && (byte_addr as usize) < cpu.memory.len()
            {
                let byte = cpu.memory[byte_addr as usize];

                // Format hex with color if changed
                let hex_str = if byte_addr == cpu.pc {
                    // Show PC position with special formatting
                    format!("{byte:02x}").on_yellow().black().to_string()
                } else if changed_addrs.contains(&byte_addr) {
                    format!("{byte:02x}").green().to_string()
                } else if byte == 0 {
                    format!("{byte:02x}").dark_grey().to_string()
                } else {
                    format!("{byte:02x}")
                };
                output.push_str(&hex_str);
                output.push(' ');

                // Collect ASCII representation
                if byte.is_ascii_graphic() || byte == b' ' {
                    ascii_chars.push(byte as char);
                } else {
                    ascii_chars.push('.');
                }
            } else {
                output.push_str("   ");
                ascii_chars.push(' ');
            }
        }

        // ASCII column
        output.push(' ');
        for (i, ch) in ascii_chars.iter().enumerate() {
            if i == 8 {
                output.push(' ');
            }
            output.push(*ch);
        }

        output.push('\n');
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use brubeck::rv32_i::CPU;

    #[test]
    fn test_format_memory_basic() {
        let mut cpu = CPU::new(1024);
        // Write some test data
        cpu.memory[0x100] = 0x48; // 'H'
        cpu.memory[0x101] = 0x65; // 'e'
        cpu.memory[0x102] = 0x6C; // 'l'
        cpu.memory[0x103] = 0x6C; // 'l'
        cpu.memory[0x104] = 0x6F; // 'o'

        let result = format_memory_range_with_colors(&cpu, Some(0x100), Some(0x110), None);
        assert!(result.contains("0x00000100"));
        assert!(result.contains("48 65 6c 6c 6f")); // "Hello" in hex
        assert!(result.contains("Hello")); // ASCII representation
    }

    #[test]
    fn test_format_memory_with_pc() {
        let mut cpu = CPU::new(1024);
        cpu.pc = 0x100;

        let result = format_memory_range_with_colors(&cpu, Some(0x100), Some(0x110), None);
        // Should highlight the address line containing PC
        assert!(result.contains("0x00000100"));
    }
}
