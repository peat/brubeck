//! Unit tests for the undo/redo history management system

use brubeck::history::{HistoryManager, StateSnapshot, MemoryDelta};

mod history_manager {
    use super::*;
    
    #[test]
    fn test_empty_history_undo_fails() {
        let mut history = HistoryManager::new(100);
        assert!(history.undo().is_none());
    }
    
    #[test]
    fn test_single_instruction_undo() {
        let mut history = HistoryManager::new(100);
        
        // Create a snapshot
        let snapshot = StateSnapshot {
            instruction: "ADDI x1, x0, 42".to_string(),
            registers: [0; 32],
            pc: 0,
            registers_after: [0; 32],
            pc_after: 4,
            csr_changes: vec![],
            memory_changes: vec![],
        };
        
        history.push(snapshot.clone());
        
        // Should be able to undo
        let undone = history.undo();
        assert!(undone.is_some());
        assert_eq!(undone.unwrap().instruction, "ADDI x1, x0, 42");
        
        // Should not be able to undo again
        assert!(history.undo().is_none());
    }
    
    #[test]
    fn test_multiple_undo_redo() {
        let mut history = HistoryManager::new(100);
        
        // Push three instructions
        for i in 0..3 {
            let snapshot = StateSnapshot {
                instruction: format!("ADDI x{}, x0, {}", i+1, i+10),
                registers: [0; 32],
                pc: i * 4,
                registers_after: [0; 32],
                pc_after: (i + 1) * 4,
                csr_changes: vec![],
                memory_changes: vec![],
            };
            history.push(snapshot);
        }
        
        // Undo all three
        assert_eq!(history.undo().unwrap().instruction, "ADDI x3, x0, 12");
        assert_eq!(history.undo().unwrap().instruction, "ADDI x2, x0, 11");
        assert_eq!(history.undo().unwrap().instruction, "ADDI x1, x0, 10");
        assert!(history.undo().is_none());
        
        // Redo all three
        assert_eq!(history.redo().unwrap().instruction, "ADDI x1, x0, 10");
        assert_eq!(history.redo().unwrap().instruction, "ADDI x2, x0, 11");
        assert_eq!(history.redo().unwrap().instruction, "ADDI x3, x0, 12");
        assert!(history.redo().is_none());
    }
    
    #[test]
    fn test_history_limit_enforcement() {
        let mut history = HistoryManager::new(3); // Small limit
        
        // Push 5 instructions
        for i in 0..5 {
            let snapshot = StateSnapshot {
                instruction: format!("Instruction {}", i),
                registers: [0; 32],
                pc: i * 4,
                registers_after: [0; 32],
                pc_after: (i + 1) * 4,
                csr_changes: vec![],
                memory_changes: vec![],
            };
            history.push(snapshot);
        }
        
        // Should only be able to undo 3 times (oldest 2 were dropped)
        assert_eq!(history.undo().unwrap().instruction, "Instruction 4");
        assert_eq!(history.undo().unwrap().instruction, "Instruction 3");
        assert_eq!(history.undo().unwrap().instruction, "Instruction 2");
        assert!(history.undo().is_none());
    }
    
    #[test]
    fn test_redo_cleared_on_new_instruction() {
        let mut history = HistoryManager::new(100);
        
        // Push two instructions
        history.push(StateSnapshot {
            instruction: "First".to_string(),
            registers: [0; 32],
            pc: 0,
            registers_after: [0; 32],
            pc_after: 4,
            csr_changes: vec![],
            memory_changes: vec![],
        });
        
        history.push(StateSnapshot {
            instruction: "Second".to_string(),
            registers: [0; 32],
            pc: 4,
            registers_after: [0; 32],
            pc_after: 8,
            csr_changes: vec![],
            memory_changes: vec![],
        });
        
        // Undo one
        history.undo();
        
        // Verify we can redo
        assert!(history.can_redo());
        
        // Push a new instruction
        history.push(StateSnapshot {
            instruction: "Third".to_string(),
            registers: [0; 32],
            pc: 8,
            registers_after: [0; 32],
            pc_after: 12,
            csr_changes: vec![],
            memory_changes: vec![],
        });
        
        // Redo should no longer be available
        assert!(!history.can_redo());
        assert!(history.redo().is_none());
    }
    
}

mod state_capture {
    use super::*;
    
    #[test]
    fn test_register_change_capture() {
        let old_registers = [0; 32];
        let mut new_registers = [0; 32];
        
        // Change some registers
        new_registers[1] = 42;  // x1
        new_registers[5] = 100; // x5
        
        let snapshot = StateSnapshot::capture_changes(
            "ADDI x1, x0, 42",
            &old_registers,
            &new_registers,
            0,
            4,
            vec![],
            vec![],
        );
        
        assert_eq!(snapshot.instruction, "ADDI x1, x0, 42");
        assert_eq!(snapshot.registers, old_registers);
        assert_eq!(snapshot.pc, 0);
    }
    
    #[test]
    fn test_memory_change_capture() {
        let memory_changes = vec![
            MemoryDelta {
                address: 0x1000,
                old_value: 0x00,
                new_value: 0xFF,
            },
            MemoryDelta {
                address: 0x1001,
                old_value: 0x00,
                new_value: 0xAB,
            },
        ];
        
        let snapshot = StateSnapshot {
            instruction: "SB x1, 0(x2)".to_string(),
            registers: [0; 32],
            pc: 0,
            registers_after: [0; 32],
            pc_after: 4,
            csr_changes: vec![],
            memory_changes: memory_changes.clone(),
        };
        
        assert_eq!(snapshot.memory_changes.len(), 2);
        assert_eq!(snapshot.memory_changes[0].address, 0x1000);
        assert_eq!(snapshot.memory_changes[0].new_value, 0xFF);
    }
    
    #[test]
    fn test_csr_change_capture() {
        let csr_changes = vec![
            (0x340, 0x0000_0000, 0x1234_5678), // mscratch
        ];
        
        let snapshot = StateSnapshot {
            instruction: "CSRRW x1, mscratch, x2".to_string(),
            registers: [0; 32],
            pc: 0,
            registers_after: [0; 32],
            pc_after: 4,
            csr_changes,
            memory_changes: vec![],
        };
        
        assert_eq!(snapshot.csr_changes.len(), 1);
        assert_eq!(snapshot.csr_changes[0].0, 0x340);
        assert_eq!(snapshot.csr_changes[0].2, 0x1234_5678);
    }
    
    #[test]
    fn test_pc_change_capture() {
        let snapshot = StateSnapshot {
            instruction: "JAL x1, 0x100".to_string(),
            registers: [0; 32],
            pc: 0x1000,  // PC before the jump
            registers_after: [0; 32],
            pc_after: 0x1100,  // PC after the jump
            csr_changes: vec![],
            memory_changes: vec![],
        };
        
        assert_eq!(snapshot.pc, 0x1000);
    }
}

mod state_restoration {
    use super::*;
    
    #[test]
    fn test_register_restoration() {
        // This will test that we can restore register state
        // Implementation will come with the actual StateSnapshot::restore method
        
        let mut registers = [0; 32];
        registers[1] = 42;
        registers[2] = 100;
        
        let _snapshot = StateSnapshot {
            instruction: "Test".to_string(),
            registers: [0; 32], // All zeros
            pc: 0,
            registers_after: [0; 32],
            pc_after: 4,
            csr_changes: vec![],
            memory_changes: vec![],
        };
        
        // After restore, registers should be back to all zeros
        // snapshot.restore(&mut cpu);
        // assert_eq!(cpu.get_register(Register::X1), 0);
    }
    
    #[test]
    fn test_memory_restoration() {
        let memory_changes = vec![
            MemoryDelta {
                address: 0x1000,
                old_value: 0xAB,
                new_value: 0xFF,
            },
        ];
        
        let _snapshot = StateSnapshot {
            instruction: "Test".to_string(),
            registers: [0; 32],
            pc: 0,
            registers_after: [0; 32],
            pc_after: 4,
            csr_changes: vec![],
            memory_changes,
        };
        
        // After restore, memory[0x1000] should be 0xAB (old value)
        // snapshot.restore(&mut cpu);
        // assert_eq!(cpu.read_byte(0x1000), 0xAB);
    }
    
    #[test]
    fn test_csr_restoration() {
        let csr_changes = vec![
            (0x340, 0xDEAD_BEEF, 0x1234_5678), // mscratch
        ];
        
        let _snapshot = StateSnapshot {
            instruction: "Test".to_string(),
            registers: [0; 32],
            pc: 0,
            registers_after: [0; 32],
            pc_after: 4,
            csr_changes,
            memory_changes: vec![],
        };
        
        // After restore, CSR should have old value
        // snapshot.restore(&mut cpu);
        // assert_eq!(cpu.read_csr(0x340).unwrap(), 0xDEAD_BEEF);
    }
    
    #[test]
    fn test_complete_state_restoration() {
        // Test that all state components are restored together
        let mut registers = [0; 32];
        registers[5] = 999;
        
        let _snapshot = StateSnapshot {
            instruction: "Complex instruction".to_string(),
            registers,
            pc: 0x2000,
            registers_after: registers,
            pc_after: 0x2004,
            csr_changes: vec![(0x340, 0x1111, 0x2222)],
            memory_changes: vec![
                MemoryDelta {
                    address: 0x3000,
                    old_value: 0x33,
                    new_value: 0x44,
                },
            ],
        };
        
        // After restore:
        // - PC should be 0x2000
        // - x5 should be 999
        // - CSR 0x340 should be 0x1111
        // - Memory[0x3000] should be 0x33
    }
}