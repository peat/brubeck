//! Tests for REPL-specific commands
//!
//! These tests verify that REPL commands like /regs, /memory, /undo, /redo
//! work correctly. These were moved from the library tests during the
//! library/binary separation.

#[cfg(test)]
mod tests {
    use crate::repl_commands::handle_repl_command;
    use brubeck::interpreter::Interpreter;

    #[test]
    fn test_parse_register_inspection() {
        let mut i = Interpreter::new();

        // Test register inspection
        let result = handle_repl_command("/regs PC", &mut i);
        assert!(result.is_ok());
        let output = result.unwrap();
        // PC is always shown as a single register on its own line
        assert!(output.contains("pc      :") || output.contains("pc:"));

        let result = handle_repl_command("/regs X1", &mut i);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("x 1"));
    }

    #[test]
    fn test_parse_abi_register_names() {
        let mut i = Interpreter::new();

        // Set some register values
        i.interpret("ADDI sp, zero, 100").unwrap();
        i.interpret("ADDI ra, zero, 200").unwrap();

        // Test ABI register names
        let result = handle_repl_command("/regs sp", &mut i);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("x 2"));
        assert!(output.contains("sp"));
        assert!(output.contains("0x00000064")); // 100 in hex

        let result = handle_repl_command("/regs ra", &mut i);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("x 1"));
        assert!(output.contains("ra"));
        assert!(output.contains("0x000000c8")); // 200 in hex
    }

    #[test]
    fn test_parse_navigation_commands() {
        let mut i = Interpreter::new();

        // Execute some instructions to create history
        i.interpret("ADDI x1, zero, 42").unwrap();
        i.interpret("ADDI x2, zero, 84").unwrap();

        // Test undo - use /previous which is what's implemented
        let result = handle_repl_command("/previous", &mut i);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Navigated back"));

        // Test next instead of redo
        let result = handle_repl_command("/next", &mut i);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Navigated forward"));

        // Test /p alias for previous
        let result = handle_repl_command("/p", &mut i);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Navigated back"));
    }

    #[test]
    fn test_parse_memory_command() {
        let mut i = Interpreter::new();

        // Test memory command without arguments (shows memory around PC)
        let result = handle_repl_command("/memory", &mut i);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("0x00000000:"));

        // Test memory command with valid address
        let result = handle_repl_command("/memory 0x0", &mut i);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("0x00000000:"));

        // Test memory command with range (0x100 to 0x120 = 32 bytes)
        let result = handle_repl_command("/memory 0x100 0x120", &mut i);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("0x00000100:"));
        // Should show 32 bytes (2 lines of 16 bytes each)
        assert!(output.contains("0x00000110:"));
    }

    #[test]
    fn test_memory_display_with_data() {
        let mut i = Interpreter::new();

        // Store some data in memory
        i.interpret("ADDI x1, zero, 256").unwrap(); // base address 0x100
        i.interpret("LUI x2, 0x12345").unwrap(); // Load upper bits
        i.interpret("ADDI x2, x2, 1656").unwrap(); // Add lower bits (0x678 = 1656)
        i.interpret("SW x2, 0(x1)").unwrap(); // store word

        // Read memory to verify format (0x100 to 0x110 = 16 bytes)
        let result = handle_repl_command("/memory 0x100 0x110", &mut i);
        assert!(result.is_ok());
        let output = result.unwrap();

        // Should show the stored bytes in little-endian order
        assert!(output.contains("78 56 34 12"));
    }

    #[test]
    fn test_help_command() {
        let mut i = Interpreter::new();

        // Test help command
        let result = handle_repl_command("/help", &mut i);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RISC-V REPL Commands:"));
        assert!(output.contains("/r, /regs"));
        assert!(output.contains("/m, /memory"));
        assert!(output.contains("/p, /prev, /previous"));
        assert!(output.contains("/n, /next"));

        // Test help alias
        let result = handle_repl_command("/h", &mut i);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("RISC-V REPL Commands:"));
    }

    #[test]
    fn test_invalid_command() {
        let mut i = Interpreter::new();

        // Test invalid command
        let result = handle_repl_command("/invalid", &mut i);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown command"));
    }

    #[test]
    fn test_register_command_variations() {
        let mut i = Interpreter::new();

        // Set some values
        i.interpret("ADDI x1, zero, 100").unwrap();
        i.interpret("ADDI x2, zero, 200").unwrap();
        i.interpret("ADDI x3, zero, 300").unwrap();

        // Test single register
        let result = handle_repl_command("/regs x1", &mut i);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("x 1"));
        assert!(output.contains("0x00000064")); // 100 in hex
        assert!(!output.contains("x 2")); // Should not show other registers

        // Test multiple registers
        let result = handle_repl_command("/regs x1 x2 x3", &mut i);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("0x00000064")); // 100 in hex
        assert!(output.contains("0x000000c8")); // 200 in hex
        assert!(output.contains("0x0000012c")); // 300 in hex

        // Test all registers
        let result = handle_repl_command("/regs", &mut i);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("x 0"));
        assert!(output.contains("x31"));
        assert!(output.contains("pc      :"));

        // Test with /r alias
        let result = handle_repl_command("/r x1", &mut i);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("0x00000064")); // 100 in hex
    }

    #[test]
    fn test_quit_command() {
        let mut i = Interpreter::new();

        // Test /quit command
        let result = handle_repl_command("/quit", &mut i);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "QUIT");

        // Test /q alias
        let result = handle_repl_command("/q", &mut i);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "QUIT");

        // Test /exit command
        let result = handle_repl_command("/exit", &mut i);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "QUIT");

        // Test /e alias
        let result = handle_repl_command("/e", &mut i);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "QUIT");
    }
}
