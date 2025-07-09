//! Unit tests for system instructions (FENCE, ECALL, EBREAK)
//!
//! These tests verify the behavior of system-level instructions that
//! interact with the execution environment.
//!
//! Reference: RISC-V ISA Manual, Volume I: Unprivileged ISA, Version 20191213
//! Section 2.7 - Memory Ordering Instructions (FENCE)
//! Section 2.8 - Environment Call and Breakpoints (ECALL, EBREAK)
//!
//! Key concepts:
//! - FENCE: Memory ordering instruction (acts as NOP in simple implementations)
//! - ECALL: System call to execution environment
//! - EBREAK: Breakpoint for debugging
//!
//! Encoding details:
//! - FENCE: fm[4] pred[4] succ[4] rs1[5] 000 rd[5] 0001111
//! - ECALL: 000000000000 00000 000 00000 1110011
//! - EBREAK: 000000000001 00000 000 00000 1110011

use brubeck::rv32_i::{
    cpu::Error as CpuError, formats::IType, instructions::Instruction, registers::Register,
};

// Import test helpers
use crate::unit::test_helpers::{CpuAssertions, CpuBuilder, ExecuteWithContext};

#[test]
fn test_fence_basic() {
    // FENCE: Memory ordering instruction
    // In our simple implementation, acts as NOP (just advances PC)
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, 0x12345678) // Some state
        .with_register(Register::X2, 0xABCDEF00) // to verify preservation
        .build();

    // FENCE doesn't use its operands in basic implementation
    let fence = Instruction::FENCE(IType::default());

    // Save state before FENCE
    let x1_before = cpu.get_register(Register::X1);
    let x2_before = cpu.get_register(Register::X2);

    // Execute FENCE
    cpu.execute_expect(fence, "FENCE instruction");

    // Verify only PC changed
    cpu.assert_pc(4, "FENCE advances PC by 4");
    cpu.assert_register(Register::X1, x1_before, "FENCE preserves registers");
    cpu.assert_register(Register::X2, x2_before, "FENCE preserves registers");
}

#[test]
fn test_fence_memory_ordering() {
    // Demonstrate FENCE's intended use for memory ordering
    // Even though our implementation treats it as NOP
    let mut cpu = CpuBuilder::new()
        .with_memory_pattern(0, &[0x12, 0x34, 0x56, 0x78])
        .build();

    let fence = Instruction::FENCE(IType::default());

    // In a real multi-hart system:
    // 1. Store to shared memory location
    // 2. FENCE
    // 3. Store to flag indicating data ready
    //
    // The FENCE ensures the data store is visible before the flag store

    cpu.execute_expect(fence, "FENCE for memory ordering");
    cpu.assert_pc(4, "FENCE completes successfully");

    // In our implementation, FENCE is just a NOP
    // But in real hardware, it ensures memory operation ordering
}

#[test]
fn test_ecall_trap() {
    // ECALL: Environment Call (system call)
    // Should trap to the execution environment
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X10, 1) // A0: Syscall number (by convention)
        .with_register(Register::X11, 100) // A1: First argument
        .build();

    let ecall = Instruction::ECALL(IType::default());

    // ECALL should generate an environment call trap
    let result = cpu.execute(ecall);

    assert!(result.is_err(), "ECALL should trap");
    match result {
        Err(CpuError::EnvironmentCall) => {
            // Expected behavior
        }
        _ => panic!("ECALL should generate EnvironmentCall error"),
    }

    // PC should not advance (trap occurred)
    cpu.assert_pc(0, "PC unchanged after ECALL trap");
}

#[test]
fn test_ecall_syscall_convention() {
    // Demonstrate typical ECALL usage following RISC-V calling convention
    // System call number in a7 (x17), arguments in a0-a5
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X17, 93) // A7: Exit syscall number (Linux)
        .with_register(Register::X10, 0) // A0: Exit code 0
        .build();

    let ecall = Instruction::ECALL(IType::default());

    // In a real system, the trap handler would:
    // 1. Read syscall number from a7
    // 2. Read arguments from a0-a5
    // 3. Perform the system call
    // 4. Return result in a0, error in a1
    // 5. Resume execution after ECALL

    let result = cpu.execute(ecall);
    assert!(
        matches!(result, Err(CpuError::EnvironmentCall)),
        "ECALL triggers system call"
    );
}

#[test]
fn test_ebreak_trap() {
    // EBREAK: Breakpoint instruction
    // Should trap to the debugger
    let mut cpu = CpuBuilder::new()
        .with_pc(0x1000) // Some non-zero PC
        .build();

    let ebreak = Instruction::EBREAK(IType::default());

    // EBREAK should generate a breakpoint trap
    let result = cpu.execute(ebreak);

    assert!(result.is_err(), "EBREAK should trap");
    match result {
        Err(CpuError::Breakpoint) => {
            // Expected behavior
        }
        _ => panic!("EBREAK should generate Breakpoint error"),
    }

    // PC should not advance (trap occurred)
    cpu.assert_pc(0x1000, "PC unchanged after EBREAK trap");
}

#[test]
fn test_ebreak_debugger_usage() {
    // Demonstrate typical EBREAK usage patterns
    let mut cpu = CpuBuilder::new()
        .with_register(Register::X1, 42) // Some value to inspect
        .with_register(Register::X2, 100) // Another value
        .build();

    let ebreak = Instruction::EBREAK(IType::default());

    // Common EBREAK use cases:
    // 1. Software breakpoints inserted by debugger
    // 2. Assertion failures in debug builds
    // 3. Semihosting calls (with special instruction sequence)

    let result = cpu.execute(ebreak);

    // Debugger can inspect state when breakpoint hits
    assert!(
        matches!(result, Err(CpuError::Breakpoint)),
        "EBREAK stops execution for debugging"
    );

    // All state preserved for inspection
    cpu.assert_register(Register::X1, 42, "State preserved at breakpoint");
    cpu.assert_register(Register::X2, 100, "State preserved at breakpoint");
}

#[test]
fn test_system_instruction_encoding() {
    // Verify the encoding expectations for system instructions
    // These have specific bit patterns defined in the ISA

    // FENCE uses I-type format but with special field meanings
    let fence = Instruction::FENCE(IType::default());
    match fence {
        Instruction::FENCE(_) => {
            // In real encoding:
            // - fm field specifies fence mode
            // - pred/succ specify predecessor/successor sets
            // - rs1/rd reserved for future use (should be 0)
        }
        _ => panic!("Expected FENCE instruction"),
    }

    // ECALL and EBREAK use I-type but only differ by immediate
    let ecall = Instruction::ECALL(IType::default());
    let ebreak = Instruction::EBREAK(IType::default());

    match (ecall, ebreak) {
        (Instruction::ECALL(_), Instruction::EBREAK(_)) => {
            // Both use opcode 1110011 (SYSTEM)
            // ECALL: imm = 000000000000
            // EBREAK: imm = 000000000001
        }
        _ => panic!("Expected ECALL and EBREAK instructions"),
    }
}
