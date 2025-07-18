//! Formatting module for displaying structured data from the library
//!
//! This module contains formatters that convert the library's structured
//! data types (StateDelta, errors, etc.) into human-readable strings for
//! display in the REPL.
//!
//! ## Color Support
//! Several formatters support color highlighting to make output more readable:
//! - **Registers**: Zero values in gray, changed values in green
//! - **Memory**: Changed bytes in green, zeros in gray, PC location highlighted
//!
//! Colors are applied using ANSI escape codes via the crossterm library.

pub mod errors;
pub mod help;
pub mod memory;
pub mod registers;
pub mod state_delta;
