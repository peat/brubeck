//! Undo/redo history management for the REPL
//!
//! This module provides state snapshot and restoration functionality,
//! enabling users to undo and redo instruction execution.

use std::collections::VecDeque;

/// Represents a change to a single memory byte
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDelta {
    pub address: u32,
    pub old_value: u8,
    pub new_value: u8,
}

/// Captures state changes from a single instruction
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    /// Instruction that was executed (for display)
    pub instruction: String,
    
    /// Register values BEFORE execution
    pub registers: [u32; 32],
    
    /// PC value BEFORE execution
    pub pc: u32,
    
    /// Register values AFTER execution (for redo)
    pub registers_after: [u32; 32],
    
    /// PC value AFTER execution (for redo)
    pub pc_after: u32,
    
    /// CSR changes: (address, old_value, new_value)
    pub csr_changes: Vec<(u32, u32, u32)>,
    
    /// Memory changes (only modified bytes)
    pub memory_changes: Vec<MemoryDelta>,
}

impl StateSnapshot {
    /// Captures the changes between old and new state
    pub fn capture_changes(
        instruction: &str,
        old_registers: &[u32; 32],
        new_registers: &[u32; 32],
        old_pc: u32,
        new_pc: u32,
        csr_changes: Vec<(u32, u32, u32)>,
        memory_changes: Vec<MemoryDelta>,
    ) -> Self {
        Self {
            instruction: instruction.to_string(),
            registers: *old_registers,
            pc: old_pc,
            registers_after: *new_registers,
            pc_after: new_pc,
            csr_changes,
            memory_changes,
        }
    }
}

/// Manages undo/redo history with a ring buffer
pub struct HistoryManager {
    /// Ring buffer of state snapshots
    history: VecDeque<StateSnapshot>,
    
    /// Current position in history
    /// -1 means we're at the latest state (nothing to redo)
    /// 0..history.len()-1 means we've undone some operations
    current_position: isize,
    
    /// Maximum history size
    max_history: usize,
}

impl HistoryManager {
    /// Creates a new history manager with the specified maximum size
    pub fn new(max_history: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_history),
            current_position: -1,
            max_history,
        }
    }
    
    /// Adds a new state snapshot to history
    pub fn push(&mut self, snapshot: StateSnapshot) {
        // If we're not at the latest position, we need to clear redo history
        if self.current_position >= 0 && self.current_position < (self.history.len() - 1) as isize {
            // Remove everything after current position
            let keep = (self.current_position + 1) as usize;
            self.history.truncate(keep);
        }
        
        // Add the new snapshot
        self.history.push_back(snapshot);
        
        // Enforce history limit
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }
        
        // Update position to latest
        self.current_position = (self.history.len() - 1) as isize;
    }
    
    /// Undoes the last operation, returning the snapshot if successful
    pub fn undo(&mut self) -> Option<&StateSnapshot> {
        // If history is empty, nothing to undo
        if self.history.is_empty() {
            return None;
        }
        
        // If we're at the latest state (position == len - 1), we can undo
        // If we're at an earlier state (from previous undos), we can still go back
        if self.current_position >= 0 {
            let snapshot = &self.history[self.current_position as usize];
            self.current_position -= 1;
            return Some(snapshot);
        }
        
        None
    }
    
    /// Redoes a previously undone operation
    pub fn redo(&mut self) -> Option<&StateSnapshot> {
        if self.current_position < (self.history.len() - 1) as isize {
            self.current_position += 1;
            Some(&self.history[self.current_position as usize])
        } else {
            None
        }
    }
    
    /// Returns true if redo is available
    pub fn can_redo(&self) -> bool {
        self.current_position < (self.history.len() - 1) as isize
    }
    
    /// Returns the current position in history (for testing)
    #[cfg(test)]
    pub fn current_position(&self) -> isize {
        self.current_position
    }
}