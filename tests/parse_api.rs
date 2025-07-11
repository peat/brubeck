//! Tests for the new parse() API

use brubeck::{
    parse,
    rv32_i::{Instruction, Register},
    ParseError,
};

#[test]
fn test_parse_regular_instruction() {
    let instructions = parse("ADDI x1, x0, 42").unwrap();
    assert_eq!(instructions.len(), 1);

    match &instructions[0] {
        Instruction::ADDI(i) => {
            assert_eq!(i.rd, Register::X1);
            assert_eq!(i.rs1, Register::X0);
            assert_eq!(i.imm.as_i32(), 42);
        }
        _ => panic!("Expected ADDI instruction"),
    }
}

#[test]
fn test_parse_pseudo_instruction_mv() {
    let instructions = parse("MV x1, x2").unwrap();
    assert_eq!(instructions.len(), 1);

    // MV x1, x2 expands to ADDI x1, x2, 0
    match &instructions[0] {
        Instruction::ADDI(i) => {
            assert_eq!(i.rd, Register::X1);
            assert_eq!(i.rs1, Register::X2);
            assert_eq!(i.imm.as_i32(), 0);
        }
        _ => panic!("Expected ADDI instruction from MV expansion"),
    }
}

#[test]
fn test_parse_pseudo_instruction_li_small() {
    let instructions = parse("LI x1, 42").unwrap();
    assert_eq!(instructions.len(), 1);

    // Small LI values expand to ADDI x1, x0, value
    match &instructions[0] {
        Instruction::ADDI(i) => {
            assert_eq!(i.rd, Register::X1);
            assert_eq!(i.rs1, Register::X0);
            assert_eq!(i.imm.as_i32(), 42);
        }
        _ => panic!("Expected ADDI instruction from LI expansion"),
    }
}

#[test]
fn test_parse_errors() {
    // Unknown instruction
    match parse("ADDDD x1, x2, x3") {
        Err(ParseError::UnknownInstruction { instruction, .. }) => {
            assert_eq!(instruction, "ADDDD");
        }
        _ => panic!("Expected UnknownInstruction error"),
    }

    // Invalid register - parser interprets x99 as an instruction
    match parse("ADD x99, x2, x3") {
        Err(ParseError::UnknownInstruction { instruction, .. }) => {
            assert_eq!(instruction, "X99");
        }
        Err(e) => panic!("Expected UnknownInstruction error, got: {:?}", e),
        Ok(_) => panic!("Expected error, got success"),
    }

    // Test with a clearer invalid register case - x99 is also treated as instruction
    match parse("ADDI x99, x0, 0") {
        Err(ParseError::UnknownInstruction { instruction, .. }) => {
            assert_eq!(instruction, "X99");
        }
        Err(e) => panic!(
            "Expected UnknownInstruction error for ADDI x99, got: {:?}",
            e
        ),
        Ok(_) => panic!("Expected error, got success"),
    }

    // Wrong argument count - wrapped as SyntaxError
    match parse("ADD x1, x2") {
        Err(ParseError::SyntaxError { message }) => {
            assert!(message.contains("Invalid RType arguments"));
        }
        Err(e) => panic!("Expected SyntaxError, got: {:?}", e),
        Ok(_) => panic!("Expected error, got success"),
    }

    // Immediate out of range
    match parse("ADDI x1, x0, 5000") {
        Err(ParseError::ImmediateOutOfRange {
            instruction,
            value,
            min,
            max,
        }) => {
            assert_eq!(instruction, "ADDI");
            assert_eq!(value, 5000);
            assert_eq!(min, -2048);
            assert_eq!(max, 2047);
        }
        _ => panic!("Expected ImmediateOutOfRange error"),
    }
}

#[test]
fn test_parse_slash_commands_rejected() {
    // Slash commands should be rejected by the library
    match parse("/regs") {
        Err(ParseError::SyntaxError { message }) => {
            assert!(message.contains("not supported in library"));
        }
        _ => panic!("Expected SyntaxError for slash command"),
    }
}
