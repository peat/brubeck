//! Command history management for the REPL
//!
//! This module provides a command history system that allows users to navigate
//! through previously executed commands using arrow keys.

use std::collections::VecDeque;

/// Manages command history for the REPL
#[derive(Debug)]
pub struct CommandHistory {
    /// Storage for command entries, newest at front
    entries: VecDeque<String>,
    /// Maximum number of entries to keep
    max_size: usize,
    /// Current position when navigating (None = at current input)
    current_position: Option<usize>,
    /// The working command being edited before history navigation
    working_command: String,
}

impl CommandHistory {
    /// Creates a new command history with specified maximum size
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_size),
            max_size,
            current_position: None,
            working_command: String::new(),
        }
    }

    /// Adds a command to history
    ///
    /// - Empty commands are ignored
    /// - Duplicates of the most recent command are ignored
    /// - Oldest entries are removed if at capacity
    pub fn add(&mut self, command: String) {
        // Reset navigation state
        self.current_position = None;
        self.working_command.clear();

        // Ignore empty commands
        if command.trim().is_empty() {
            return;
        }

        // Ignore if same as most recent
        if let Some(last) = self.entries.front() {
            if last == &command {
                return;
            }
        }

        // Add to front
        self.entries.push_front(command);

        // Enforce size limit
        while self.entries.len() > self.max_size {
            self.entries.pop_back();
        }
    }

    /// Starts navigation by saving the current working command
    pub fn start_navigation(&mut self, working_command: String) {
        if self.current_position.is_none() {
            self.working_command = working_command;
            self.current_position = None;
        }
    }

    /// Navigates to the previous (older) command
    ///
    /// Returns None if at the oldest command
    pub fn previous(&mut self) -> Option<&str> {
        if self.entries.is_empty() {
            return None;
        }

        match self.current_position {
            None => {
                // Starting navigation, go to most recent
                self.current_position = Some(0);
                self.entries.front().map(|s| s.as_str())
            }
            Some(pos) => {
                if pos + 1 < self.entries.len() {
                    self.current_position = Some(pos + 1);
                    self.entries.get(pos + 1).map(|s| s.as_str())
                } else {
                    // Already at oldest
                    None
                }
            }
        }
    }

    /// Navigates to the next (newer) command
    ///
    /// Returns None if at the newest (returns to working command)
    pub fn next(&mut self) -> Option<&str> {
        match self.current_position {
            None => None, // Already at working command
            Some(0) => {
                // Return to working command
                self.current_position = None;
                Some(&self.working_command)
            }
            Some(pos) => {
                self.current_position = Some(pos - 1);
                self.entries.get(pos - 1).map(|s| s.as_str())
            }
        }
    }

    /// Cancels navigation and returns to working command
    pub fn cancel_navigation(&mut self) -> &str {
        self.current_position = None;
        &self.working_command
    }

    /// Returns true if currently navigating history
    pub fn is_navigating(&self) -> bool {
        self.current_position.is_some()
    }

    /// Clears all history
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_position = None;
        self.working_command.clear();
    }

    /// Returns the number of entries in history
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if history is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            history.add(format!("command {}", i));
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
}
