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

### Fully Implemented (40 instructions)
- Arithmetic: ADD, ADDI, SUB
- Logical: AND, ANDI, OR, ORI, XOR, XORI
- Shifts: SLL, SLLI, SRL, SRLI, SRA, SRAI
- Comparison: SLT, SLTI, SLTU, SLTIU
- Loads: LW, LH, LHU, LB, LBU
- Stores: SW, SH, SB
- Upper immediate: LUI, AUIPC
- Jumps: JAL, JALR
- Branches: BEQ, BNE, BLT, BLTU, BGE, BGEU
- Other: NOP

### Recognized but Not Implemented (3 instructions)
- EBREAK - Environment break
- ECALL - Environment call
- FENCE - Memory ordering

### Not Present (6 CSR instructions)
- CSRRW - Atomic Read/Write CSR
- CSRRS - Atomic Read and Set Bits
- CSRRC - Atomic Read and Clear Bits
- CSRRWI - Immediate variant of CSRRW
- CSRRSI - Immediate variant of CSRRS
- CSRRCI - Immediate variant of CSRRC

## References
- RV32I Base Integer Instruction Set: `riscv-isa-manual/src/rv32.adoc`
- CSR Instructions: `riscv-isa-manual/src/zicsr.adoc`
- Instruction Encoding Formats: `riscv-isa-manual/src/rv32.adoc` (Section 2.2)