# RISC-V Instruction Implementation Guide

This guide describes the systematic process for implementing new RISC-V instructions in Brubeck.

## Implementation Checklist

When implementing a new instruction from the RISC-V specification, follow these steps in order:

### 1. Instruction Definition Phase
- [ ] Add the instruction variant to `src/rv32_i/instructions.rs` Instruction enum
- [ ] Use the appropriate format type (RType, IType, SType, BType, UType, JType)
- [ ] For new formats, define them in `src/rv32_i/formats.rs`

### 2. Decoding Phase
- [ ] Add the opcode/funct3/funct7 mapping in `src/rv32_i/instructions.rs` decode() method
- [ ] Match the exact encoding from the specification
- [ ] Handle any special decoding requirements

### 3. Execution Phase
- [ ] Add the instruction case to `src/rv32_i/cpu.rs` execute() method
- [ ] Implement the execution method (e.g., `rv32i_instruction_name()`)
- [ ] Follow the specification's pseudocode exactly
- [ ] Handle all edge cases mentioned in the spec

### 4. Interpreter Phase
- [ ] Add the instruction name to the tokenizer in `src/interpreter.rs`
- [ ] Ensure the instruction can be parsed from assembly syntax
- [ ] Add any special argument parsing if needed

### 5. Testing Phase
- [ ] Add comprehensive tests in `src/rv32_i/mod.rs`
- [ ] Test normal operation
- [ ] Test edge cases from the specification
- [ ] Test error conditions

### 6. Documentation Phase
- [ ] Add doc comments to the execution method describing the instruction
- [ ] Update README.md if completing an instruction group
- [ ] Ensure comments reference the relevant spec section

## Example: Implementing CSRRW

```rust
// 1. In instructions.rs - Add to enum
pub enum Instruction {
    // ... existing instructions ...
    CSRRW(IType),
}

// 2. In instructions.rs - Add to decode()
0b1110011 => match funct3 {
    0b001 => Instruction::CSRRW(IType::new(word)?),
    // ...
}

// 3. In cpu.rs - Add execution
impl CPU {
    pub fn execute(&mut self, instruction: Instruction) -> Result<(), Error> {
        match instruction {
            // ... existing cases ...
            Instruction::CSRRW(i) => self.rv32i_csrrw(i),
        }
    }

    /// CSRRW (Atomic Read/Write CSR) atomically swaps values in the CSRs 
    /// and integer registers. See riscv-isa-manual/src/zicsr.adoc
    fn rv32i_csrrw(&mut self, instruction: IType) -> Result<(), Error> {
        // Implementation following spec
    }
}

// 4. In interpreter.rs - Add to tokenizer
"CSRRW" => Token::Instruction(Instruction::CSRRW(IType::default())),

// 5. In mod.rs - Add tests
#[test]
fn csrrw() {
    let mut cpu = CPU::default();
    // Test implementation
}
```

## Current Implementation Status

### Complete RV32I Base Integer Instruction Set (47 instructions)
All RV32I instructions are now fully implemented:
- Arithmetic: ADD, ADDI, SUB
- Logical: AND, ANDI, OR, ORI, XOR, XORI
- Shifts: SLL, SLLI, SRL, SRLI, SRA, SRAI
- Comparison: SLT, SLTI, SLTU, SLTIU
- Loads: LW, LH, LHU, LB, LBU (with proper sign extension)
- Stores: SW, SH, SB
- Upper immediate: LUI, AUIPC
- Jumps: JAL, JALR
- Branches: BEQ, BNE, BLT, BLTU, BGE, BGEU
- System: FENCE (memory ordering), ECALL (system call), EBREAK (breakpoint)
- Other: NOP

### Pseudo-Instructions (8 implemented)
Common RISC-V pseudo-instructions that expand to real instructions:
- MV rd, rs → ADDI rd, rs, 0
- NOT rd, rs → XORI rd, rs, -1
- SEQZ rd, rs → SLTIU rd, rs, 1
- SNEZ rd, rs → SLTU rd, x0, rs
- J offset → JAL x0, offset
- JR rs → JALR x0, rs, 0
- RET → JALR x0, x1, 0
- LI rd, imm → ADDI/LUI+ADDI sequence

### In Progress (6 CSR instructions)
CSR infrastructure complete with comprehensive test suite. Implementation in progress:
- CSRRW - Atomic Read/Write CSR ⚠️ (tests ready, execution methods pending)
- CSRRS - Atomic Read and Set Bits ⚠️ (tests ready, execution methods pending)
- CSRRC - Atomic Read and Clear Bits ⚠️ (tests ready, execution methods pending)
- CSRRWI - Immediate variant of CSRRW ⚠️ (tests ready, execution methods pending)
- CSRRSI - Immediate variant of CSRRS ⚠️ (tests ready, execution methods pending)
- CSRRCI - Immediate variant of CSRRC ⚠️ (tests ready, execution methods pending)

**CSR Infrastructure Complete:**
- 4096 CSR address space with existence and read-only tracking
- Standard CSRs initialized (mstatus, misa, cycle, time, instret, etc.)
- WARL behavior for mstatus register
- Comprehensive test suite with 25+ tests covering spec compliance
- Educational documentation with RISC-V spec references

## References
- RV32I Base Integer Instruction Set: `riscv-isa-manual/src/rv32.adoc`
- CSR Instructions: `riscv-isa-manual/src/zicsr.adoc`
- Instruction Encoding Formats: `riscv-isa-manual/src/rv32.adoc` (Section 2.2)