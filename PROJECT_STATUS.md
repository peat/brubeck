# Brubeck Project Status & Roadmap

Last Updated: 2025-07-10

## 🎯 Current Focus: REPL Usability Phase 2

Making Brubeck more beginner-friendly and educational based on user feedback.

## 📋 Task List

### 🔴 High Priority (User Experience)

#### 1. Rename Undo/Redo Commands ⏱️ ~2 hours
**Status**: ✅ Completed  
**Why**: Current names imply "fixing mistakes" rather than "navigating history"  
**Changes**:
- `/undo` → `/previous` (aliases: `/prev`, `/p`)
- `/redo` → `/next` (aliases: `/n`)
- Update parser, help text, error messages, and tests
- Update documentation

#### 2. Memory Inspection Command ⏱️ ~4 hours
**Status**: Not started  
**Why**: Critical for debugging loads/stores and understanding memory  
**Specification**:
- `/memory` - Show 64 bytes around PC
- `/memory <addr>` - Show 64 bytes at address
- `/memory <start> <end>` - Show range (max 256 bytes)
- Alias: `/m`
- Format: Address | Hex bytes | ASCII representation
```
0x00001000: 48 65 6c 6c 6f 20 57 6f | 72 6c 64 21 00 00 00 00  Hello World!....
0x00001010: 00 00 00 00 00 00 00 00 | 00 00 00 00 00 00 00 00  ................
```

#### 3. Reset Command ⏱️ ~2 hours
**Status**: Not started  
**Why**: Users need to start fresh without restarting REPL  
**Specification**:
- `/reset` - Clear all state with confirmation
- Prompt: "Reset CPU? This will clear all registers, memory, and history. (y/N): "
- Reset: registers → 0, PC → 0, clear memory, clear history
- No aliases (destructive command should be explicit)

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
**Status**: Not started  
**Why**: Ensure README accurately reflects current features and capabilities  
**Tasks**:
- Review for accuracy and clarity
- Update features list
- Ensure examples work
- Remove outdated information

#### 7. Remove Hype from Strings ⏱️ ~2 hours
**Status**: Not started  
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

#### 9. Tab Completion
- Instructions, registers, commands
- Context-aware suggestions

#### 10. Command History
- Up/down arrows for previous commands
- Persistent across sessions

## 📊 Progress Summary

### ✅ Completed (Phase 1)
- [x] PC address prompt `[0x00000000]>`
- [x] Human-readable instruction output
- [x] Colorized output (green/red dots)
- [x] Command system with `/` prefix
- [x] `/regs` command with specific register support
- [x] `/help` command
- [x] Removed direct register inspection
- [x] Undo/redo functionality
- [x] Professional CLI with clap
- [x] Modular interpreter architecture

### 🚧 In Progress
- [ ] Phase 2 REPL improvements (see tasks above)

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

**Next Action**: Start with #1 (Rename commands) as a warm-up, then tackle #2 (Memory inspection)