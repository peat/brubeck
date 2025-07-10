//! Unit tests for core components
//!
//! Run with: cargo test --test unit_components

mod unit;

#[cfg(feature = "repl")]
#[path = "unit/history.rs"]
mod history;

#[cfg(feature = "repl")]
#[path = "unit/cli.rs"]
mod cli;
