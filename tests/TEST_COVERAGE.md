# Brubeck Test Coverage Status

This document tracks our current test coverage against the goals outlined in TESTING_GOALS.md.

## Summary
- **Total Existing Tests**: 26 (all inline with source code)
- **Test Categories Covered**: 2/5 (partial)
- **Test Categories Missing**: 3/5

## Current Test Inventory

### ✅ Existing Tests (26 total)

#### Unit Tests - Instructions (17 tests in `src/rv32_i/mod.rs`)
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

#### Unit Tests - Components (5 tests in `src/immediate.rs`)
- [x] `always_sign_extend` - Sign extension behavior
- [x] `min_max` - Immediate value bounds
- [x] `set_signed` - Setting signed values
- [x] `get_signed` - Getting signed values
- [x] `get_unsigned` - Getting unsigned values

#### Integration Tests - Parser (4 tests in `src/interpreter.rs`)
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
SLL, SLLI, SRL, SRLI, SRA, SRAI, LHU, LBU, FENCE, EBREAK, ECALL

### Not Implemented
CSRRW, CSRRS, CSRRC, CSRRWI, CSRRSI, CSRRCI

## Priority Actions

### High Priority
1. **Create format encoding/decoding tests** - Foundation for correctness
2. **Add error handling tests** - Critical for educational use
3. **Expand instruction tests** - Add edge cases and spec compliance

### Medium Priority
1. **Create multi-instruction sequence tests** - Real program behavior
2. **Add REPL integration tests** - User experience validation
3. **Implement spec example tests** - Authoritative correctness

### Low Priority
1. **Set up cross-validation framework** - External verification
2. **Add educational quality tests** - Polish user experience
3. **Create property-based tests** - Advanced testing

## Test Migration Plan

1. **Phase 1**: Create test helpers and utilities
2. **Phase 2**: Migrate existing inline tests to new structure
3. **Phase 3**: Fill gaps in high-priority categories
4. **Phase 4**: Implement medium-priority tests
5. **Phase 5**: Add advanced testing capabilities