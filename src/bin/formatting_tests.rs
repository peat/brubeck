//! Tests for the formatting module

#[cfg(test)]
mod tests {
    use crate::formatting;
    use brubeck::interpreter::Interpreter;
    use brubeck::rv32_i::{MemoryDelta, Register, StateDelta};
    use brubeck::HistoryError;

    #[test]
    fn test_format_instruction_result_basic() {
        let mut delta = StateDelta::new();
        delta.pc_change = (0, 4);
        delta.register_changes.push((Register::X1, 0, 42));

        let result = formatting::state_delta::format_instruction_result(&delta);
        assert!(result.contains("X1: 0 → 42"));
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

        let result = formatting::state_delta::format_instruction_result(&delta);
        assert!(result.contains("memory bytes changed"));
        assert!(result.contains("PC: 0x00000000 → 0x00000004"));
    }

    #[test]
    fn test_format_registers_basic() {
        let mut interpreter = Interpreter::new();
        interpreter.cpu.set_register(Register::X1, 42);
        interpreter.cpu.set_register(Register::X2, 0);

        let result =
            formatting::registers::format_registers_with_colors(&interpreter.cpu, false, None);
        assert!(result.contains("x0"));
        assert!(result.contains("x1"));
        assert!(result.contains("0x0000002a"));
        assert!(result.contains("42"));
    }

    #[test]
    fn test_format_registers_with_abi_names() {
        let interpreter = Interpreter::new();
        let result =
            formatting::registers::format_registers_with_colors(&interpreter.cpu, true, None);
        assert!(result.contains("x0 (zero)"));
        assert!(result.contains("x1 (ra)"));
        assert!(result.contains("x2 (sp)"));
    }

    #[test]
    fn test_format_memory_basic() {
        let mut interpreter = Interpreter::new();
        // Write some test data
        interpreter.cpu.memory[0x100] = 0x48; // 'H'
        interpreter.cpu.memory[0x101] = 0x65; // 'e'
        interpreter.cpu.memory[0x102] = 0x6C; // 'l'
        interpreter.cpu.memory[0x103] = 0x6C; // 'l'
        interpreter.cpu.memory[0x104] = 0x6F; // 'o'

        let result = formatting::memory::format_memory_range_with_colors(
            &interpreter.cpu,
            Some(0x100),
            Some(0x110),
            None,
        );
        assert!(result.contains("0x00000100"));
        assert!(result.contains("48 65 6c 6c 6f")); // "Hello" in hex
        assert!(result.contains("Hello")); // ASCII representation
    }

    #[test]
    fn test_format_memory_with_pc() {
        let mut interpreter = Interpreter::new();
        interpreter.cpu.pc = 0x100;

        let result = formatting::memory::format_memory_range_with_colors(
            &interpreter.cpu,
            Some(0x100),
            Some(0x110),
            None,
        );
        // Should highlight the address line containing PC
        assert!(result.contains("0x00000100"));
    }

    #[test]
    fn test_format_help() {
        let help = formatting::help::format_help();
        assert!(help.contains("RISC-V REPL Commands"));
        assert!(help.contains("/regs"));
        assert!(help.contains("/memory"));
        assert!(help.contains("/previous"));
        assert!(help.contains("/help"));
    }

    #[test]
    fn test_format_history_error() {
        let error = HistoryError::AtBeginning;
        let result = formatting::errors::format_history_error(&error, true);
        assert!(result.contains("beginning of the undo history"));
        assert!(result.contains("💡 Tip:"));
        assert!(result.contains("--history-limit"));
    }

    #[test]
    fn test_format_history_error_no_tips() {
        let error = HistoryError::AtEnd;
        let result = formatting::errors::format_history_error(&error, false);
        assert!(result.contains("most recent state"));
        assert!(!result.contains("💡 Tip:"));
    }

    #[test]
    fn test_format_io_error() {
        use std::io;

        let error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let result = formatting::errors::format_io_error(&error, "Failed to read script", true);
        assert!(result.contains("Failed to read script"));
        assert!(result.contains("file not found"));
        assert!(result.contains("💡 Tip:"));
        assert!(result.contains("Check that the script file path"));
    }

    #[test]
    fn test_format_parse_error() {
        let result =
            formatting::errors::format_parse_error("Invalid memory size", "memory_size", true);
        assert!(result.contains("Invalid memory size"));
        assert!(result.contains("💡 Tip:"));
        assert!(result.contains("1M"));
        assert!(result.contains("256k"));
    }

    #[test]
    fn test_format_repl_command_error() {
        let result = formatting::errors::format_repl_command_error("Unknown command: /foo", true);
        assert!(result.contains("Unknown command"));
        assert!(result.contains("💡 Tip:"));
        assert!(result.contains("Available commands"));
        assert!(result.contains("/help"));
    }

    #[test]
    fn test_format_state_delta_compact() {
        let mut delta = StateDelta::new();
        delta.register_changes.push((Register::X1, 0, 42));
        delta.register_changes.push((Register::X2, 10, 20));

        let result = formatting::state_delta::format_state_delta_compact(&delta);
        assert!(result.contains("Changed: 2 registers"));
    }

    #[test]
    fn test_format_specific_registers() {
        let mut interpreter = Interpreter::new();
        interpreter.cpu.set_register(Register::X1, 100);
        interpreter.cpu.set_register(Register::X2, 200);

        let regs = vec![Register::X1, Register::X2];
        let result = formatting::registers::format_specific_registers(&interpreter.cpu, &regs);
        assert!(result.contains("x1"));
        assert!(result.contains("x2"));
        assert!(result.contains("100"));
        assert!(result.contains("200"));
    }
}
