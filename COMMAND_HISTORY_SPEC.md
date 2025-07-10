# Command History Specification for Brubeck REPL

## Overview

This specification outlines the implementation of command history functionality for the Brubeck RISC-V REPL, allowing users to navigate through previously executed commands using up/down arrow keys.

## 1. User Experience

### Basic Navigation
- **Up Arrow (↑)**: Navigate to previous command in history
- **Down Arrow (↓)**: Navigate to next command in history
- **Enter**: Execute current command (and add to history if not duplicate)
- **Escape**: Clear current line

### What Gets Saved
- All successfully parsed commands (both instructions and `/` commands)
- Commands are saved AFTER successful parsing, not execution
- Empty lines and whitespace-only input are NOT saved
- Comments (lines starting with `#`) are NOT saved

### Deduplication
- Consecutive duplicate commands are not added to history
- Example: Executing "ADDI x1, x0, 42" twice in a row only adds one entry

### History Behavior
- History is maintained in memory during REPL session
- Default history size: 1000 commands (configurable via CLI)
- When limit reached, oldest commands are removed (FIFO)
- Current line buffer is preserved when navigating history
- Returning to "present" (past last command) restores original input

### Edge Cases
1. **Empty History**: Up/down arrows do nothing
2. **At History Boundaries**: 
   - At oldest: Up arrow does nothing
   - At newest: Down arrow returns to current input
3. **Partial Input**: Current partial input is saved when navigating away
4. **Multi-line Commands**: Not supported (single line only)

## 2. Technical Architecture

### Terminal Input Handling
- Use `crossterm` for raw mode terminal control
- Handle keyboard events directly instead of `read_line()`
- Maintain cursor position for line editing

### Storage Architecture
```rust
pub struct CommandHistory {
    entries: VecDeque<String>,
    max_size: usize,
    current_index: Option<usize>,
    saved_line: String,  // Preserves current input when navigating
}
```

### Integration Points
1. **Binary Level** (`src/bin/brubeck.rs`):
   - Replace `io::stdin().read_line()` with event-based input
   - Add `CommandHistory` instance to interactive loop
   - Handle special keys (arrows, escape)

2. **Feature Flag**: 
   - Extend existing `repl` feature to include history
   - No changes to core library

3. **CLI Integration** (`src/cli.rs`):
   - Add `--history-size <n>` flag (default: 1000)
   - Add `--no-history` flag to disable

## 3. Implementation Details

### Dependencies
- Already have `crossterm = "0.27"` - no new dependencies needed
- Feature-gated behind `repl` feature

### File Structure Changes
```
src/
├── bin/
│   └── brubeck.rs         # Modified: Event-based input loop
├── repl/                  # New module
│   ├── mod.rs            # Module exports
│   ├── history.rs        # CommandHistory implementation
│   └── input.rs          # Terminal input handling
└── cli.rs                # Modified: History configuration
```

### Key Functions

#### Input Handler
```rust
// src/repl/input.rs
pub fn read_line_with_history(
    prompt: &str,
    history: &mut CommandHistory,
) -> io::Result<String> {
    // 1. Enable raw mode
    // 2. Print prompt
    // 3. Event loop:
    //    - Handle character input
    //    - Handle special keys (arrows, backspace, etc.)
    //    - Update display
    // 4. Restore terminal mode
    // 5. Return final line
}
```

#### History Management
```rust
// src/repl/history.rs
impl CommandHistory {
    pub fn new(max_size: usize) -> Self;
    pub fn add(&mut self, command: String);
    pub fn previous(&mut self) -> Option<&str>;
    pub fn next(&mut self) -> Option<&str>;
    pub fn reset_navigation(&mut self);
}
```

#### Modified REPL Loop
```rust
// src/bin/brubeck.rs
fn run_interactive(interpreter: &mut Interpreter, quiet: bool) -> io::Result<()> {
    let mut history = CommandHistory::new(config.history_size);
    
    loop {
        let prompt = format!("[0x{:08x}]> ", interpreter.get_pc());
        
        // Use new input handler
        let input = read_line_with_history(&prompt, &mut history)?;
        
        if input.trim().is_empty() {
            continue;
        }
        
        // Add to history if parsing succeeds
        if interpreter.parse(&input).is_ok() {
            history.add(input.clone());
        }
        
        execute_and_print(interpreter, &input, true, quiet, false)?;
    }
}
```

### Terminal Handling Details
1. **Raw Mode**: Required for capturing arrow keys
2. **Line Buffer**: Maintain current line with cursor position
3. **Display Updates**: Redraw line after each keystroke
4. **Signal Handling**: Properly restore terminal on Ctrl-C

## 4. Testing Strategy

### Unit Tests (`src/repl/history.rs`)
```rust
#[cfg(test)]
mod tests {
    // Test history addition and limits
    // Test navigation (previous/next)
    // Test deduplication
    // Test boundary conditions
}
```

### Integration Tests
Since terminal interaction is difficult to test automatically:

1. **Mock Input Stream**: Create test helper that simulates key events
2. **Scenario Testing**: Test common usage patterns
3. **Manual Testing Checklist**:
   - [ ] Up/down navigation works
   - [ ] History persists across commands
   - [ ] Deduplication works
   - [ ] Boundaries handled correctly
   - [ ] Ctrl-C properly restores terminal

### Test Cases
1. **Empty History**: Verify arrows do nothing
2. **Single Command**: Up shows command, down returns to empty
3. **Multiple Commands**: Full navigation cycle
4. **History Limit**: Verify FIFO behavior
5. **Duplicate Commands**: Verify deduplication
6. **Special Characters**: Test with complex input
7. **Terminal Resize**: Ensure display remains correct

## 5. Implementation Plan

### Phase 1: Core History (2 hours)
- Implement `CommandHistory` struct
- Add unit tests
- Basic add/navigate functionality

### Phase 2: Terminal Input (3 hours)
- Implement `read_line_with_history()`
- Handle arrow keys and basic editing
- Test raw mode handling

### Phase 3: Integration (2 hours)
- Modify REPL loop
- Add CLI flags
- Update help text

### Phase 4: Polish (1 hour)
- Handle edge cases
- Add comprehensive error handling
- Update documentation

## 6. Future Enhancements

1. **Persistent History**: Save to `~/.brubeck_history`
2. **Search**: Ctrl-R for reverse search
3. **History Command**: `/history` to view command history
4. **Smart Completion**: Tab completion using history
5. **Multi-line Support**: For future script-like features

## 7. Risks and Mitigations

1. **Terminal Compatibility**: 
   - Risk: Different terminals handle keys differently
   - Mitigation: Use crossterm's abstraction layer

2. **Signal Handling**:
   - Risk: Terminal left in raw mode after crash
   - Mitigation: Proper cleanup in drop handlers

3. **Performance**:
   - Risk: Large history could slow navigation
   - Mitigation: VecDeque for O(1) operations

## 8. Success Criteria

- [ ] Up/down arrows navigate command history
- [ ] Current input preserved when navigating
- [ ] History limited to configured size
- [ ] No duplicate consecutive commands
- [ ] Terminal properly restored on exit
- [ ] Works on macOS, Linux, and Windows
- [ ] No impact on non-interactive modes