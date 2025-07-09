# Brubeck REPL Usability Feedback

## Session Summary
Based on hands-on testing with a new RISC-V learner, we identified several key usability improvements needed to make Brubeck more beginner-friendly and educational.

## Critical Issues Identified

### 1. Missing Visual Prompt
- **Problem**: No prompt to indicate where to type
- **Current**: Blank line after "Ctrl-C to quit"
- **Solution**: Add PC address prompt: `[0x00000000]> `
- **Benefits**: Shows program execution flow, teaches PC concept, useful for jumps/branches

### 2. Cryptic Debug Output
- **Problem**: Shows internal instruction structure instead of human explanation
- **Current**: `✅ ADDI(IType { opcode: 0, rd: X1, funct3: 0, rs1: X0, imm: Immediate { value: 42, bits: 12 } })`
- **Solution**: `✅ Added 42 to x0 (0) and stored the result in x1 (42)`
- **Benefits**: Explains what happened in plain English, educational value

### 3. No Register State Overview
- **Problem**: Can't see full CPU state at once
- **Current**: Must type `x1`, `x2`, `x3`... individually
- **Solution**: `/regs` command with flexible syntax
- **Benefits**: Quick state inspection, learning aid

## Proposed Command System

### Design Principles
- **"/" prefix** for Brubeck commands vs RISC-V instructions
- **Flexible syntax** - handle spaces and commas naturally
- **Progressive disclosure** - simple to complex usage
- **Safety confirmations** for destructive actions
- **Shorthand aliases** for frequent operations

### Command Specifications

#### `/regs` - Register Inspection
```
/regs                    - Show all registers (with * for modified)
/regs x4                 - Show just x4
/regs x4 x3 x1          - Show multiple registers
/regs x4, x3, x1        - Comma-tolerant syntax
/regs ra sp gp          - Works with ABI names
```

**Shorthand**: `/r`

#### `/memory` - Memory Inspection
```
/memory                  - Show memory around PC
/memory 0x1000          - Show memory at specific address
/memory 0x1000 0x2000   - Show range
```

**Shorthand**: `/m`

#### `/help` - Documentation
```
/help                   - Show all help
/help registers         - Show help about registers
/help jumps branches    - Show help about specific topics
```

**Shorthand**: `/h`

#### `/reset` - Reset CPU State
```
/reset
Are you sure you want to reset the CPU? This will clear all registers and memory. (y/N): 
```

**Safety**: Requires confirmation

### Command Frequency Predictions

**Super frequent** (single-letter shortcuts):
- `/r` - Show registers (constant state checking)
- `/m` - Show memory (debugging loads/stores)
- `/h` - Help (learning new instructions)

**Frequent** (short abbreviations):
- `/reset` - Start over when things get messy
- `/pc` - Check program counter

**Occasional** (full names fine):
- `/clear` - Clean up screen
- `/exit` or `/quit` - Leave REPL

## UI Improvements

### Welcome Message Enhancement
```
Brubeck: A RISC-V REPL (RV32I + CSR)
Ctrl-C to quit

[0x00000000]> 
```

### Register Display Format
```
brubeck> /regs
Program Counter: 0x0000000C

Registers:
x0  (zero): 0x00000000    x1  (ra)  : 0x0000002A *
x2  (sp)  : 0x00000064 *  x3  (gp)  : 0x0000008E *
x4  (tp)  : 0x00000000    x5  (t0)  : 0x00000000
...
(* = modified)
```

## Implementation Priority

### Phase 1 (Critical for Learning)
1. Add PC address prompt
2. Human-readable instruction output
3. Basic `/regs` command
4. `/help` command

### Phase 2 (Enhanced UX)
1. `/memory` command
2. Command aliases (`/r`, `/m`, `/h`)
3. `/reset` with confirmation
4. Flexible command syntax (comma tolerance)

### Phase 3 (Advanced Features)
1. Range support (`/regs x1-x5`)
2. Context-aware help
3. Command history
4. Tab completion

## User Experience Goals

- **Immediate clarity** - Always clear what happened and why
- **Exploration-friendly** - Easy to try things and see results
- **Mistake-tolerant** - Easy to recover from errors
- **Progressive learning** - Simple commands that grow in complexity
- **Familiar patterns** - Command conventions that feel natural

## Next Steps

1. Implement Phase 1 improvements
2. Test with more new users
3. Gather feedback on command usage patterns
4. Iterate on command design based on actual usage
5. Add more educational features based on learning needs

## Key Insight

The current debug output serves compiler developers, but learners need human-readable explanations that teach RISC-V concepts. The command system should prioritize learning and exploration over technical precision.