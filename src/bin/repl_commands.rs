//! REPL command handling
//!
//! This module handles the parsing and execution of REPL-specific slash commands
//! that are not part of the core library.

use crate::formatting;
use brubeck::interpreter::Interpreter;
use brubeck::rv32_i::Register;

/// Represents a REPL-specific command
#[derive(Debug)]
pub enum ReplCommand {
    ShowRegs,
    ShowSpecificRegs(Vec<Register>),
    ShowHelp,
    Previous,
    Next,
    Reset,
    ShowMemory {
        start: Option<u32>,
        end: Option<u32>,
    },
    Quit,
}

/// Parse and execute a REPL command
pub fn handle_repl_command(input: &str, interpreter: &mut Interpreter) -> Result<String, String> {
    handle_repl_command_with_delta(input, interpreter, None)
}

/// Parse and execute a REPL command with access to last delta for coloring
pub fn handle_repl_command_with_delta(
    input: &str,
    interpreter: &mut Interpreter,
    last_delta: Option<&brubeck::rv32_i::StateDelta>,
) -> Result<String, String> {
    let normalized = input.trim().to_uppercase();
    let parts: Vec<&str> = normalized.split_whitespace().collect();

    if parts.is_empty() || !parts[0].starts_with('/') {
        return Err("Not a REPL command".to_string());
    }

    let cmd = parse_repl_command(&parts)?;
    execute_repl_command_with_delta(cmd, interpreter, last_delta)
}

/// Parse the command and arguments
fn parse_repl_command(parts: &[&str]) -> Result<ReplCommand, String> {
    match parts[0] {
        "/REGS" | "/R" => {
            if parts.len() == 1 {
                Ok(ReplCommand::ShowRegs)
            } else {
                // Parse register arguments
                let mut regs = Vec::new();
                for arg in &parts[1..] {
                    match parse_register(arg) {
                        Some(reg) => regs.push(reg),
                        None => return Err(format!("Invalid register: {arg}")),
                    }
                }
                Ok(ReplCommand::ShowSpecificRegs(regs))
            }
        }
        "/HELP" | "/H" => Ok(ReplCommand::ShowHelp),
        "/PREVIOUS" | "/PREV" | "/P" => Ok(ReplCommand::Previous),
        "/NEXT" | "/N" => Ok(ReplCommand::Next),
        "/RESET" => Ok(ReplCommand::Reset),
        "/QUIT" | "/Q" | "/EXIT" | "/E" => Ok(ReplCommand::Quit),
        "/MEMORY" | "/M" => match parts.len() {
            1 => Ok(ReplCommand::ShowMemory {
                start: None,
                end: None,
            }),
            2 => {
                let addr = parse_address(parts[1])?;
                Ok(ReplCommand::ShowMemory {
                    start: Some(addr),
                    end: None,
                })
            }
            3 => {
                let start = parse_address(parts[1])?;
                let end = parse_address(parts[2])?;
                if end <= start {
                    return Err("End address must be greater than start address".to_string());
                }
                if end - start > 256 {
                    return Err("Memory range too large (max 256 bytes)".to_string());
                }
                Ok(ReplCommand::ShowMemory {
                    start: Some(start),
                    end: Some(end),
                })
            }
            _ => Err("Too many arguments for /memory command".to_string()),
        },
        _ => Err(format!("Unknown command: {}", parts[0])),
    }
}

/// Execute the REPL command with optional delta for coloring
fn execute_repl_command_with_delta(
    cmd: ReplCommand,
    interpreter: &mut Interpreter,
    last_delta: Option<&brubeck::rv32_i::StateDelta>,
) -> Result<String, String> {
    match cmd {
        ReplCommand::ShowRegs => Ok(formatting::registers::format_registers_with_colors(
            &interpreter.cpu,
            true,
            last_delta,
        )),
        ReplCommand::ShowSpecificRegs(regs) => Ok(
            formatting::registers::format_specific_registers(&interpreter.cpu, &regs),
        ),
        ReplCommand::ShowHelp => Ok(formatting::help::format_help()),
        ReplCommand::Previous => handle_previous(interpreter),
        ReplCommand::Next => handle_next(interpreter),
        ReplCommand::Reset => handle_reset(interpreter),
        ReplCommand::ShowMemory { start, end } => {
            Ok(formatting::memory::format_memory_range_with_colors(
                &interpreter.cpu,
                start,
                end,
                last_delta,
            ))
        }
        ReplCommand::Quit => {
            // Return a special error that signals the main loop to exit
            Err("QUIT".to_string())
        }
    }
}

/// Handle the /previous command
fn handle_previous(interpreter: &mut Interpreter) -> Result<String, String> {
    // Use the new API and format the delta
    match interpreter.previous_state() {
        Ok(delta) => Ok(format!(
            "Navigated back: {}",
            formatting::state_delta::format_instruction_result(&delta)
        )),
        Err(e) => Err(formatting::errors::format_history_error(&e, true)),
    }
}

/// Handle the /next command
fn handle_next(interpreter: &mut Interpreter) -> Result<String, String> {
    // Use the new API and format the delta
    match interpreter.next_state() {
        Ok(delta) => Ok(format!(
            "Navigated forward: {}",
            formatting::state_delta::format_instruction_result(&delta)
        )),
        Err(e) => Err(formatting::errors::format_history_error(&e, true)),
    }
}

/// Handle the /reset command
fn handle_reset(interpreter: &mut Interpreter) -> Result<String, String> {
    use std::io::{self, Write};

    // Print confirmation prompt
    print!("Reset CPU? This will clear all registers, memory, and history. (y/N): ");
    io::stdout()
        .flush()
        .map_err(|e| format!("Failed to flush stdout: {e}"))?;

    // Read user input
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| format!("Failed to read input: {e}"))?;

    // Check if user confirmed
    let confirmed = input.trim().eq_ignore_ascii_case("y");

    if confirmed {
        // Reset interpreter (CPU and history)
        interpreter.reset();

        Ok("CPU state reset".to_string())
    } else {
        Ok("Reset cancelled".to_string())
    }
}

/// Parse a register name (both numeric and ABI names)
fn parse_register(input: &str) -> Option<Register> {
    match input.to_uppercase().as_str() {
        "X0" => Some(Register::X0),
        "X1" => Some(Register::X1),
        "X2" => Some(Register::X2),
        "X3" => Some(Register::X3),
        "X4" => Some(Register::X4),
        "X5" => Some(Register::X5),
        "X6" => Some(Register::X6),
        "X7" => Some(Register::X7),
        "X8" => Some(Register::X8),
        "X9" => Some(Register::X9),
        "X10" => Some(Register::X10),
        "X11" => Some(Register::X11),
        "X12" => Some(Register::X12),
        "X13" => Some(Register::X13),
        "X14" => Some(Register::X14),
        "X15" => Some(Register::X15),
        "X16" => Some(Register::X16),
        "X17" => Some(Register::X17),
        "X18" => Some(Register::X18),
        "X19" => Some(Register::X19),
        "X20" => Some(Register::X20),
        "X21" => Some(Register::X21),
        "X22" => Some(Register::X22),
        "X23" => Some(Register::X23),
        "X24" => Some(Register::X24),
        "X25" => Some(Register::X25),
        "X26" => Some(Register::X26),
        "X27" => Some(Register::X27),
        "X28" => Some(Register::X28),
        "X29" => Some(Register::X29),
        "X30" => Some(Register::X30),
        "X31" => Some(Register::X31),

        // ABI names
        "ZERO" => Some(Register::X0),
        "RA" => Some(Register::X1),
        "SP" => Some(Register::X2),
        "GP" => Some(Register::X3),
        "TP" => Some(Register::X4),
        "T0" => Some(Register::X5),
        "T1" => Some(Register::X6),
        "T2" => Some(Register::X7),
        "S0" | "FP" => Some(Register::X8), // S0 is also frame pointer
        "S1" => Some(Register::X9),
        "A0" => Some(Register::X10),
        "A1" => Some(Register::X11),
        "A2" => Some(Register::X12),
        "A3" => Some(Register::X13),
        "A4" => Some(Register::X14),
        "A5" => Some(Register::X15),
        "A6" => Some(Register::X16),
        "A7" => Some(Register::X17),
        "S2" => Some(Register::X18),
        "S3" => Some(Register::X19),
        "S4" => Some(Register::X20),
        "S5" => Some(Register::X21),
        "S6" => Some(Register::X22),
        "S7" => Some(Register::X23),
        "S8" => Some(Register::X24),
        "S9" => Some(Register::X25),
        "S10" => Some(Register::X26),
        "S11" => Some(Register::X27),
        "T3" => Some(Register::X28),
        "T4" => Some(Register::X29),
        "T5" => Some(Register::X30),
        "T6" => Some(Register::X31),

        // Special registers
        "PC" => Some(Register::PC),

        _ => None,
    }
}

/// Parse a memory address (unsigned 32-bit value)
fn parse_address(input: &str) -> Result<u32, String> {
    let input = input.trim();

    let result = if let Some(hex_str) = input.strip_prefix("0X") {
        // Hexadecimal
        u32::from_str_radix(hex_str, 16)
    } else if let Some(bin_str) = input.strip_prefix("0B") {
        // Binary
        u32::from_str_radix(bin_str, 2)
    } else {
        // Decimal
        input.parse::<u32>()
    };

    result.map_err(|_| format!("Invalid address: {input}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use brubeck::interpreter::Interpreter;

    #[test]
    fn test_parse_register_inspection() {
        let mut i = Interpreter::new();

        // Test register inspection
        let result = handle_repl_command("/regs PC", &mut i);
        assert!(result.is_ok());
        let output = result.unwrap();
        // PC is always shown as a single register on its own line
        assert!(output.contains("pc:") || output.contains("PC:"));

        let result = handle_repl_command("/regs X1", &mut i);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("x1"));
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
        assert!(output.contains("x2"));
        assert!(output.contains("sp"));
        assert!(output.contains("0x00000064")); // 100 in hex

        let result = handle_repl_command("/regs ra", &mut i);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("x1"));
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
        let output = result.unwrap();
        // Check for the address format (without space, since we might have color codes)
        assert!(output.contains("0x00000000"), "Output: {}", output);

        // Test memory command with valid address
        let result = handle_repl_command("/memory 0x0", &mut i);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("0x00000000"));

        // Test memory command with range (0x100 to 0x120 = 32 bytes)
        let result = handle_repl_command("/memory 0x100 0x120", &mut i);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("0x00000100"));
        // Should show 32 bytes (2 lines of 16 bytes each)
        assert!(output.contains("0x00000110"));
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
        assert!(output.contains("x1"));
        assert!(output.contains("0x00000064")); // 100 in hex
        assert!(!output.contains("x2")); // Should not show other registers

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
        assert!(output.contains("x0"));
        assert!(output.contains("x31"));
        assert!(output.contains("pc"));

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
