# Brubeck Phase 2 TODO List

Based on REPL_USABILITY_FEEDBACK.md, here's what remains to be implemented:

## High Priority Features

### 0. Rename Undo/Redo Commands
- **Purpose**: Better naming convention for instruction history navigation
- **Changes**:
  - Rename `/undo` to `/previous` (alias: `/prev` or `/p`)
  - Rename `/redo` to `/next` (alias: `/n`)
  - Update help text to reflect new names
  - Update all error messages
  - Update tests
- **Rationale**: "Previous" and "next" better describe navigating through instruction history rather than "undo/redo" which implies mistake correction

### 1. `/memory` Command
- **Purpose**: Inspect memory contents for debugging loads/stores
- **Syntax**:
  - `/memory` - Show memory around PC
  - `/memory 0x1000` - Show memory at specific address  
  - `/memory 0x1000 0x2000` - Show range
- **Alias**: `/m`
- **Display format**: Show addresses, hex values, and ASCII representation

### 2. `/reset` Command  
- **Purpose**: Reset CPU state when learning/debugging
- **Safety**: Require confirmation to prevent accidental data loss
- **Prompt**: "Are you sure you want to reset the CPU? This will clear all registers and memory. (y/N):"
- **What to reset**:
  - All registers to 0
  - PC to 0
  - Clear all memory
  - Clear undo/redo history

## Medium Priority Features

### 3. Enhanced Error Messages
- Add more educational context to error messages
- Suggest corrections for common mistakes
- Include RISC-V learning tips

### 4. Instruction History Display
- Show last N instructions executed
- Useful for understanding program flow
- Could be part of a `/history` command

## Low Priority Features (Phase 3)

### 5. Range Support
- `/regs x1-x5` - Show registers x1 through x5
- `/memory 0x1000-0x2000` - Show memory range

### 6. Tab Completion
- Complete instruction names
- Complete register names
- Complete command names

### 7. Command History (↑/↓ arrows)
- Navigate through previous commands
- Already partially supported by crossterm

## Implementation Notes

- Keep commands consistent with existing patterns
- All commands should have educational value
- Error messages should teach, not just report
- Consider beginners learning RISC-V

## Next Steps

1. Implement `/memory` command with basic functionality
2. Implement `/reset` command with confirmation
3. Add `/m` alias for memory command
4. Test with users and gather feedback
5. Iterate based on usage patterns