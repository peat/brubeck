# Brubeck Test Coverage Status

This document tracks our current test coverage against the goals outlined in TESTING_GOALS.md.

## Summary
- **Total Tests**: 78 unit tests + 7 component tests + 4 integration tests
- **Test Helper Framework**: ✅ Fully integrated
- **Educational Documentation**: ✅ Added to all tests
- **Test Categories Covered**: 3/5 (comprehensive)
- **Test Categories Missing**: 2/5
- **Known Issues**: Parser handling of negative immediates

## Test Helper Framework Integration

All tests now use a comprehensive test helper framework (`tests/unit/test_helpers.rs`) that provides:
- **CpuBuilder**: Fluent API for CPU state setup
- **CpuAssertions**: Descriptive assertion methods with context
- **Named Constants**: Replace magic numbers throughout tests
- **Educational Documentation**: ISA references, visual diagrams, and RISC-V patterns

## Current Test Inventory

### ✅ Unit Tests - Instructions (71 tests total)

#### Instruction Categories (tests/unit/instructions/)
| Category | File | Tests | Key Features |
|----------|------|-------|--------------|
| Arithmetic | arithmetic.rs | 7 | ADD/SUB overflow, ADDI patterns |
| Loads/Stores | loads_stores.rs | 14 | Memory visualization, endianness |
| Shifts | shifts.rs | 14 | 5-bit masking behavior |
| Branches | branches.rs | 11 | PC-relative addressing |
| Jumps | jumps.rs | 9 | Call/return conventions |
| Upper Immediate | upper_immediate.rs | 6 | LUI/AUIPC combinations |
| Logical | logical.rs | 5 | Bit manipulation patterns |
| Comparison | comparison.rs | 5 | Signed vs unsigned |
| Miscellaneous | misc.rs | 2 | NOP use cases |

### ✅ Unit Tests - Components (7 tests total)

#### Component Tests (tests/unit/components/)
- **immediate.rs** (7 tests): Comprehensive immediate value handling
  - Sign extension behavior (critical RISC-V concept)
  - Bounds checking for different bit widths
  - Signed vs unsigned value handling
  - Educational examples of common gotchas

#### Integration Tests - Parser (migrated to `tests/integration/parser.rs`)
- [x] `normalize_input` - Input normalization
- [x] `tokenize_input` - Tokenization
- [x] `parse_command` - Command parsing
- [x] `trivial_add` - Simple execution test

### ❌ Missing Test Categories

#### Unit Tests - Formats
- [ ] R-Type encoding/decoding
- [ ] I-Type encoding/decoding
- [ ] S-Type encoding/decoding
- [ ] B-Type encoding/decoding
- [ ] U-Type encoding/decoding
- [ ] J-Type encoding/decoding

#### Integration Tests - Sequences
- [ ] Function calls (JAL/JALR patterns)
- [ ] Loops (branch patterns)
- [ ] Stack operations
- [ ] Common code patterns

#### Integration Tests - REPL
- [ ] Multi-instruction sessions
- [ ] State inspection commands
- [ ] Error recovery

#### Compliance Tests
- [ ] Examples from RISC-V specification
- [ ] Systematic instruction coverage
- [ ] Cross-validation with reference simulators

#### Error Handling Tests
- [ ] Invalid assembly syntax
- [ ] Out-of-range immediates
- [ ] Invalid register names
- [ ] Memory access violations
- [ ] Misaligned jumps/branches

#### Educational Tests
- [ ] Error message clarity
- [ ] Diagnostic output accuracy
- [ ] Documentation example validation

## Coverage by Instruction

### Fully Tested (basic tests only)
ADD, ADDI, SUB, AND, ANDI, OR, ORI, XOR, XORI, SLT, SLTI, SLTU, SLTIU,
LUI, AUIPC, JAL, JALR, BEQ, BNE, BLT, BLTU, BGE, BGEU, LW, LH, LB, SW, SH, SB, NOP

### Partially Tested
None - all tested instructions have only basic tests

### Not Tested
LHU, LBU, FENCE, EBREAK, ECALL

### Test Improvements from Migration
- **Educational Documentation**: Every test file now includes ISA manual references
- **Visual Diagrams**: Memory layouts, bit patterns, and encoding explanations
- **Common Patterns**: RISC-V idioms like NOT (XORI -1), RET (JALR x0, 0(ra))
- **Consistent Structure**: All tests use the same helper patterns
- **Better Error Messages**: CpuAssertions provide context for failures

### Not Implemented
CSRRW, CSRRS, CSRRC, CSRRWI, CSRRSI, CSRRCI

## Priority Actions

### High Priority
1. **Fix sign extension in LB/LH instructions** - Currently failing tests
2. **Create format encoding/decoding tests** - Foundation for correctness
3. **Add error handling tests** - Critical for educational use
4. **Expand instruction tests** - Add edge cases and spec compliance

### Medium Priority
1. **Create multi-instruction sequence tests** - Real program behavior
2. **Add REPL integration tests** - User experience validation
3. **Implement spec example tests** - Authoritative correctness

### Low Priority
1. **Set up cross-validation framework** - External verification
2. **Add educational quality tests** - Polish user experience
3. **Create property-based tests** - Advanced testing

## Test Migration Status

1. **Phase 1**: ✅ Test structure created
2. **Phase 2**: ✅ All existing tests migrated
3. **Phase 3**: 🔄 In progress - added shift tests, found LB/LH issues
4. **Phase 4**: ⏳ Pending
5. **Phase 5**: ⏳ Pending

## Implementation Issues Found

### Sign Extension in Load Instructions ✅ FIXED
- **Issue**: LB and LH instructions were not performing sign extension
- **Impact**: 4 tests were failing in loads_stores.rs
- **Expected**: LB/LH should sign-extend, LBU/LHU should zero-extend
- **Fix**: Updated LB to use `u8 as i8 as i32 as u32` and LH to use `u16 as i16 as i32 as u32`
- **Status**: Fixed - all tests now pass

### Parser Handling of Negative Immediates
- **Issue**: Parser uses set_unsigned for all immediates, fails on negative values
- **Impact**: Cannot parse instructions like "ADDI x1, zero, -1"
- **Expected**: Parser should use set_signed for immediates that can be negative
- **Status**: Documented with test workaround