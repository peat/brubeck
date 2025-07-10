//! Basic integration tests for undo/redo functionality
//!
//! These tests cover common use cases and basic functionality.

use crate::common::TestContext;
use crate::helpers::UndoRedoExt;

#[test]
fn test_undo_redo_basic_arithmetic() {
    let mut ctx = TestContext::new();

    // Execute a sequence of arithmetic instructions
    ctx.exec("ADDI x1, x0, 10")
        .exec("ADDI x2, x0, 20")
        .exec("ADD x3, x1, x2");

    // Verify current state
    ctx.check_reg("x3", "0x0000001e");

    // Undo the ADD
    ctx.undo_expect("ADD").check_reg("x3", "0x00000000");

    // Undo the second ADDI
    ctx.undo().check_reg("x2", "0x00000000");

    // Redo the second ADDI
    ctx.redo_expect("ADDI").check_reg("x2", "0x00000014");

    // Redo the ADD
    ctx.redo().check_reg("x3", "0x0000001e");
}

#[test]
fn test_undo_memory_operations() {
    let mut ctx = TestContext::new();

    // Set up base address and value
    ctx.exec("ADDI x1, x0, 1024") // 0x400
        .exec("LI x2, 0xABCD"); // Use LI for larger value

    // Store to memory
    ctx.exec("SW x2, 0(x1)");

    // Load it back to verify
    ctx.exec("LW x3, 0(x1)").check_reg("x3", "0x0000abcd"); // 0xABCD

    // Undo the load (shouldn't affect memory)
    ctx.undo().check_reg("x3", "0x00000000");

    // Undo the store (should restore memory)
    ctx.undo();

    // Try to load again - should get 0 since memory was restored
    ctx.exec("LW x4, 0(x1)").check_reg("x4", "0x00000000");
}

#[test]
fn test_undo_csr_operations() {
    let mut ctx = TestContext::new();

    // Write to MSCRATCH CSR
    ctx.exec("ADDI x1, x0, 1234").exec("CSRRW x2, 0x340, x1"); // Write to mscratch

    // Read it back to verify
    ctx.exec("CSRRS x3, 0x340, x0") // Read mscratch
        .check_reg("x3", "0x000004d2");

    // Undo the read
    ctx.undo().check_reg("x3", "0x00000000");

    // Undo the write
    ctx.undo();

    // Read again - should get original value (0)
    ctx.exec("CSRRS x4, 0x340, x0")
        .check_reg("x4", "0x00000000");
}

#[test]
fn test_undo_limit() {
    let mut ctx = TestContext::new();

    // Execute many instructions
    for i in 1..10 {
        ctx.exec(&format!("ADDI x{}, x0, {}", i, i * 10));
    }

    // Try to undo all - should eventually fail
    let mut undo_count = 0;
    while ctx.inner.interpret("/previous").is_ok() {
        undo_count += 1;
    }

    // Should have undone at least some instructions
    assert!(undo_count > 0);

    // Further undo should fail
    ctx.undo_should_fail();
}

#[test]
fn test_redo_cleared_after_new_instruction() {
    let mut ctx = TestContext::new();

    // Execute some instructions
    ctx.exec("ADDI x1, x0, 10").exec("ADDI x2, x0, 20");

    // Undo one
    ctx.undo().check_reg("x2", "0x00000000");

    // Execute a new instruction
    ctx.exec("ADDI x3, x0, 30");

    // Redo should now fail
    ctx.redo_should_fail();
}

#[test]
fn test_pseudo_instruction_undo() {
    let mut ctx = TestContext::new();

    // Execute pseudo-instructions
    ctx.exec("LI x1, 0x12345").exec("MV x2, x1");

    // Verify state
    ctx.check_reg("x2", "0x00012345"); // 0x12345

    // Undo MV
    ctx.undo_expect("MV").check_reg("x2", "0x00000000");

    // x1 should still have the value
    ctx.check_reg("x1", "0x00012345");
}

#[test]
fn test_branch_instruction_undo() {
    let mut ctx = TestContext::new();

    // Set up for branch
    ctx.exec("ADDI x1, x0, 5").exec("ADDI x2, x0, 5");

    // Get current PC
    let pc_before = ctx.get_pc();

    // Execute branch
    ctx.exec("BEQ x1, x2, 8");

    // PC should have advanced
    let pc_after = ctx.get_pc();
    assert_ne!(pc_before, pc_after);

    // Undo the branch
    ctx.undo();

    // PC should be restored
    let pc_restored = ctx.get_pc();
    assert_eq!(pc_before, pc_restored);
}

#[test]
fn test_invalid_instruction_not_in_history() {
    let mut ctx = TestContext::new();

    // Execute valid instruction
    ctx.exec("ADDI x1, x0, 10");

    // Try invalid instruction
    let err = ctx.exec_fail("INVALID x1, x2, x3");
    assert!(err.contains("Invalid token") || err.contains("Unknown"));

    // Undo should undo the ADDI, not fail on the invalid instruction
    ctx.undo_expect("ADDI").check_reg("x1", "0x00000000");

    // Another undo should fail (no more history)
    ctx.undo_should_fail();
}
