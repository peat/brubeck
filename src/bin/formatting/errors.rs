//! Error formatting with helpful context and tips

use brubeck::{CPUError, ExecutionError, HistoryError, ParseError};

/// Formats an ExecutionError with optional tips
pub fn format_error(error: &ExecutionError, tips_enabled: bool) -> String {
    match error {
        ExecutionError::ParseError(e) => format_parse_error(e, tips_enabled),
        ExecutionError::CPUError(e) => format_cpu_error(e, tips_enabled),
    }
}

/// Formats a ParseError with helpful context
pub fn format_parse_error(error: &ParseError, tips_enabled: bool) -> String {
    let mut output = String::new();
    
    match error {
        ParseError::UnknownInstruction { instruction, suggestion } => {
            output.push_str(&format!("Unknown instruction '{instruction}'"));
            if let Some(s) = suggestion {
                output.push_str(&format!(". Did you mean '{s}'?"));
            }
            if tips_enabled {
                output.push_str("\n💡 Tip: Type /help to see available commands and instruction formats");
            }
        }
        ParseError::InvalidRegister { register } => {
            output.push_str(&format!("Invalid register '{register}'"));
            if tips_enabled {
                output.push_str("\n💡 Tip: Valid registers are x0-x31 or ABI names like 'sp', 'ra', 'zero'");
            }
        }
        ParseError::WrongArgumentCount { instruction, expected, found } => {
            output.push_str(&format!(
                "{instruction} expects {expected} argument{}, but {found} {} provided",
                if *expected == 1 { "" } else { "s" },
                if *found == 1 { "was" } else { "were" }
            ));
            if tips_enabled {
                output.push_str(&format!(
                    "\n💡 Tip: Check the instruction format. Example: {} {}",
                    instruction,
                    get_instruction_example(instruction)
                ));
            }
        }
        ParseError::ImmediateOutOfRange { instruction, value, min, max } => {
            output.push_str(&format!(
                "Immediate value {value} out of range for {instruction} (valid range: {min} to {max})"
            ));
            if tips_enabled {
                output.push_str("\n💡 Tip: Use LI pseudo-instruction for large constants");
            }
        }
        ParseError::SyntaxError { message } => {
            output.push_str(message);
            if tips_enabled && message.contains("slash commands") {
                output.push_str("\n💡 Tip: Slash commands like /regs are REPL features, not assembly instructions");
            }
        }
    }
    
    output
}

/// Formats a CPUError with context
pub fn format_cpu_error(error: &CPUError, tips_enabled: bool) -> String {
    let mut output = error.to_string();
    
    if tips_enabled {
        match error {
            CPUError::MisalignedJump(_) => {
                output.push_str("\n💡 Tip: RISC-V requires instruction addresses to be 4-byte aligned");
            }
            CPUError::AccessViolation(_) => {
                output.push_str("\n💡 Tip: Check memory bounds. Default CPU has 1MB (0x00000000-0x000FFFFF)");
            }
            _ => {}
        }
    }
    
    output
}

/// Formats a HistoryError
pub fn format_history_error(error: &HistoryError, tips_enabled: bool) -> String {
    let mut output = error.to_string();
    
    if tips_enabled {
        match error {
            HistoryError::AtBeginning => {
                output.push_str("\n💡 Tip: You're at the beginning of the undo history. Use --history-limit to increase history size");
            }
            HistoryError::AtEnd => {
                output.push_str("\n💡 Tip: You're at the most recent state. Execute new instructions to continue");
            }
        }
    }
    
    output
}

/// Get example usage for an instruction
fn get_instruction_example(instruction: &str) -> &'static str {
    match instruction.to_uppercase().as_str() {
        "ADD" | "SUB" | "AND" | "OR" | "XOR" | "SLL" | "SRL" | "SRA" | "SLT" | "SLTU" => {
            "rd, rs1, rs2"
        }
        "ADDI" | "ANDI" | "ORI" | "XORI" | "SLTI" | "SLTIU" => "rd, rs1, imm",
        "SLLI" | "SRLI" | "SRAI" => "rd, rs1, shamt",
        "LW" | "LH" | "LHU" | "LB" | "LBU" => "rd, offset(rs1)",
        "SW" | "SH" | "SB" => "rs2, offset(rs1)",
        "BEQ" | "BNE" | "BLT" | "BGE" | "BLTU" | "BGEU" => "rs1, rs2, offset",
        "JAL" => "rd, offset",
        "JALR" => "rd, offset(rs1)",
        "LUI" | "AUIPC" => "rd, imm",
        _ => "<args>",
    }
}