//! Terminal input handling with command history support
//!
//! This module provides an event-based input system that supports
//! arrow key navigation through command history.

use super::CommandHistory;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::Print,
    terminal::{self, ClearType},
};
use std::io::{self, Write};

/// Reads a line of input with command history support
///
/// This function:
/// - Displays the prompt
/// - Handles character input and editing
/// - Supports arrow keys for history navigation
/// - Returns the final command when Enter is pressed
///
/// # Arguments
/// - `prompt`: The prompt to display
/// - `history`: The command history to navigate
pub fn read_line_with_history(prompt: &str, history: &mut CommandHistory) -> io::Result<String> {
    let mut stdout = io::stdout();
    let mut input = String::new();
    let mut cursor_position = 0;

    // Enable raw mode for direct key handling
    terminal::enable_raw_mode()?;

    // Print the prompt
    print!("{prompt}");
    stdout.flush()?;

    let result = (|| -> io::Result<String> {
        loop {
            if let Event::Key(KeyEvent {
                code, modifiers, ..
            }) = event::read()?
            {
                match code {
                    KeyCode::Enter => {
                        println!(); // New line after input
                        return Ok(input);
                    }
                    KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                        println!("^C");
                        return Err(io::Error::new(io::ErrorKind::Interrupted, "Ctrl+C"));
                    }
                    KeyCode::Char(c) => {
                        input.insert(cursor_position, c);
                        cursor_position += 1;
                        refresh_line(prompt, &input, cursor_position)?;
                    }
                    KeyCode::Backspace => {
                        if cursor_position > 0 {
                            cursor_position -= 1;
                            input.remove(cursor_position);
                            refresh_line(prompt, &input, cursor_position)?;
                        }
                    }
                    KeyCode::Delete => {
                        if cursor_position < input.len() {
                            input.remove(cursor_position);
                            refresh_line(prompt, &input, cursor_position)?;
                        }
                    }
                    KeyCode::Left => {
                        if cursor_position > 0 {
                            cursor_position -= 1;
                            refresh_line(prompt, &input, cursor_position)?;
                        }
                    }
                    KeyCode::Right => {
                        if cursor_position < input.len() {
                            cursor_position += 1;
                            refresh_line(prompt, &input, cursor_position)?;
                        }
                    }
                    KeyCode::Home => {
                        cursor_position = 0;
                        refresh_line(prompt, &input, cursor_position)?;
                    }
                    KeyCode::End => {
                        cursor_position = input.len();
                        refresh_line(prompt, &input, cursor_position)?;
                    }
                    KeyCode::Up => {
                        // Start navigation if needed
                        if !history.is_navigating() {
                            history.start_navigation(input.clone());
                        }

                        if let Some(cmd) = history.previous() {
                            input = cmd.to_string();
                            cursor_position = input.len();
                            refresh_line(prompt, &input, cursor_position)?;
                        }
                    }
                    KeyCode::Down => {
                        if history.is_navigating() {
                            if let Some(cmd) = history.next() {
                                input = cmd.to_string();
                                cursor_position = input.len();
                                refresh_line(prompt, &input, cursor_position)?;
                            }
                        }
                    }
                    KeyCode::Esc => {
                        if history.is_navigating() {
                            input = history.cancel_navigation().to_string();
                            cursor_position = input.len();
                            refresh_line(prompt, &input, cursor_position)?;
                        }
                    }
                    _ => {} // Ignore other keys
                }
            }
        }
    })();

    // Always restore terminal state
    terminal::disable_raw_mode()?;

    result
}

/// Refreshes the current line display
///
/// This clears the current line and redraws the prompt and input
/// with the cursor at the correct position.
fn refresh_line(prompt: &str, input: &str, cursor_pos: usize) -> io::Result<()> {
    let mut stdout = io::stdout();

    // Move to start of line and clear it
    execute!(
        stdout,
        cursor::MoveToColumn(0),
        terminal::Clear(ClearType::CurrentLine),
        Print(prompt),
        Print(input),
    )?;

    // Position cursor correctly
    let total_pos = prompt.len() + cursor_pos;
    execute!(stdout, cursor::MoveToColumn(total_pos as u16))?;

    stdout.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    // Note: Testing terminal interaction is complex and typically
    // requires mocking or integration tests. The core logic is tested
    // in the CommandHistory tests.
}
