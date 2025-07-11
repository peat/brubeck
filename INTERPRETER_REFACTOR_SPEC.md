# Interpreter Public API Refactoring Specification

## Overview
Refactor the Interpreter's public interface to provide a cleaner, more focused API that separates parsing from execution and returns structured data instead of formatted strings.

## Current API Problems
1. Returns formatted strings instead of structured data
2. Mixes parsing and execution in a single step
3. Exposes internal implementation details (cpu_mut, history_mut)
4. Generic error types don't indicate failure source
5. Pseudo-instruction vs instruction distinction leaks to users

## New Public API

```rust
pub struct Interpreter {
    pub cpu: CPU,           // Direct read-only access
    history: StateHistory,  // Private
    initial_config: Config, // Private, for reset()
}

impl Interpreter {
    // === Initialization ===
    pub fn new() -> Self;
    pub fn with_config(memory_size: usize, history_limit: usize) -> Self;
    
    // === Core Operations ===
    // Parse assembly text (including pseudo-instructions) into real instructions
    // This is a static method - no self parameter
    pub fn parse(input: &str) -> Result<Vec<Instruction>, ParseError>;
    
    // Execute a single instruction and record in history
    pub fn execute(&mut self, instruction: Instruction) -> Result<StateDelta, CPUError>;
    
    // === History Navigation ===
    pub fn previous_state(&mut self) -> Result<StateDelta, HistoryError>;
    pub fn next_state(&mut self) -> Result<StateDelta, HistoryError>;
    
    // === State Management ===
    pub fn reset(&mut self);
}
```

## Error Types

```rust
pub enum ParseError {
    UnknownInstruction { 
        instruction: String, 
        suggestion: Option<String> 
    },
    InvalidRegister { 
        register: String 
    },
    WrongArgumentCount { 
        instruction: String, 
        expected: usize, 
        found: usize 
    },
    ImmediateOutOfRange { 
        instruction: String, 
        value: i32, 
        min: i32, 
        max: i32 
    },
    SyntaxError { 
        message: String 
    },
}

pub enum HistoryError {
    AtBeginning,  // Can't go further back
    AtEnd,        // Can't go further forward
}

// CPUError already exists in rv32_i module
pub use rv32_i::CPUError;
```

## Key Design Decisions

1. **Two-stage process**: Parse and execute are separate operations
2. **StateDelta returns**: All state-changing methods return what changed
3. **No string formatting**: Library returns data, binary handles presentation
4. **Direct CPU access**: Public field instead of getter method
5. **Pseudo-instructions transparent**: Parse expands them to real instructions
6. **Static parse method**: Doesn't need interpreter state

## Usage Examples

```rust
// Basic usage
let mut interpreter = Interpreter::new();

// Parse assembly (could be instruction or pseudo-instruction)
let instructions = Interpreter::parse("LI x1, 0x12345")?;  // Returns 2 instructions

// Execute each instruction
for inst in instructions {
    let delta = interpreter.execute(inst)?;
    println!("Changed {} registers", delta.register_changes.len());
}

// Direct CPU access
let pc = interpreter.cpu.pc;
let x1 = interpreter.cpu.get_register(Register::X1);

// History navigation
match interpreter.previous_state() {
    Ok(delta) => println!("Undid changes to {} registers", delta.register_changes.len()),
    Err(HistoryError::AtBeginning) => println!("Already at beginning"),
    Err(HistoryError::AtEnd) => unreachable!(),
}

// Reset to initial state
interpreter.reset();
```

## Implementation Plan

1. **Phase 1: Error Types**
   - Create new error enums
   - Update existing code to use them

2. **Phase 2: Remove String Formatting**
   - Move formatter.rs functionality to binary
   - Update executor to return StateDelta

3. **Phase 3: Refactor Parser**
   - Make parse() static
   - Handle pseudo-instruction expansion internally
   - Return Vec<Instruction>

4. **Phase 4: Update Public Methods**
   - Change execute() to return StateDelta
   - Update history navigation methods
   - Add reset() method
   - Store initial config

5. **Phase 5: Cleanup**
   - Remove deprecated methods (cpu_mut, get_pc, clear_history, interpret)
   - Make cpu field public
   - Hide internal types (Command enum)

6. **Phase 6: Update Tests**
   - Rewrite tests to check StateDelta instead of strings
   - Update test structure for new API

7. **Phase 7: Update Binary**
   - Move formatting logic from library
   - Update REPL to use new two-stage process

## Breaking Changes

All existing code using the Interpreter will need updates:
- Parse and execute are now separate steps
- No more string outputs - must handle StateDelta
- Error types have changed
- Some methods removed (cpu_mut, get_pc, etc.)

## Benefits

1. **Cleaner separation of concerns**: Library handles execution, binary handles presentation
2. **Better error handling**: Know exactly what failed
3. **More flexible**: Can parse without executing, can use StateDelta data however needed
4. **Simpler API**: Fewer methods, clearer purpose
5. **Future-proof**: Easy to add new features without breaking API