//! Memory display formatter

use brubeck::rv32_i::CPU;

/// Formats a memory range in hex dump format with ASCII sidebar
pub fn format_memory_range(cpu: &CPU, start: Option<u32>, end: Option<u32>) -> String {
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

    let mut output = String::new();
    output.push_str("Address    00 01 02 03 04 05 06 07 | 08 09 0A 0B 0C 0D 0E 0F  ASCII\n");
    output.push_str(
        "--------   ----------------------- + -----------------------  ----------------\n",
    );

    // Align start to 16-byte boundary
    let aligned_start = start_addr & !0xF;
    let aligned_end = (end_addr + 15) & !0xF;

    for addr in (aligned_start..aligned_end).step_by(16) {
        // Address column
        output.push_str(&format!("0x{addr:08x} "));

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
                output.push_str(&format!("{byte:02x} "));

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
