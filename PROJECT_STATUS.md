# Brubeck Project Status & Roadmap

Last Updated: 2025-07-11

## 🎯 Current Focus: Enhanced Learning Features

With the library/binary separation complete, focusing on educational improvements and usability.

## ✅ Major Refactoring: Library/Binary Separation [COMPLETED]

### Background
The codebase previously used `#[cfg(feature = "repl")]` flags throughout the library to conditionally include REPL-specific functionality. This created complexity and made the library less suitable for embedding. The refactoring created a clean separation:

- **Library**: Pure RISC-V emulation with state navigation (no feature flags)
- **Binary**: All interactive REPL features

### Architecture Design

#### Library Interface (Minimal & Focused)
```rust
impl Interpreter {
    pub fn interpret(&mut self, input: &str) -> Result<String, Error>  // Execute instruction
    pub fn previous_state(&mut self) -> Result<String, Error>          // Navigate backward
    pub fn next_state(&mut self) -> Result<String, Error>              // Navigate forward
    pub fn cpu(&self) -> &CPU                                          // Access for inspection
    pub fn cpu_mut(&mut self) -> &mut CPU                              // Mutable access
}
```

#### Binary Responsibilities (All Interactive Features)
- Parse and handle all slash commands (`/regs`, `/help`, `/memory`, `/reset`, `/previous`, `/next`)
- Format output (registers, memory, help text)
- Handle I/O (colors, prompts, confirmations)
- Manage CLI arguments and configuration

### Implementation Tasks

#### Phase 1: Extract Command System ⏱️ ~4 hours
**Status**: ✅ Completed
1. Remove `Command` enum variants from library:
   - Keep only `Exec` and `ExecPseudo` in library
   - Move `ShowRegs`, `ShowSpecificRegs`, `ShowHelp`, `Previous`, `Next`, `Reset`, `ShowMemory` to binary
2. Create binary-specific command parser for slash commands
3. Update library parser to only handle instruction parsing

#### Phase 2: Move Formatting to Binary ⏱️ ~3 hours
**Status**: ✅ Completed
1. Move from `src/interpreter/formatter.rs` to binary:
   - `format_all_registers()`
   - `format_specific_registers()`
   - `format_help()`
   - `format_memory()`
2. Keep only `format_instruction_result()` in library
3. Create `src/bin/repl_formatter.rs` for REPL-specific formatting

#### Phase 3: Simplify Interpreter ⏱️ ~3 hours
**Status**: ✅ Completed
1. Remove `history: Option<HistoryManager>` field
2. Rename methods:
   - `undo()` → `previous_state()`
   - `redo()` → `next_state()`
3. Store simple `Vec<StateDelta>` with configurable limit
4. Remove `with_config()` method - move configuration to binary

#### Phase 4: Clean Up Library Modules ⏱️ ~2 hours
**Status**: ✅ Completed
1. Remove all `#[cfg(feature = "repl")]` from:
   - `src/lib.rs`
   - `src/interpreter.rs`
   - `src/interpreter/*.rs`
2. Move interactive functions (reset confirmation, etc.) to binary
3. Remove `src/cli.rs` from library, keep only in binary

#### Phase 5: Update Tests ⏱️ ~2 hours
**Status**: ✅ Completed
1. Update library tests to work without feature flags
2. Move REPL-specific tests to binary tests
3. Ensure integration tests still pass

### Success Criteria
- [x] Library builds with zero feature flags
- [x] Library exports minimal, focused API
- [x] Binary contains all REPL/interactive features
- [x] All tests pass
- [x] No breaking changes for end users

## 📋 Task List

### 🔴 High Priority (Architecture)

#### 1. Rename Undo/Redo Commands ⏱️ ~2 hours
**Status**: ✅ Completed  
**Why**: Current names imply "fixing mistakes" rather than "navigating history"  
**Changes**:
- `/undo` → `/previous` (aliases: `/prev`, `/p`)
- `/redo` → `/next` (aliases: `/n`)
- Update parser, help text, error messages, and tests
- Update documentation

#### 2. Memory Inspection Command ⏱️ ~4 hours
**Status**: ✅ Completed  
**Why**: Critical for debugging loads/stores and understanding memory  
**Changes**:
- Added `/memory` command with alias `/m`
- Three modes: no args (around PC), single address, or range
- Displays memory in hex format with ASCII representation
- 16-byte aligned display with separators
- Shows current PC if in displayed range
- Comprehensive tests added

#### 3. Reset Command ⏱️ ~2 hours
**Status**: ✅ Completed  
**Why**: Users need to start fresh without restarting REPL  
**Changes**:
- Added `/reset` command to parser
- Prompts for confirmation before resetting
- CPU::reset() method handles all state clearing
- HistoryManager::clear() clears execution history
- Comprehensive test coverage added

### 🟡 Medium Priority (Enhanced Learning)

#### 4. Enhanced Error Messages ⏱️ ~3 hours
**Status**: Not started  
**Why**: Turn errors into learning opportunities  
**Examples**:
- Add "Did you mean?" suggestions
- Include mini-tutorials in errors
- Link to RISC-V concepts

#### 5. Instruction History Command ⏱️ ~2 hours
**Status**: Not started  
**Why**: See execution flow and learn from it  
**Specification**:
- `/history [n]` - Show last n instructions (default: 10)
- Show PC, instruction, and effect
- Alias: `/hist`

#### 6. Review and Update README.md ⏱️ ~1 hour
**Status**: ✅ Completed  
**Why**: Ensure README accurately reflects current features and capabilities  
**Tasks**:
- Review for accuracy and clarity
- Update features list
- Ensure examples work
- Remove outdated information

#### 7. Remove Hype from Strings ⏱️ ~2 hours
**Status**: ✅ Completed  
**Why**: Make the project more professional and concise  
**Tasks**:
- Review all user-facing strings
- Remove overstatements and excessive adjectives
- Make error messages more concise
- Focus on clarity over enthusiasm

### 🟢 Low Priority (Nice to Have)

#### 8. Range Support
- `/regs x1-x5` - Show register range
- `/memory 0x1000-0x2000` - Memory range syntax

#### 9. Consolidate Test Organization ⏱️ ~2 hours
**Status**: ✅ Completed  
**Why**: Better test organization and maintainability  
**Changes**:
- Moved CSR tests from `src/rv32_i/cpu.rs` to `tests/unit/components/csr.rs`
- Moved pseudo-instruction tests from `src/rv32_i/pseudo_instructions.rs` to `tests/unit/instructions/pseudo.rs`
- Created `tests/unit_tests.rs` as entry point for unit tests
- All tests now follow consistent organization pattern
- Source files no longer contain test modules

#### 10. Command History with Arrow Keys ⏱️ ~8 hours
**Status**: ✅ Completed  
**Why**: Essential REPL feature for productivity  
**Implementation**:
- Up/down arrows navigate command history
- Escape cancels browsing
- Automatic deduplication
- Configurable via --history-size and --no-history flags
- Event-based terminal input with crossterm
- Feature-gated in binary only
- Future enhancement: persist to `.brubeck_history` in working directory

#### 11. Tab Completion ⏱️ ~6 hours
**Status**: Not started  
**Why**: Speeds up instruction entry and reduces typos  
**Specification**:
- Tab completes instructions, registers, commands
- Context-aware (e.g., after "ADD" suggest registers)
- Multiple matches show options
- Feature-gated in binary only

## 📊 Progress Summary

### ✅ Completed
- [x] PC address prompt `[0x00000000]>`
- [x] Human-readable instruction output
- [x] Colorized output (green/red dots)
- [x] Command system with `/` prefix
- [x] `/regs` command with specific register support
- [x] `/help` command
- [x] Removed direct register inspection
- [x] History navigation (`/previous`, `/next`)
- [x] Professional CLI with clap
- [x] Modular interpreter architecture
- [x] `/reset` command with confirmation
- [x] `/memory` command for memory inspection
- [x] Command history with arrow keys
- [x] Library/Binary separation (no feature flags in library)
- [x] Clean library API (previous_state, next_state, cpu access)
- [x] All REPL features moved to binary

### 🚧 In Progress
- [ ] Enhanced Learning Features (error messages, instruction history)

### 📚 Documentation Status
- `REPL_USABILITY_FEEDBACK.md` - Original user feedback and analysis
- `REFACTORING_SUMMARY.md` - Details of interpreter modularization
- `INSTRUCTION_IMPLEMENTATION.md` - Guide for adding new instructions
- `CLAUDE.md` - AI assistant context (keep updated!)

## 🎯 Success Metrics

1. **Beginner-friendly**: New users can explore RISC-V without confusion
2. **Educational**: Errors and output teach concepts
3. **Consistent**: All commands follow same patterns
4. **Responsive**: Quick feedback for all operations

## 🚀 Future Vision (Phase 3+)

- **Debugging**: Breakpoints, watchpoints, step execution
- **Visualization**: Register/memory changes, instruction encoding
- **Extensions**: M (multiply), F (float), V (vector) instructions
- **Web Version**: WASM-based browser playground

## 📝 Notes

- Keep library pure (no dependencies) for embedded/WASM use
- Binary can have rich features via feature flags
- All changes should consider educational value
- Maintain comprehensive test coverage

---

**Next Action**: Enhanced Error Messages - Add educational content to error messages