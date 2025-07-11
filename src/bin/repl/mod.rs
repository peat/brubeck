//! REPL-specific functionality for Brubeck
//!
//! This module contains features that enhance the interactive REPL experience
//! but are not part of the core emulation library.

mod history;
mod input;

pub use history::CommandHistory;
pub use input::read_line_with_history;
