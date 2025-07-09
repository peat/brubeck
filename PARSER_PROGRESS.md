# Parser Improvement Progress

## Session Summary
We are working on turning the Brubeck RISC-V parser into a production-grade system with robust validation and helpful error messages.

## Completed Work

### 1. Comprehensive Test Suite
Created `tests/parser_comprehensive.rs` with 36 tests documenting:
- Current working behavior
- Known bugs and limitations
- Desired future features

### 2. Fixed Failing Tests
All tests in the comprehensive suite now pass. We fixed:
- `test_instruction_variants_coverage`: Added proper register setup for JALR alignment
- `test_immediate_representation_bugs`: Updated expectations to match RISC-V sign-extension
- `test_pseudo_li_negative`: Corrected test - LI actually handles negative values properly

### 3. Parser Analysis Findings

#### ~~Core Issues Identified~~ (Fixed in Session 2):
1. ~~**Negative Immediate Bug**: Parser converts signed values to unsigned in `parse_value()`, causing -1 to become 4294967295~~ ✅
2. ~~**Immediate Handling**: All instructions use `set_unsigned()` even when they should use `set_signed()`~~ ✅

#### Still Outstanding:
3. **Missing Syntax**: No support for standard RISC-V offset(register) notation for loads/stores
4. **Limited Error Messages**: Errors lack context and helpful suggestions
5. **Validation Gaps**: Shift immediates accept values > 31 (should be 5-bit max)

#### Current Architecture:
- Parser in `src/interpreter.rs`
- Flow: `parse()` → `normalize()` → `tokenize()` → `build_command()`
- All instruction types now correctly use `set_signed()` for immediates

## Session 2 Completed Work

### Fixed the Negative Immediate Bug! 🎉
The parser now correctly handles negative immediates in all instructions. Changes made:

1. **Changed Token enum**: `Value32(u32)` → `Value32(i32)` to preserve signed values
2. **Fixed parse_value()**: Removed incorrect `as u32` cast that was converting -1 to 4294967295
3. **Updated all instruction builders**: Changed from `set_unsigned()` to `set_signed()` for all immediates
4. **Updated comprehensive tests**: Fixed test expectations to match correct RISC-V signed immediate behavior

All 36 parser comprehensive tests now pass!

### Key Insight from RISC-V Spec
**ALL immediates in RV32I are sign-extended** (except 5-bit CSR immediates). This includes:
- I-type: 12-bit signed (-2048 to 2047)
- S-type: 12-bit signed (-2048 to 2047)
- B-type: 12-bit signed (-4096 to 4094, encoded as multiples of 2)
- U-type: 20-bit signed (-524288 to 524287)
- J-type: 20-bit signed (-1048576 to 1048574, encoded as multiples of 2)

Even ANDI/ORI/XORI use sign-extended immediates, and SLTIU sign-extends then treats as unsigned for comparison.

## Session 3 Completed Work

### Fixed Shift Immediate Validation ✅
Shift instructions (SLLI, SRLI, SRAI) now properly validate immediates at parse time:

1. **Created `build_shift_itype()`**: Specialized builder that validates shift amounts are 0-31
2. **Improved error messages**: Shows the invalid value and valid range
3. **Updated tests**: Now correctly expect errors for out-of-range values

Example error: `Shift amount 100 out of range. Valid range is 0-31`

### Implemented Standard Load/Store Syntax ✅
The parser now supports standard RISC-V offset(register) notation:

1. **Added new token type**: `OffsetRegister { offset: i32, register: Register }`
2. **Enhanced tokenizer**: Parses patterns like "100(x2)", "-4(sp)", "0(x0)"
3. **Created specialized builders**: 
   - `build_load_itype()` for LB, LH, LW, LBU, LHU
   - `build_store_stype()` for SB, SH, SW
4. **Maintained backward compatibility**: Both syntaxes work:
   - Standard: `LW x1, 100(x2)` ✅
   - Legacy: `LW x1, x2, 100` ✅

All load/store instructions now accept standard RISC-V assembly syntax!

## Session 4 Completed Work

### Improved Error Messages ✅
The parser now provides rich, context-aware error messages inspired by the Rust compiler:

1. **Created structured error types**:
   - `UnknownInstruction`: Includes suggestions for similar instructions
   - `InvalidRegister`: Provides helpful guidance on valid register names
   - `WrongArgumentCount`: Shows expected vs. found argument counts
   - `ImmediateOutOfRange`: Displays the valid range for the instruction

2. **Added instruction suggestions**: Uses edit distance to suggest similar valid instructions
   - Example: `Unknown instruction 'ADDD'. Did you mean 'ADD'?`

3. **Enhanced immediate validation messages**:
   - Example: `Immediate value 5000 out of range for ADDI (valid range: -2048 to 2047)`

4. **Better shift instruction errors**:
   - Example: `Immediate value 100 out of range for SLLI (valid range: 0-31)`

### Added Immediate Range Validation ✅
All instruction types now validate immediate ranges at parse time:

1. **I-Type instructions**: Validate 12-bit signed range (-2048 to 2047)
2. **U-Type instructions**: Validate 20-bit signed range (-524288 to 524287)
3. **J-Type instructions**: Validate 20-bit signed range and even alignment
4. **B-Type instructions**: Validate 12-bit signed range and even alignment
5. **Shift instructions**: Already validate 0-31 range
6. **System instructions**: Validate they take no arguments

## Session 5 Completed Work

### Added PC Register Validation ✅
The parser now correctly prevents misuse of the PC register:

1. **Created `validate_not_pc()` helper**: Checks if a register is PC and returns a descriptive error
2. **Updated all instruction builders**: Added PC validation to prevent its use where inappropriate:
   - R-Type: PC cannot be used as any operand (rd, rs1, rs2)
   - I-Type: PC cannot be used as source, and only JALR allows it as destination
   - U-Type: PC cannot be used as destination (even though AUIPC reads PC implicitly)
   - J-Type: PC cannot be used as link register
   - B-Type: PC cannot be used in branch comparisons
   - Shift instructions: PC cannot be used at all
   - Load/Store: PC cannot be used as destination or base address

3. **Helpful error messages**: When PC is misused, users get a clear message:
   - Example: `"PC register cannot be used as source 1 in this instruction. PC is only accessible via AUIPC or as an implicit operand in jumps."`

4. **Comprehensive test coverage**: Added tests for all instruction types to ensure PC is properly rejected

## Remaining Parser Issues

### Critical Correctness Issues
1. ~~**Shift Immediate Validation**: SLLI/SRLI/SRAI accept any 12-bit value but should only accept 0-31~~ ✅
2. ~~**PC Register Misuse**: PC can be incorrectly used in regular instructions (e.g., `ADD x1, PC, x0`)~~ ✅

### Missing RISC-V Standard Features
1. ~~**Load/Store Offset Syntax**: Need `LW x1, offset(base)` instead of current `LW x1, base, offset`~~ ✅
2. ~~**No-Argument Pseudo-instructions**: RET and JR don't parse without arguments~~ ✅ (Fixed in Session 3)

### Quality of Life Improvements
1. ~~**Poor Error Messages**: Current errors like "Invalid IType arguments" lack helpful context~~ ✅
2. ~~**No Immediate Range Info**: Errors don't tell users the valid range for immediates~~ ✅
3. ~~**No Instruction Suggestions**: Unknown instructions could suggest similar valid ones~~ ✅

## Next Steps Priority Order

### ~~1. Fix Shift Immediate Validation~~ ✅ COMPLETED
### ~~2. Implement Standard Load/Store Syntax~~ ✅ COMPLETED
### ~~3. Improve Error Messages~~ ✅ COMPLETED
### ~~4. Add Immediate Range Validation~~ ✅ COMPLETED
### ~~5. Add PC Register Validation~~ ✅ COMPLETED

### 6. Parser Architecture Redesign (Low Priority)
- Current monolithic design makes adding features difficult
- Consider proper lexer/parser separation
- Plan for future extensibility (labels, expressions, etc.)

## Key Code Locations
- Parser: `src/interpreter.rs`
- Tests: `tests/parser_comprehensive.rs`
- Instructions: `src/rv32_i/instructions.rs`
- Pseudo-instructions: `src/rv32_i/pseudo_instructions.rs`

## Architecture Considerations for Next Session
When designing the new parser architecture, consider:
- Separation of lexing, parsing, and semantic analysis phases
- Proper error recovery and reporting
- Extensibility for future RISC-V extensions
- Clear separation between assembly syntax and instruction encoding
- Support for both immediate execution and future compilation modes