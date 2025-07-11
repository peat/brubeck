//! Comprehensive tests covering ALL RV32I instructions
//!
//! These tests ensure every instruction type can be properly undone and redone.

use crate::common::TestContext;
use crate::helpers::UndoRedoExt;
use brubeck::interpreter::Interpreter;

// Helper function to create test context
fn test_ctx(name: &str) -> TestContext<Interpreter> {
    TestContext::<Interpreter>::new().with_name(name)
}

// ==================== ARITHMETIC INSTRUCTIONS ====================

#[test]
fn test_arithmetic_complete() {
    let mut ctx = test_ctx("test_arithmetic_complete");
    // Test ADD, SUB
    ctx.exec("ADDI x1, x0, 50")
        .exec("ADDI x2, x0, 30")
        .exec("ADD x3, x1, x2") // 50 + 30 = 80
        .exec("SUB x4, x1, x2") // 50 - 30 = 20
        .check_reg("x3", "0x00000050")
        .check_reg("x4", "0x00000014");

    // Undo SUB and ADD
    ctx.undo()
        .check_reg("x4", "0x00000000")
        .undo()
        .check_reg("x3", "0x00000000");

    // Redo both
    ctx.redo()
        .check_reg("x3", "0x00000050")
        .redo()
        .check_reg("x4", "0x00000014");
}

#[test]
fn test_arithmetic_edge_cases() {
    let mut ctx = test_ctx("test_arithmetic_edge_cases");
    // Test overflow
    ctx.exec("LI x1, 0x7FFFFFFF") // Max positive
        .exec("ADDI x2, x1, 1"); // Overflow

    // Test with x0 (always zero)
    ctx.exec("ADD x3, x0, x1") // x3 = x1
        .exec("SUB x4, x0, x0"); // x4 = 0

    // Undo and verify
    ctx.undo_n(2).check_reg("x3", "0x00000000");
}

// ==================== LOGICAL INSTRUCTIONS ====================

#[test]
fn test_logical_operations() {
    let mut ctx = test_ctx("test_logical_operations");
    // Setup test values
    ctx.exec("ADDI x1, x0, 0xFF") // 0b11111111
        .exec("ADDI x2, x0, 0x0F"); // 0b00001111

    // Test AND, OR, XOR
    ctx.exec("AND x3, x1, x2") // 0x0F
        .exec("OR x4, x1, x2") // 0xFF
        .exec("XOR x5, x1, x2") // 0xF0
        .check_reg("x3", "0x0000000f") // 0x0F
        .check_reg("x4", "0x000000ff") // 0xFF
        .check_reg("x5", "0x000000f0"); // 0xF0

    // Test immediate variants
    ctx.exec("ANDI x6, x1, 0x0F")
        .exec("ORI x7, x2, 0xF0")
        .exec("XORI x8, x1, -1"); // NOT operation

    // Undo all logical operations
    ctx.undo_n(6).check_regs_zero(3, 8);
}

// ==================== SHIFT INSTRUCTIONS ====================

#[test]
fn test_shift_operations() {
    let mut ctx = test_ctx("test_shift_operations");
    // Test logical shifts
    ctx.exec("ADDI x1, x0, 0x80") // 128
        .exec("SLLI x2, x1, 1") // 256
        .exec("SRLI x3, x1, 1"); // 64

    // Test arithmetic shift (sign extension)
    ctx.exec("LI x4, -2147483648") // Min signed 32-bit
        .exec("SRAI x5, x4, 1"); // Sign-extended shift

    // Test register-based shifts
    ctx.exec("ADDI x6, x0, 2")
        .exec("SLL x7, x1, x6") // Shift by register
        .exec("SRL x8, x1, x6")
        .exec("SRA x9, x4, x6");

    // Undo one and verify
    ctx.undo().check_reg("x9", "0x00000000");
}

// ==================== COMPARISON INSTRUCTIONS ====================

#[test]
fn test_comparison_operations() {
    let mut ctx = test_ctx("test_comparison_operations");
    // Setup test values
    ctx.exec("ADDI x1, x0, -10").exec("ADDI x2, x0, 5");

    // Signed comparisons
    ctx.exec("SLT x3, x1, x2") // -10 < 5 = 1
        .exec("SLT x4, x2, x1") // 5 < -10 = 0
        .exec("SLTI x5, x1, 0") // -10 < 0 = 1
        .check_reg("x3", "0x00000001")
        .check_reg("x4", "0x00000000")
        .check_reg("x5", "0x00000001");

    // Unsigned comparisons
    ctx.exec("SLTU x6, x1, x2") // unsigned: large < 5 = 0
        .exec("SLTIU x7, x1, 0") // unsigned: large < 0 = 0
        .check_reg("x6", "0x00000000");

    // Undo all comparisons
    ctx.undo_n(5);
}

// ==================== MEMORY OPERATIONS ====================

#[test]
fn test_memory_all_sizes() {
    let mut ctx = test_ctx("test_memory_all_sizes");
    // Base address
    ctx.exec("ADDI x1, x0, 0x100");

    // Test byte operations
    ctx.exec("ADDI x2, x0, 0xFF")
        .exec("SB x2, 0(x1)") // Store byte
        .exec("LB x3, 0(x1)") // Load signed byte
        .exec("LBU x4, 0(x1)") // Load unsigned byte
        .check_reg("x3", "0xffffffff") // Sign-extended 0xFF
        .check_reg("x4", "0x000000ff"); // Zero-extended

    // Test halfword operations
    ctx.exec("LI x5, 0x8FFF")
        .exec("SH x5, 4(x1)") // Store halfword
        .exec("LH x6, 4(x1)") // Load signed halfword
        .exec("LHU x7, 4(x1)"); // Load unsigned halfword

    // Test word operations
    ctx.exec("LI x8, 0x12345678")
        .exec("SW x8, 8(x1)") // Store word
        .exec("LW x9, 8(x1)"); // Load word

    // Undo stores (loads don't change state)
    ctx.undo_n(2); // Undo LW and SW

    // Verify memory was restored
    ctx.exec("LW x10, 8(x1)").check_reg("x10", "0x00000000");
}

#[test]
fn test_memory_negative_offsets() {
    let mut ctx = test_ctx("test_memory_negative_offsets");
    // Test negative offsets
    ctx.exec("ADDI x1, x0, 0x200").exec("ADDI x2, x0, 42");

    ctx.exec("SW x2, -4(x1)") // Store at 0x1FC
        .exec("LW x3, -4(x1)") // Load from 0x1FC
        .check_reg("x3", "0x0000002a");

    // Undo and verify
    ctx.undo_n(2);
    ctx.exec("LW x4, -4(x1)").check_reg("x4", "0x00000000");
}

// ==================== UPPER IMMEDIATE INSTRUCTIONS ====================

#[test]
fn test_upper_immediate() {
    let mut ctx = test_ctx("test_upper_immediate");
    // Test LUI
    ctx.exec("LUI x1, 0x12345").check_reg("x1", "0x12345000"); // 0x12345000

    // Test AUIPC (PC-relative)
    ctx.exec("AUIPC x2, 0x1000"); // PC + (0x1000 << 12)

    // Undo both
    ctx.undo()
        .check_reg("x2", "0x00000000")
        .undo()
        .check_reg("x1", "0x00000000");
}

// ==================== BRANCH INSTRUCTIONS ====================

#[test]
fn test_all_branch_types() {
    let mut ctx = test_ctx("test_all_branch_types");
    // Setup comparison values
    ctx.exec("ADDI x1, x0, 10")
        .exec("ADDI x2, x0, 10")
        .exec("ADDI x3, x0, 20")
        .exec("ADDI x4, x0, -5");

    // Test each branch type
    ctx.exec("BEQ x1, x2, 8") // Equal
        .exec("BNE x1, x3, 8") // Not equal
        .exec("BLT x4, x1, 8") // -5 < 10
        .exec("BGE x3, x1, 8") // 20 >= 10
        .exec("BLTU x1, x3, 8") // 10 < 20 (unsigned)
        .exec("BGEU x3, x1, 8"); // 20 >= 10 (unsigned)

    // Undo all branches
    for _ in 0..6 {
        ctx.undo_expect("B"); // All branch instructions start with B
    }
}

// ==================== JUMP INSTRUCTIONS ====================

#[test]
fn test_jump_instructions() {
    let mut ctx = test_ctx("test_jump_instructions");
    // Test JAL (Jump and Link)
    ctx.exec("JAL x1, 0x100").undo_expect("JAL");

    // Test JALR (Jump and Link Register)
    ctx.exec("ADDI x2, x0, 0x200")
        .exec("JALR x3, x2, 8")
        .undo()
        .check_reg("x3", "0x00000000");
}

// ==================== SYSTEM INSTRUCTIONS ====================

#[test]
fn test_system_instructions() {
    let mut ctx = test_ctx("test_system_instructions");
    // Test FENCE (memory ordering) - this should succeed
    ctx.exec("FENCE");

    // Test ECALL/EBREAK - these will trap
    let ecall_err = ctx.exec_fail("ECALL");
    assert!(ecall_err.contains("Environment call"));

    let ebreak_err = ctx.exec_fail("EBREAK");
    assert!(ebreak_err.contains("Breakpoint"));

    // Only FENCE should be undoable
    ctx.undo_expect("FENCE").undo_should_fail();
}

// ==================== CSR INSTRUCTIONS ====================

#[test]
fn test_csr_all_variants() {
    let mut ctx = test_ctx("test_csr_all_variants");
    // Setup values
    ctx.exec("ADDI x1, x0, 0x555") // 1365
        .exec("ADDI x2, x0, 0x2AA"); // 682

    // Test all variants
    ctx.exec("CSRRW x3, 0x340, x1") // Write to mscratch
        .exec("CSRRS x4, 0x340, x2") // Set bits
        .exec("CSRRC x5, 0x340, x1") // Clear bits
        .exec("CSRRWI x6, 0x340, 15") // Write immediate
        .exec("CSRRSI x7, 0x340, 7") // Set immediate bits
        .exec("CSRRCI x8, 0x340, 3"); // Clear immediate bits

    // Undo all CSR operations
    ctx.undo_n(6);

    // Verify CSR is back to original state
    ctx.exec("CSRRS x9, 0x340, x0")
        .check_reg("x9", "0x00000000");
}

#[test]
fn test_csr_read_only() {
    let mut ctx = test_ctx("test_csr_read_only");
    // Try to write to read-only CSR (should fail)
    let err = ctx.exec_fail("CSRRW x1, 0xC00, x0"); // CYCLE is read-only
    assert!(err.contains("read-only"));

    // Failed instruction should not be in history
    ctx.undo_should_fail();
}

// ==================== PSEUDO-INSTRUCTIONS ====================

#[test]
fn test_all_pseudo_instructions() {
    let mut ctx = test_ctx("test_all_pseudo_instructions");
    // Test various pseudo-instructions
    ctx.exec("ADDI x1, x0, 42")
        .exec("MV x2, x1")
        .exec("NOT x3, x1")
        .exec("SEQZ x4, x0") // Should be 1
        .exec("SEQZ x5, x1") // Should be 0
        .exec("SNEZ x6, x1"); // Should be 1

    // Test jumps
    ctx.exec("J 8") // Jump forward
        .exec("ADDI x1, x0, 0x100") // Valid return address
        .exec("RET"); // Return to x1

    // Undo in reverse order
    ctx.undo_n(3) // RET, ADDI, J
        .undo_expect("Undid previous instruction"); // Updated for new delta-based system
}

#[test]
fn test_li_large_values() {
    let mut ctx = test_ctx("test_li_large_values");
    // Test LI with values requiring LUI + ADDI
    ctx.exec("LI x1, 0x12345678")
        .undo_expect("LI") // This undoes the ADDI part
        .undo() // This undoes the LUI part
        .check_reg("x1", "0x00000000");
}

// ==================== COMPLEX SCENARIOS ====================

#[test]
fn test_memory_aliasing() {
    let mut ctx = test_ctx("test_memory_aliasing");
    // Test overlapping memory writes
    ctx.exec("ADDI x1, x0, 0x100")
        .exec("LI x2, 0x12345678")
        .exec("SW x2, 0(x1)") // Write word
        .exec("ADDI x3, x0, 0xFF")
        .exec("SB x3, 1(x1)"); // Overwrite second byte

    // Load and check
    ctx.exec("LW x4, 0(x1)");

    // Undo the byte store
    ctx.undo_n(2); // LW and SB

    // Load again - should have original word
    ctx.exec("LW x5, 0(x1)").check_reg("x5", "0x12345678"); // 0x12345678
}

#[test]
fn test_maximum_undo_redo_cycles() {
    let mut ctx = test_ctx("test_maximum_undo_redo_cycles");
    // Execute several instructions
    for i in 1..=5 {
        ctx.exec(&format!("ADDI x{}, x0, {}", i, i * 10));
    }

    // Undo all
    ctx.undo_n(5);

    // Redo all
    for _ in 0..5 {
        ctx.redo();
    }

    // Verify final state
    for i in 1..=5 {
        ctx.check_reg(&format!("x{i}"), &format!("0x{:08x}", i * 10));
    }

    // Undo some, execute new instruction, verify redo is cleared
    ctx.undo_n(2).exec("ADDI x6, x0, 99").redo_should_fail();
}
