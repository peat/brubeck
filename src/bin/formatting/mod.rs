//! Formatting module for displaying structured data from the library
//!
//! This module contains formatters that convert the library's structured
//! data types (StateDelta, errors, etc.) into human-readable strings for
//! display in the REPL.

pub mod errors;
pub mod help;
pub mod memory;
pub mod registers;
pub mod state_delta;