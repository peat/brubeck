//! Formatting module for displaying structured data from the library
//!
//! This module contains formatters that convert the library's structured
//! data types (StateDelta, errors, etc.) into human-readable strings for
//! display in the REPL.

pub mod errors;
pub mod memory;
pub mod registers;
pub mod state_delta;

pub use errors::format_error;
pub use memory::format_memory_range;
pub use registers::format_registers;
pub use state_delta::format_state_delta;