//! Unit tests for load and store instructions
//!
//! These tests verify memory access patterns, different widths (byte, halfword, word),
//! sign extension behavior, and little-endian byte ordering.
//!
//! Reference: RISC-V ISA Manual, Volume I: Unprivileged ISA, Version 20191213
//! Section 2.6 - Load and Store Instructions
//!
//! Key concepts tested:
//! - Little-endian byte ordering
//! - Sign extension (LB, LH) vs zero extension (LBU, LHU)
//! - Immediate offset addressing
//! - Memory alignment behavior
//!
//! Memory Layout Visualization (Little-Endian):
//! Word 0x12345678 stored at address 1024:
//! ```
//! Address: [1024] [1025] [1026] [1027]
//! Bytes:    0x78   0x56   0x34   0x12
//!           LSB                    MSB
//! ```

use brubeck::rv32_i::{
    formats::{IType, SType},
    instructions::Instruction,
    registers::Register,
};

// Import test helpers
use crate::unit::test_helpers::{values, CpuAssertions, CpuBuilder, ExecuteWithContext};

#[test]
fn test_lw_basic() {
    // LW: rd = sign_extend(memory[rs1 + sign_extend(immediate)])
    // Loads 32-bit word from memory
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, values::TEST_ADDR)
        .with_memory_word_le(values::TEST_ADDR, 0x12345678)
        .build();

    let mut inst = IType::default();
    inst.rs1 = Register::X1;
    inst.rd = Register::X2;
    inst.imm.set_unsigned(0).unwrap(); // Zero offset

    let lw = Instruction::LW(inst);
    cpu.execute_expect(lw, "LW from base address");

    cpu.assert_register(
        Register::X2,
        0x12345678,
        "LW loads full 32-bit word in little-endian order",
    );
}

#[test]
fn test_lw_with_offset() {
    // Test positive and negative offsets for load instructions
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, values::TEST_ADDR)
        .with_memory_pattern(
            values::TEST_ADDR,
            &[
                0x11, 0x22, 0x33, 0x44, // Word at offset 0
                0x55, 0x66, 0x77, 0x88, // Word at offset 4
            ],
        )
        .build();

    let mut inst = IType::default();
    inst.rs1 = Register::X1;
    inst.rd = Register::X2;

    // Test positive offset
    inst.imm.set_signed(4).unwrap();
    let lw = Instruction::LW(inst);
    cpu.execute_expect(lw, "LW with positive offset");
    cpu.assert_register(
        Register::X2,
        0x88776655,
        "LW from base + 4 loads second word",
    );

    // Test offset 2 (misaligned but allowed in this implementation)
    inst.imm.set_signed(2).unwrap();
    let lw = Instruction::LW(inst);
    cpu.execute_expect(lw, "LW with misaligned offset");
    cpu.assert_register(
        Register::X2,
        0x66554433,
        "LW from base + 2 (misaligned access)",
    );
}

#[test]
fn test_lh_sign_extension() {
    // LH: Load halfword with sign extension
    // Bit 15 determines sign: 0 = positive, 1 = negative
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, values::TEST_ADDR)
        .build();

    let mut inst = IType::default();
    inst.rs1 = Register::X1;
    inst.rd = Register::X2;
    inst.imm.set_unsigned(0).unwrap();
    let lh = Instruction::LH(inst);

    // Test cases: (bytes, expected_result, description)
    let test_cases = [
        ([0x34, 0x12], 0x00001234, "positive halfword (bit 15 = 0)"),
        ([0x00, 0x80], 0xFFFF8000, "negative halfword (bit 15 = 1)"),
        (
            [0xFF, 0xFF],
            0xFFFFFFFF,
            "all ones sign-extends to all ones",
        ),
        ([0xFF, 0x7F], 0x00007FFF, "max positive halfword"),
    ];

    for (bytes, expected, desc) in test_cases {
        cpu.memory[values::TEST_ADDR as usize] = bytes[0];
        cpu.memory[values::TEST_ADDR as usize + 1] = bytes[1];

        cpu.execute_expect(lh, desc);
        cpu.assert_register(Register::X2, expected, desc);
    }
}

#[test]
fn test_lhu_zero_extension() {
    // LHU: Load halfword unsigned (zero-extended)
    // Always fills upper 16 bits with zeros
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, values::TEST_ADDR)
        .build();

    let mut inst = IType::default();
    inst.rs1 = Register::X1;
    inst.rd = Register::X2;
    inst.imm.set_unsigned(0).unwrap();
    let lhu = Instruction::LHU(inst);

    // Test value that would be negative if sign-extended
    cpu.memory[values::TEST_ADDR as usize] = 0xFF;
    cpu.memory[values::TEST_ADDR as usize + 1] = 0xFF; // 0xFFFF

    cpu.execute_expect(lhu, "LHU with all bits set");
    cpu.assert_register(
        Register::X2,
        0x0000FFFF,
        "LHU zero-extends to 32 bits (no sign extension)",
    );
}

#[test]
fn test_lb_sign_extension() {
    // LB: Load byte with sign extension
    // Bit 7 determines sign: 0 = positive, 1 = negative
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, values::TEST_ADDR)
        .build();

    let mut inst = IType::default();
    inst.rs1 = Register::X1;
    inst.rd = Register::X2;
    inst.imm.set_unsigned(0).unwrap();
    let lb = Instruction::LB(inst);

    // Test cases: (byte_value, expected_result, description)
    let test_cases = [
        (0x00, 0x00000000, "zero byte"),
        (0x7F, 0x0000007F, "max positive byte (+127)"),
        (0x80, 0xFFFFFF80, "min negative byte (-128)"),
        (0xFF, 0xFFFFFFFF, "negative one (-1)"),
        (0x01, 0x00000001, "positive one"),
    ];

    for (byte_val, expected, desc) in test_cases {
        cpu.memory[values::TEST_ADDR as usize] = byte_val;
        cpu.execute_expect(lb, desc);
        cpu.assert_register(Register::X2, expected, desc);
    }
}

#[test]
fn test_lbu_zero_extension() {
    // LBU: Load byte unsigned (zero-extended)
    // Always fills upper 24 bits with zeros
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, values::TEST_ADDR)
        .with_memory_byte(values::TEST_ADDR, 0xFF)
        .build();

    let mut inst = IType::default();
    inst.rs1 = Register::X1;
    inst.rd = Register::X2;
    inst.imm.set_unsigned(0).unwrap();

    let lbu = Instruction::LBU(inst);
    cpu.execute_expect(lbu, "LBU with 0xFF");
    cpu.assert_register(
        Register::X2,
        0x000000FF,
        "LBU zero-extends 0xFF to 255 (not -1)",
    );
}

#[test]
fn test_sw_basic() {
    // SW: Store word to memory
    // memory[rs1 + sign_extend(immediate)] = rs2
    // Stores in little-endian byte order
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, 100) // Base address
        .with_register(Register::X2, 0x12345678) // Value to store
        .build();

    let mut inst = SType::default();
    inst.rs1 = Register::X1;
    inst.rs2 = Register::X2;
    inst.imm.set_unsigned(0).unwrap();

    let sw = Instruction::SW(inst);
    cpu.execute_expect(sw, "SW to memory");

    // Verify little-endian storage
    cpu.assert_memory_bytes(
        100,
        &[0x78, 0x56, 0x34, 0x12],
        "SW stores word in little-endian order",
    );
}

#[test]
fn test_sh_truncation() {
    // SH: Store halfword (lower 16 bits only)
    // Upper 16 bits of source register are ignored
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, 200)
        .with_register(Register::X2, 0xFFFF1234)
        .with_memory_pattern(200, &[0, 0, 0xFF, 0xFF]) // Pre-fill to verify
        .build();

    let mut inst = SType::default();
    inst.rs1 = Register::X1;
    inst.rs2 = Register::X2;
    inst.imm.set_unsigned(0).unwrap();

    let sh = Instruction::SH(inst);
    cpu.execute_expect(sh, "SH truncates to 16 bits");

    cpu.assert_memory_bytes(
        200,
        &[0x34, 0x12, 0xFF, 0xFF],
        "SH stores only lower 16 bits, preserves other memory",
    );
}

#[test]
fn test_sb_truncation() {
    // SB: Store byte (lower 8 bits only)
    // Upper 24 bits of source register are ignored
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, 300)
        .with_register(Register::X2, 0xFFFFFF78)
        .with_memory_pattern(300, &[0xFF, 0xFF]) // Pre-fill to verify
        .build();

    let mut inst = SType::default();
    inst.rs1 = Register::X1;
    inst.rs2 = Register::X2;
    inst.imm.set_unsigned(0).unwrap();

    let sb = Instruction::SB(inst);
    cpu.execute_expect(sb, "SB truncates to 8 bits");

    cpu.assert_memory_bytes(
        300,
        &[0x78, 0xFF],
        "SB stores only lowest 8 bits, preserves other memory",
    );
}

#[test]
fn test_sw_lw_roundtrip() {
    // Test that SW followed by LW preserves the full 32-bit value
    let test_value = 0xDEADBEEF;
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, 100) // Base address
        .with_register(Register::X2, test_value) // Value to store
        .build();

    // Store word
    let mut store_inst = SType::default();
    store_inst.rs1 = Register::X1;
    store_inst.rs2 = Register::X2;
    let sw = Instruction::SW(store_inst);
    cpu.execute_expect(sw, "SW test value");

    // Load word back into different register
    let mut load_inst = IType::default();
    load_inst.rs1 = Register::X1;
    load_inst.rd = Register::X3;
    let lw = Instruction::LW(load_inst);
    cpu.execute_expect(lw, "LW test value back");

    cpu.assert_register(
        Register::X3,
        test_value,
        "SW/LW roundtrip preserves 32-bit value",
    );
}

#[test]
fn test_sh_lh_roundtrip() {
    // Test SH truncation and LH sign extension together
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, 100)
        .with_register(Register::X2, 0xFFFF8765) // Negative when viewed as 16-bit
        .build();

    // Store halfword (truncates to 0x8765)
    let mut store_inst = SType::default();
    store_inst.rs1 = Register::X1;
    store_inst.rs2 = Register::X2;
    let sh = Instruction::SH(store_inst);
    cpu.execute_expect(sh, "SH negative halfword");

    // Load halfword back with sign extension
    let mut load_inst = IType::default();
    load_inst.rs1 = Register::X1;
    load_inst.rd = Register::X3;
    let lh = Instruction::LH(load_inst);
    cpu.execute_expect(lh, "LH negative halfword");

    cpu.assert_register(
        Register::X3,
        0xFFFF8765,
        "SH/LH roundtrip: 16-bit value preserved with sign extension",
    );
}

#[test]
fn test_sb_lb_roundtrip() {
    // Test SB truncation and LB sign extension together
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, 100)
        .with_register(Register::X2, 0xFFFFFF81) // Negative when viewed as 8-bit
        .build();

    // Store byte (truncates to 0x81)
    let mut store_inst = SType::default();
    store_inst.rs1 = Register::X1;
    store_inst.rs2 = Register::X2;
    let sb = Instruction::SB(store_inst);
    cpu.execute_expect(sb, "SB negative byte");

    // Load byte back with sign extension
    let mut load_inst = IType::default();
    load_inst.rs1 = Register::X1;
    load_inst.rd = Register::X3;
    let lb = Instruction::LB(load_inst);
    cpu.execute_expect(lb, "LB negative byte");

    cpu.assert_register(
        Register::X3,
        0xFFFFFF81,
        "SB/LB roundtrip: 8-bit value preserved with sign extension",
    );
}

#[test]
fn test_load_store_with_negative_offset() {
    // Test that negative immediates work correctly for addressing
    // Common pattern: accessing stack-allocated locals
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, 100) // Base pointer
        .with_memory_word_le(96, 0x12EFCDAB) // Word at base-4
        .build();

    // Load with negative offset
    let mut load_inst = IType::default();
    load_inst.rs1 = Register::X1;
    load_inst.rd = Register::X2;
    load_inst.imm.set_signed(-4).unwrap(); // Access address 96

    let lw = Instruction::LW(load_inst);
    cpu.execute_expect(lw, "LW with negative offset");

    cpu.assert_register(
        Register::X2,
        0x12EFCDAB,
        "LW correctly calculates address with negative offset",
    );
}

#[test]
fn test_misaligned_access_behavior() {
    // RISC-V spec allows implementations to support misaligned access
    // This implementation currently allows it - document the behavior
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, 1001) // Misaligned address (not multiple of 4)
        .with_memory_pattern(1001, &[0x11, 0x22, 0x33, 0x44])
        .build();

    let mut inst = IType::default();
    inst.rs1 = Register::X1;
    inst.rd = Register::X2;
    inst.imm.set_unsigned(0).unwrap();

    let lw = Instruction::LW(inst);
    cpu.execute_expect(lw, "LW from misaligned address");

    cpu.assert_register(
        Register::X2,
        0x44332211,
        "Misaligned LW succeeds (implementation choice)",
    );
}
