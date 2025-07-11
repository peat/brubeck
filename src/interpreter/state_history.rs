use crate::rv32_i::StateDelta;

/// Simple state history for navigation
pub struct StateHistory {
    deltas: Vec<StateDelta>,
    current_position: usize,
    limit: usize,
}

impl StateHistory {
    pub fn new(limit: usize) -> Self {
        Self {
            deltas: Vec::new(),
            current_position: 0,
            limit,
        }
    }

    pub fn record_delta(&mut self, delta: StateDelta) {
        // Don't record anything if limit is 0
        if self.limit == 0 {
            return;
        }

        // If we're in the middle of history, truncate everything after current position
        self.deltas.truncate(self.current_position);

        // Add the new delta
        self.deltas.push(delta);
        self.current_position = self.deltas.len();

        // Enforce the limit by removing oldest entries
        while self.deltas.len() > self.limit {
            self.deltas.remove(0);
            self.current_position = self.current_position.saturating_sub(1);
        }
    }

    pub fn get_previous_delta(&mut self) -> Option<&StateDelta> {
        if self.current_position > 0 {
            self.current_position -= 1;
            self.deltas.get(self.current_position)
        } else {
            None
        }
    }

    pub fn get_next_delta(&mut self) -> Option<&StateDelta> {
        if self.current_position < self.deltas.len() {
            let delta = self.deltas.get(self.current_position);
            self.current_position += 1;
            delta
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.deltas.clear();
        self.current_position = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rv32_i::Register;

    fn create_test_delta(reg: Register, old_val: u32, new_val: u32) -> StateDelta {
        StateDelta {
            register_changes: vec![(reg, old_val, new_val)],
            memory_changes: vec![],
            pc_change: (0, 4),
            csr_changes: vec![],
        }
    }

    #[test]
    fn test_basic_record_and_navigate() {
        let mut history = StateHistory::new(10);

        // Record some deltas
        let delta1 = create_test_delta(Register::X1, 0, 10);
        let delta2 = create_test_delta(Register::X2, 0, 20);
        let delta3 = create_test_delta(Register::X3, 0, 30);

        history.record_delta(delta1.clone());
        history.record_delta(delta2.clone());
        history.record_delta(delta3.clone());

        // Navigate backwards
        assert_eq!(history.get_previous_delta().unwrap(), &delta3);
        assert_eq!(history.get_previous_delta().unwrap(), &delta2);
        assert_eq!(history.get_previous_delta().unwrap(), &delta1);
        assert!(history.get_previous_delta().is_none());

        // Navigate forwards
        assert_eq!(history.get_next_delta().unwrap(), &delta1);
        assert_eq!(history.get_next_delta().unwrap(), &delta2);
        assert_eq!(history.get_next_delta().unwrap(), &delta3);
        assert!(history.get_next_delta().is_none());
    }

    #[test]
    fn test_history_limit() {
        let mut history = StateHistory::new(3);

        // Record 5 deltas (exceeds limit of 3)
        for i in 1..=5 {
            let delta = create_test_delta(Register::X1, i - 1, i);
            history.record_delta(delta);
        }

        // Should only be able to go back 3 times
        assert!(history.get_previous_delta().is_some()); // Delta 5
        assert!(history.get_previous_delta().is_some()); // Delta 4
        assert!(history.get_previous_delta().is_some()); // Delta 3
        assert!(history.get_previous_delta().is_none()); // Can't go further

        // Verify the oldest deltas (1 and 2) were dropped
        let delta = history.get_next_delta().unwrap();
        assert_eq!(delta.register_changes[0].1, 2); // Old value should be 2 (from delta 3)
    }

    #[test]
    fn test_zero_limit() {
        let mut history = StateHistory::new(0);

        // Try to record a delta
        let delta = create_test_delta(Register::X1, 0, 10);
        history.record_delta(delta);

        // Nothing should be recorded
        assert!(history.get_previous_delta().is_none());
        assert!(history.get_next_delta().is_none());
    }

    #[test]
    fn test_clear() {
        let mut history = StateHistory::new(10);

        // Record some deltas
        history.record_delta(create_test_delta(Register::X1, 0, 10));
        history.record_delta(create_test_delta(Register::X2, 0, 20));

        // Clear history
        history.clear();

        // Should be empty
        assert!(history.get_previous_delta().is_none());
        assert!(history.get_next_delta().is_none());
    }

    #[test]
    fn test_truncate_on_new_branch() {
        let mut history = StateHistory::new(10);

        // Record 3 deltas
        let delta1 = create_test_delta(Register::X1, 0, 10);
        let delta2 = create_test_delta(Register::X2, 0, 20);
        let delta3 = create_test_delta(Register::X3, 0, 30);

        history.record_delta(delta1.clone());
        history.record_delta(delta2.clone());
        history.record_delta(delta3.clone());

        // Go back twice
        history.get_previous_delta(); // At delta3
        history.get_previous_delta(); // At delta2

        // Record a new delta (should truncate delta3)
        let delta4 = create_test_delta(Register::X4, 0, 40);
        history.record_delta(delta4.clone());

        // Should not be able to redo to delta3
        assert!(history.get_next_delta().is_none());

        // We're now at position 2 (after delta1 and delta4)
        // Should be able to go back to delta4, then delta1
        assert_eq!(history.get_previous_delta().unwrap(), &delta4);
        assert_eq!(history.get_previous_delta().unwrap(), &delta1);
        assert!(history.get_previous_delta().is_none());

        // And forward again
        assert_eq!(history.get_next_delta().unwrap(), &delta1);
        assert_eq!(history.get_next_delta().unwrap(), &delta4);
        assert!(history.get_next_delta().is_none());
    }

    #[test]
    fn test_position_tracking() {
        let mut history = StateHistory::new(10);

        // Record 3 deltas
        history.record_delta(create_test_delta(Register::X1, 0, 10));
        history.record_delta(create_test_delta(Register::X2, 0, 20));
        history.record_delta(create_test_delta(Register::X3, 0, 30));

        // Current position is at the end
        assert!(history.get_next_delta().is_none());

        // Go back to middle
        history.get_previous_delta();
        history.get_previous_delta();

        // Can go both ways
        assert!(history.get_previous_delta().is_some());
        assert!(history.get_next_delta().is_some());
    }
}
