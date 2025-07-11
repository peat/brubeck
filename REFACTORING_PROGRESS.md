# Interpreter API Refactoring Progress

## Summary

We've successfully refactored the Interpreter's public API to separate concerns between the library and binary. The library now returns data (StateDelta) instead of formatted strings, making it more flexible and suitable for different frontends.

## Completed Phases (1-5)

### Phase 1: Created New Error Types ✅
- Added `ParseError`, `HistoryError`, and `ExecutionError` enums
- Structured error information (instruction names, expected vs actual counts, etc.)
- Re-exported `CPUError` for convenience
- Tests added in `tests/error_types.rs`

### Phase 2: Removed String Formatting from Library ✅
- Deleted `src/interpreter/formatter.rs`
- Updated executor to return `StateDelta` instead of strings
- Added temporary string conversion in `interpret_single` for compatibility

### Phase 3: Made parse() Static ✅
- Created new static `parse()` function that returns `Vec<Instruction>`
- Handles pseudo-instruction expansion transparently
- Converts internal errors to public `ParseError` type
- Tests added in `tests/parse_api.rs`

### Phase 4: Updated Methods to Return StateDelta ✅
- Added `execute_instruction()` returning `StateDelta`
- Added `previous_state()` and `next_state()` returning `StateDelta`
- Added `reset()` method to clear state while preserving config
- Deprecated old string-returning methods
- Tests added in `tests/interpreter_reset.rs`

### Phase 5: Cleaned Up Public API ✅
- Made `cpu` field public
- Removed `cpu()`, `cpu_mut()`, and `get_pc()` methods
- Removed deprecated methods (old execute, previous_state, next_state)
- Removed `clear_history()` (use reset() instead)
- Hid internal types (Command, Token) from public API
- Updated documentation examples to use new API

## New Public API

```rust
// Static parsing function
pub fn parse(input: &str) -> Result<Vec<Instruction>, ParseError>;

pub struct Interpreter {
    pub cpu: CPU,  // Direct access to CPU
}

impl Interpreter {
    // Construction
    pub fn new() -> Self;
    pub fn with_config(memory_size: usize, history_limit: usize) -> Self;
    
    // Execution
    pub fn execute_instruction(&mut self, instruction: Instruction) -> Result<StateDelta, CPUError>;
    
    // History navigation
    pub fn previous_state(&mut self) -> Result<StateDelta, HistoryError>;
    pub fn next_state(&mut self) -> Result<StateDelta, HistoryError>;
    
    // State management
    pub fn reset(&mut self);
    
    // Legacy method (still used by binary, to be removed)
    pub fn interpret(&mut self, input: &str) -> Result<String, Error>;
}
```

## Breaking Changes

1. **Two-stage process**: Must call `parse()` then `execute_instruction()`
2. **Direct CPU access**: Use `interpreter.cpu` instead of `interpreter.cpu()`
3. **Data not strings**: Methods return `StateDelta` for programmatic use
4. **Error types**: New structured errors instead of generic strings
5. **No semicolons**: Library only handles single instructions

## What's Left

### Phase 6: Update Tests
- Many tests still use old `interpret()` method
- Need to update to use `parse()` + `execute_instruction()`
- Check `StateDelta` contents instead of string output

### Phase 7: Update Binary
- Move formatting logic from library to binary
- Update REPL to use new two-stage process
- Handle display of `StateDelta` information
- Continue to support semicolon-separated commands

## Benefits Achieved

1. **Clean separation**: Library provides data, binary handles presentation
2. **Flexible API**: Can parse without executing, inspect state changes
3. **Better errors**: Structured information for better error handling
4. **Simpler interface**: Fewer methods, clearer responsibilities
5. **Future-proof**: Easy to add features without breaking changes