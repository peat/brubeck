//! Unit tests for REPL command history functionality
//!
//! These tests verify the command history system that allows users to navigate
//! through previously executed commands using arrow keys.
//!
//! Note: These tests are distinct from the undo/redo history tests in history.rs

// Import the REPL-specific history module
#[path = "../../src/bin/repl/history.rs"]
mod history;

use history::CommandHistory;

#[test]
fn test_empty_history() {
    let mut history = CommandHistory::new(100);
    assert!(history.is_empty());
    assert_eq!(history.len(), 0);
    assert!(history.previous().is_none());
}

#[test]
fn test_add_command() {
    let mut history = CommandHistory::new(100);
    history.add("first".to_string());
    assert_eq!(history.len(), 1);

    history.add("second".to_string());
    assert_eq!(history.len(), 2);
}

#[test]
fn test_ignore_empty_commands() {
    let mut history = CommandHistory::new(100);
    history.add("".to_string());
    history.add("  ".to_string());
    history.add("\n".to_string());
    assert_eq!(history.len(), 0);
}

#[test]
fn test_deduplication() {
    let mut history = CommandHistory::new(100);
    history.add("test".to_string());
    history.add("test".to_string());
    history.add("test".to_string());
    assert_eq!(history.len(), 1);

    history.add("different".to_string());
    assert_eq!(history.len(), 2);

    history.add("different".to_string());
    assert_eq!(history.len(), 2);
}

#[test]
fn test_size_limit() {
    let mut history = CommandHistory::new(3);
    for i in 0..5 {
        history.add(format!("command {i}"));
    }
    assert_eq!(history.len(), 3);

    // Should have kept the 3 most recent
    history.start_navigation(String::new());
    assert_eq!(history.previous(), Some("command 4"));
    assert_eq!(history.previous(), Some("command 3"));
    assert_eq!(history.previous(), Some("command 2"));
    assert!(history.previous().is_none());
}

#[test]
fn test_navigation() {
    let mut history = CommandHistory::new(100);
    history.add("first".to_string());
    history.add("second".to_string());
    history.add("third".to_string());

    // Start navigation with working command
    history.start_navigation("working".to_string());

    // Navigate backward
    assert_eq!(history.previous(), Some("third"));
    assert_eq!(history.previous(), Some("second"));
    assert_eq!(history.previous(), Some("first"));
    assert!(history.previous().is_none()); // At oldest

    // Navigate forward
    assert_eq!(history.next(), Some("second"));
    assert_eq!(history.next(), Some("third"));
    assert_eq!(history.next(), Some("working")); // Back to working
    assert!(history.next().is_none()); // Already at working
}

#[test]
fn test_cancel_navigation() {
    let mut history = CommandHistory::new(100);
    history.add("command".to_string());

    history.start_navigation("working".to_string());
    history.previous();
    assert!(history.is_navigating());

    assert_eq!(history.cancel_navigation(), "working");
    assert!(!history.is_navigating());
}

#[test]
fn test_navigation_state_reset_on_add() {
    let mut history = CommandHistory::new(100);
    history.add("first".to_string());

    // Start navigation
    history.start_navigation("working".to_string());
    history.previous();
    assert!(history.is_navigating());

    // Adding a new command should reset navigation state
    history.add("new".to_string());
    assert!(!history.is_navigating());
}
