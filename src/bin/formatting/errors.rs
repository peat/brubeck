//! Error formatting with helpful context and tips

use brubeck::HistoryError;

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
