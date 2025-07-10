//! Color utilities for the REPL
//!
//! This module provides color formatting functions to enhance the visual
//! feedback of the REPL, making it easier to see state changes.

#![allow(dead_code)]

use crossterm::style::{Color, Stylize};

/// Colors a register value based on its state
pub fn color_register_value(value: u32, is_changed: bool, is_special: Option<&str>) -> String {
    let hex_str = format!("0x{value:08x}");

    // Special registers get special colors
    if let Some(special_type) = is_special {
        return match special_type {
            "pc" => hex_str.with(Color::Red).to_string(),
            "sp" => hex_str.with(Color::Green).to_string(),
            "fp" | "s0" => hex_str.with(Color::Green).to_string(),
            _ => hex_str,
        };
    }

    // Changed registers are highlighted
    if is_changed {
        hex_str.with(Color::Yellow).to_string()
    } else if value == 0 {
        // Zero values are dimmed
        hex_str.with(Color::DarkGrey).to_string()
    } else {
        // Normal values in default color
        hex_str
    }
}

/// Colors a register name based on its type
pub fn color_register_name(name: &str, abi_name: &str) -> String {
    // Special registers get colored names too
    match abi_name {
        "sp" => format!("{} ({})", name, abi_name.with(Color::Green)),
        "fp" | "s0" => format!("{} ({})", name, abi_name.with(Color::Green)),
        "ra" => format!("{} ({})", name, abi_name.with(Color::Cyan)),
        "zero" => format!("{} ({})", name, abi_name.with(Color::DarkGrey)),
        _ => format!("{name} ({abi_name})"),
    }
}

/// Colors a memory byte based on whether it changed
pub fn color_memory_byte(byte: u8, is_changed: bool, is_pc_location: bool) -> String {
    let hex_str = format!("{byte:02x}");

    if is_pc_location {
        hex_str.with(Color::Blue).to_string()
    } else if is_changed {
        hex_str.on(Color::DarkYellow).to_string()
    } else if byte == 0 {
        hex_str.with(Color::DarkGrey).to_string()
    } else {
        hex_str
    }
}

/// Colors an ASCII character in memory display
pub fn color_ascii_char(byte: u8, is_changed: bool) -> String {
    let ch = if (0x20..=0x7E).contains(&byte) {
        byte as char
    } else {
        '.'
    };

    if is_changed {
        ch.to_string().with(Color::Yellow).to_string()
    } else if ch != '.' {
        ch.to_string().with(Color::Green).to_string()
    } else {
        ch.to_string().with(Color::DarkGrey).to_string()
    }
}

/// Colors an immediate value in instruction output
pub fn color_immediate(value: i32) -> String {
    format!("{value}").with(Color::Cyan).to_string()
}

/// Colors a branch target
pub fn color_branch_target(address: u32) -> String {
    format!("0x{address:x}").with(Color::Magenta).to_string()
}

/// Colors error message components
pub fn color_error_header(text: &str) -> String {
    text.with(Color::Red).bold().to_string()
}

pub fn color_error_detail(text: &str) -> String {
    text.with(Color::Yellow).to_string()
}

pub fn color_suggestion(text: &str) -> String {
    text.with(Color::Green).to_string()
}
