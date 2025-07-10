# Undo/Redo Specification for Brubeck REPL

## Overview

This specification describes a state management system that enables undo/redo functionality in the Brubeck REPL, allowing users to reverse instruction execution and replay them. As a side benefit, this system will also enable register change highlighting.

## Requirements

### Functional Requirements

1. **Undo Command** (`/undo` or `/u`)
   - Reverts CPU to state before last instruction
   - Shows what was undone: "Undid: ADDI x1, x0, 42"
   - Can undo multiple times up to history limit

2. **Redo Command** (`/redo`)
   - Re-executes previously undone instruction
   - Only available after undo
   - Cleared when new instruction is executed

3. **State Tracking**
   - Track all CPU state changes (registers, PC, CSRs, memory)
   - Efficient storage using delta compression
   - Configurable history limit (default: 1000 states)
   - Only track successfully executed instructions

### Non-Functional Requirements

1. **Performance**
   - Minimal overhead during normal execution
   - Fast undo/redo operations (<10ms)
   - Memory efficient (delta compression)

2. **Correctness**
   - Perfect state restoration
   - No side effects from undo/redo
   - Deterministic behavior

## Design

### Data Structures

```rust
/// Represents a change to a single memory byte
#[derive(Debug, Clone)]
struct MemoryDelta {
    address: u32,
    old_value: u8,
    new_value: u8,
}

/// Captures state changes from a single instruction
#[derive(Debug, Clone)]
struct StateSnapshot {
    /// Instruction that was executed
    instruction: String,
    
    /// Register values BEFORE execution
    registers: [u32; 32],
    
    /// PC value BEFORE execution
    pc: u32,
    
    /// CSR changes (only modified CSRs)
    csr_changes: Vec<(u32, u32, u32)>, // (address, old_value, new_value)
    
    /// Memory changes (only modified bytes)
    memory_changes: Vec<MemoryDelta>,
}

/// Manages undo/redo history
pub struct HistoryManager {
    /// Ring buffer of state snapshots
    history: VecDeque<StateSnapshot>,
    
    /// Current position in history (-1 means at latest)
    current_position: isize,
    
    /// Maximum history size
    max_history: usize,
}
```

### Integration Points

1. **Interpreter Enhancement**
   ```rust
   pub struct Interpreter {
       cpu: CPU,
       #[cfg(feature = "repl")]
       history: HistoryManager,
   }
   ```

2. **CPU Modification**
   - Add methods to capture state
   - Add methods to restore state
   - Track memory writes for deltas

3. **New Commands**
   - Add `Undo` and `Redo` to Command enum
   - Update parser to recognize /undo and /redo

### State Capture Strategy

1. **Before Execution**:
   - Snapshot registers and PC
   - Note which instruction will execute

2. **During Execution**:
   - CPU tracks memory writes
   - CPU tracks CSR modifications

3. **After Execution**:
   - Collect all changes
   - Store snapshot in history
   - Trim history if over limit

## Test Plan

### Unit Tests

1. **HistoryManager Tests**
   ```rust
   #[test]
   fn test_empty_history_undo_fails()
   fn test_single_instruction_undo()
   fn test_multiple_undo_redo()
   fn test_history_limit_enforcement()
   fn test_redo_cleared_on_new_instruction()
   ```

2. **State Capture Tests**
   ```rust
   #[test]
   fn test_register_change_capture()
   fn test_memory_change_capture()
   fn test_csr_change_capture()
   fn test_pc_change_capture()
   ```

3. **State Restoration Tests**
   ```rust
   #[test]
   fn test_register_restoration()
   fn test_memory_restoration()
   fn test_csr_restoration()
   fn test_complete_state_restoration()
   ```

### Integration Tests

1. **Instruction Coverage**
   - Test undo/redo for EVERY instruction type
   - Verify correct state restoration
   - Check edge cases (e.g., store to same address twice)

2. **Sequence Tests**
   ```rust
   #[test]
   fn test_arithmetic_sequence_undo()
   fn test_memory_operations_undo()
   fn test_branch_and_jump_undo()
   fn test_csr_operations_undo()
   ```

3. **Complex Scenarios**
   ```rust
   #[test]
   fn test_loop_execution_undo()
   fn test_recursive_call_undo()
   fn test_memory_aliasing_undo()
   ```

### Property-Based Tests

```rust
#[test]
fn prop_undo_redo_identity() {
    // Property: execute -> undo -> redo = execute
}

#[test]
fn prop_multiple_undo_consistency() {
    // Property: undo(n) -> redo(n) = identity
}
```

## Implementation Plan

### Phase 1: Core Infrastructure
1. Create `StateSnapshot` and `MemoryDelta` types
2. Implement `HistoryManager` with ring buffer
3. Write comprehensive unit tests
4. Add state capture hooks to CPU

### Phase 2: Integration
1. Integrate HistoryManager into Interpreter
2. Modify CPU to track memory/CSR changes
3. Add capture points around instruction execution
4. Implement state restoration methods

### Phase 3: Commands
1. Add Undo/Redo to Command enum
2. Update parser for /undo and /redo
3. Implement command handlers
4. Add user feedback messages

### Phase 4: Enhancements
1. Implement register change highlighting
2. Add /history command to show undo history
3. Optimize memory usage with better compression
4. Add configuration for history size

## Design Decisions

1. **Invalid Instructions**: Failed instructions are NOT added to history
   - Only successfully executed instructions can be undone
   - Simplifies state management
   - More intuitive user experience

2. **Pseudo-instructions**: Track the expanded real instructions
   - Example: MV x1, x2 → track as ADDI x1, x2, 0
   - Ensures correct state restoration
   - User sees "Undid: MV x1, x2" but internally we track ADDI

3. **CSR Side Effects**: Special handling needed for certain CSRs
   - See detailed explanation below

## CSR Side Effects Explained

Some CSRs in RISC-V have special behaviors that make undo/redo tricky:

### 1. **Performance Counters** (CYCLE, TIME, INSTRET)
These CSRs auto-increment:
- `CYCLE` (0xC00): Counts CPU cycles since boot
- `TIME` (0xC01): Real-time clock counter  
- `INSTRET` (0xC02): Counts retired instructions

**Problem**: If you read CYCLE, undo, then redo, the value will be different!
```
CSRRS x1, cycle, x0  # x1 = 1000
/undo                # Undo the read
/redo                # x1 = 1050 (time has passed!)
```

### 2. **Interrupt/Exception CSRs**
Some CSRs clear on read:
- `MCAUSE`: Might clear pending interrupt bits
- Some vendor-specific CSRs

**Problem**: Reading the CSR changes its state!

### 3. **Our Solution**
For now, we'll implement a simple approach:
1. **Save exact values**: When we undo/redo, restore the exact values that were read/written
2. **Document limitations**: Make it clear that time-based CSRs won't reflect "real" time after undo/redo
3. **Future enhancement**: Could special-case certain CSRs if needed

This means:
- Undo/redo will be "logically correct" (same values)
- But not "temporally correct" (time won't rewind)
- This is acceptable for a learning/debugging tool

## Success Criteria

1. All tests pass (100% coverage of state management)
2. Undo/redo works correctly for all RV32I instructions
3. Performance overhead < 5% during normal execution
4. Memory usage scales linearly with history size
5. User experience is intuitive and helpful