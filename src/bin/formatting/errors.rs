//! Error formatting with helpful context and tips

use brubeck::HistoryError;
use std::io;

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

/// Formats an IO error with context-specific help
pub fn format_io_error(error: &io::Error, context: &str, tips_enabled: bool) -> String {
    let mut output = format!("{context}: {error}");

    if tips_enabled {
        match error.kind() {
            io::ErrorKind::NotFound => {
                if context.contains("script") {
                    output.push_str(
                        "\n💡 Tip: Check that the script file path is correct and the file exists",
                    );
                }
            }
            io::ErrorKind::PermissionDenied => {
                output.push_str("\n💡 Tip: Check that you have permission to read the file");
            }
            io::ErrorKind::InvalidInput => {
                if context.contains("memory") {
                    output.push_str(
                        "\n💡 Tip: Valid memory sizes: 1M, 256k, 4096. Use suffixes k/K/m/M",
                    );
                }
            }
            _ => {}
        }
    }

    output
}

/// Formats a parse error with helpful examples
pub fn format_parse_error(error: &str, context: &str, tips_enabled: bool) -> String {
    let mut output = error.to_string();

    if tips_enabled && context == "memory_size" {
        output.push_str("\n💡 Tip: Valid memory sizes examples: 1M (1 megabyte), 256k (256 kilobytes), 4096 (4096 bytes)");
    }

    output
}

/// Formats a REPL command error with helpful suggestions
pub fn format_repl_command_error(error: &str, tips_enabled: bool) -> String {
    let mut output = error.to_string();

    if tips_enabled {
        if error.contains("Unknown command") {
            output.push_str("\n💡 Tip: Available commands: /regs, /memory, /previous, /next, /reset, /help, /quit. Use /help for details");
        } else if error.contains("Invalid register") {
            output.push_str("\n💡 Tip: Valid registers: x0-x31, or ABI names like ra, sp, a0, t0. Use /regs to see all");
        } else if error.contains("Invalid address") {
            output.push_str(
                "\n💡 Tip: Addresses can be decimal (100), hex (0x64), or binary (0b1100100)",
            );
        } else if error.contains("Memory range too large") {
            output.push_str(
                "\n💡 Tip: For larger ranges, use multiple /memory commands or adjust the range",
            );
        }
    }

    output
}