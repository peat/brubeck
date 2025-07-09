# Brubeck Test Coverage Status

This document tracks our current test coverage against the goals outlined in TESTING_GOALS.md.

## Summary
- **Total Tests Migrated**: 30+ (includes new tests)
- **Test Categories Covered**: 3/5 (partial)
- **Test Categories Missing**: 2/5
- **Known Issues**: Sign extension not implemented in LB/LH instructions

## Current Test Inventory

### ✅ Migrated Tests (30+ total)

#### Unit Tests - Instructions (migrated to `tests/unit/instructions/`)
- [x] `nop` - NOP instruction
- [x] `add_sub` - ADD and SUB instructions
- [x] `addi` - ADDI instruction
- [x] `slti` - SLTI instruction
- [x] `sltiu` - SLTIU instruction
- [x] `andi_ori_xori` - Logical operations
- [x] `lui` - LUI instruction
- [x] `auipc` - AUIPC instruction
- [x] `jal` - JAL instruction
- [x] `jalr` - JALR instruction
- [x] `beq` - BEQ instruction
- [x] `bne` - BNE instruction
- [x] `blt` - BLT instruction
- [x] `bltu` - BLTU instruction
- [x] `bge` - BGE instruction
- [x] `bgeu` - BGEU instruction
- [x] `lw_lh_lb` - Load instructions
- [x] `sw_sh_sb` - Store instructions

#### Unit Tests - Components (migrated to `tests/unit/components/`)
- [x] `always_sign_extend` - Sign extension behavior
- [x] `min_max` - Immediate value bounds
- [x] `set_signed` - Setting signed values
- [x] `get_signed` - Getting signed values
- [x] `get_unsigned` - Getting unsigned values

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

### Recently Added Tests
SLL, SLLI, SRL, SRLI, SRA, SRAI (comprehensive shift instruction tests added)

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