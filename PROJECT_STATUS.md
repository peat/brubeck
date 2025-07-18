# Brubeck Project Status & Roadmap

Last Updated: 2025-07-18

## 🎯 Current Focus: Interpreter Public API Refactoring

Major refactoring to provide a cleaner, data-focused API that separates parsing from execution.

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

#### 0. Structured Error Types in Library ⏱️ ~6 hours
**Status**: Not started  
**Why**: Library consumers shouldn't have to parse error strings  
**Specification**:
- Replace string-based errors with structured types containing all error data
- Each error type should have specific fields (e.g., InvalidRegister { name, suggestion })
- Binary formats these into user-friendly messages with colors and tips
- Other consumers (WASM, embedded) can handle errors appropriately
- Follows same pattern as StateDelta - data in library, presentation in consumers

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
**Status**: ✅ Completed  
**Why**: Turn errors into learning opportunities  
**Implementation**:
- Added "Did you mean?" suggestions with fuzzy string matching
- Enhanced all error types with educational content
- Added `--tips` CLI flag to enable/disable educational content
- Tips are opt-in to avoid overwhelming experienced users
- REPL banner mentions --tips flag for assistance

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
- [x] `/regs` command with specific register support and color highlighting
- [x] `/help` command
- [x] Removed direct register inspection
- [x] History navigation (`/previous`, `/next`)
- [x] Professional CLI with clap
- [x] Modular interpreter architecture
- [x] `/reset` command with confirmation
- [x] `/memory` command for memory inspection with color highlighting
- [x] Command history with arrow keys
- [x] Library/Binary separation (no feature flags in library)
- [x] Clean library API (previous_state, next_state, cpu access)
- [x] All REPL features moved to binary
- [x] Interpreter Public API Refactoring (returns structured data, not strings)
- [x] Binary Formatting Implementation (all formatters with color support)
- [x] Register Output Colorization (zeros gray, changes green)
- [x] Enhanced Error Formatting (contextual tips for all error types)

### 🚧 In Progress

None currently - all major refactoring tasks completed!

### 📚 Documentation Status
- `INSTRUCTION_IMPLEMENTATION.md` - Guide for adding new instructions
- `CLAUDE.md` - AI assistant context (keep updated!)
- `docs/archive/` - Completed specs and progress tracking from past refactoring work

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

**Recent Completions**: 
1. ✅ Binary Formatting Implementation - Complete separation of data and presentation!
   - Created comprehensive formatting module in binary
   - Implemented StateDelta formatter for instruction results
   - Added color support to register display (zeros gray, changes green)
   - Added color support to memory display (PC highlighted, changes green)
   - Enhanced all error formatters with contextual tips
   - Added comprehensive tests for all formatting functions
   - Clean separation: library returns data, binary handles all formatting

2. ✅ Interpreter Public API Refactoring - Library now returns structured data!
   - Library's `interpret()` returns `StateDelta` instead of strings
   - All formatting moved to binary
   - Clean data/presentation separation
   - Comprehensive error types with educational content
   - Full test coverage maintained

3. ✅ Register and Memory Colorization - Visual feedback for state changes!
   - Registers: zeros in gray, changed values in green
   - Memory: PC location highlighted, changed bytes in green
   - Clean implementation using StateDelta tracking
   - No complex wrapper structs needed

4. ✅ Register Display Improvements - Clean, aligned output!
   - Fixed register display to use sequential columns (x0-x15 left, x16-x31 right)
   - Removed extra whitespace in register names (x0, not "x 0")
   - Removed padding from ABI names (x0 (zero), not x0 (zero  ))
   - Consistent columnar alignment for easy reading

5. ✅ History Navigation Enhancements - Detailed change information!
   - `/prev` and `/next` now show exactly what changed
   - Example: "Navigated back: x2: 100 → 0, PC: 0x00000008 → 0x00000004"
   - Much more useful than generic "Changed: 1 register"

---

**Next Action**: Refactor library errors to be highly structured instead of strings
- Library should return structured error types with all relevant data
- Binary (and other consumers) can format errors appropriately
- No string parsing needed by consumers
- Follows same pattern as StateDelta - data in library, presentation in consumers