//! Parser module for RISC-V assembly instructions
//!
//! This module implements the four-phase parsing pipeline:
//! 1. Normalize - Clean input (whitespace, case conversion, punctuation)
//! 2. Tokenize - Convert strings to typed tokens
//! 3. Build Commands - Construct validated instruction objects
//! 4. Execute - Run instructions on the CPU emulator

use super::types::{Command, Error, Token};
use crate::rv32_i::{
    BType, IType, Instruction, JType, PseudoInstruction, RType, Register, SType, UType,
};

/// Parses a single line of RISC-V assembly into an executable command.
///
/// # The Four-Phase Parsing Process
///
/// This function implements a traditional compiler front-end with four distinct phases:
///
/// 1. **Normalize**: Clean up whitespace, convert to uppercase, handle punctuation
/// 2. **Tokenize**: Split input into meaningful tokens (instructions, registers, values)
/// 3. **Build Command**: Convert tokens into a structured command with validation
/// 4. **Return Result**: Provide helpful error messages if parsing fails
///
/// # Supported Input Types
///
/// - **Instructions**: `ADDI x1, zero, 100`, `LW x1, 4(x2)`, `JAL x1, 8`
/// - **Pseudo-instructions**: `MV x1, x2`, `LI x1, 0x1234`, `RET`
/// - **Register inspection**: `x1`, `sp`, `PC`
/// - **Multiple formats**: Hex (0x100), binary (0b1010), decimal (42)
///
/// # Examples
/// ```
/// use brubeck::interpreter::Interpreter;
///
/// let mut interpreter = Interpreter::new();
/// let result = interpreter.interpret("ADDI x1, zero, 100");     // Immediate instruction
/// let result = interpreter.interpret("LW x1, 4(x2)");           // Load with offset notation  
/// let result = interpreter.interpret("MV x1, x2");              // Pseudo-instruction
/// let result = interpreter.interpret("x1");                     // Register inspection
/// ```
///
/// # Educational Notes
///
/// This parser follows RISC-V assembly conventions:
/// - All immediates are sign-extended (even ANDI/ORI/XORI)
/// - Supports both standard `LW x1, offset(base)` and legacy `LW x1, base, offset`
/// - Validates instruction arguments and provides helpful error messages
/// - Prevents common mistakes like using PC register inappropriately
pub fn parse(input: &str) -> Result<Command, Error> {
    // Phase 1: Normalize input (clean whitespace, convert case, handle punctuation)
    let normalized = normalize(input);

    // Handle empty input - a common user mistake
    if normalized.is_empty() {
        return Err(Error::Generic("No input provided".to_owned()));
    }

    // Handle commands that start with '/'
    if let Some(first_word) = normalized.first() {
        if first_word.starts_with('/') {
            return match first_word.as_str() {
                "/REGS" | "/R" => {
                    if normalized.len() == 1 {
                        // No arguments, show all registers
                        Ok(Command::ShowRegs)
                    } else {
                        // Parse register arguments
                        let mut regs = Vec::new();
                        for arg in &normalized[1..] {
                            match parse_register(arg) {
                                Some(reg) => regs.push(reg),
                                None => {
                                    return Err(Error::Generic(format!("Invalid register: {arg}")))
                                }
                            }
                        }
                        Ok(Command::ShowSpecificRegs(regs))
                    }
                }
                "/HELP" | "/H" => Ok(Command::ShowHelp),
                #[cfg(feature = "repl")]
                "/UNDO" | "/U" => Ok(Command::Undo),
                #[cfg(feature = "repl")]
                "/REDO" => Ok(Command::Redo),
                _ => Err(Error::Generic(format!("Unknown command: {first_word}"))),
            };
        }
    }

    // Phase 2: Tokenize into meaningful units
    let mut tokens = tokenize(normalized)?;

    // Phase 3: Build a command from tokens with validation
    create_command_from_tokens(&mut tokens)
}

/// Normalizes assembly input by cleaning whitespace and handling punctuation.
///
/// # Normalization Steps
///
/// 1. **Trim whitespace**: Remove leading/trailing spaces
/// 2. **Normalize commas**: Add spaces around commas for consistent parsing
/// 3. **Split on whitespace**: Create individual tokens
/// 4. **Convert to uppercase**: RISC-V mnemonics are case-insensitive
/// 5. **Handle parentheses**: Special processing for offset(register) notation
///
/// # Examples
///
/// - `"  ADDI   x1, x2,  100  "` → `["ADDI", "X1", "X2", "100"]`
/// - `"lw x1, 4(x2)"` → `["LW", "X1", "4(X2)"]`
/// - `"add x1,x2,x3"` → `["ADD", "X1", "X2", "X3"]`
///
/// # Educational Notes
///
/// This demonstrates common text processing techniques:
/// - Whitespace normalization for flexible input
/// - Case conversion for case-insensitive parsing
/// - Special handling of language-specific punctuation
pub fn normalize(input: &str) -> Vec<String> {
    // Special handling for offset(register) notation in loads/stores
    if input.contains('(') && input.contains(')') {
        // First, handle the spaces around commas normally
        let with_comma_spaces = input.trim().replace(',', " , ");

        // Split on whitespace but preserve anything with parentheses
        let parts: Vec<&str> = with_comma_spaces.split_whitespace().collect();
        let mut result = Vec::new();
        let mut i = 0;

        while i < parts.len() {
            let part = parts[i];

            // If this part contains an opening parenthesis, it might be offset(register)
            if part.contains('(') {
                if part.contains(')') {
                    // Complete offset(register) in one token
                    result.push(part.to_uppercase());
                } else {
                    // Split across tokens, need to reconstruct
                    let mut combined = part.to_string();
                    i += 1;
                    while i < parts.len() && !parts[i].contains(')') {
                        combined.push_str(parts[i]);
                        i += 1;
                    }
                    if i < parts.len() {
                        combined.push_str(parts[i]);
                    }
                    result.push(combined.to_uppercase());
                }
            } else if part != "," {
                // Normal token (skip standalone commas)
                result.push(part.to_uppercase());
            }
            i += 1;
        }

        result
    } else {
        // No parentheses, use simple normalization
        input
            .trim()
            .replace(',', " ") // Replace commas with spaces
            .split_whitespace()
            .map(|s| s.to_uppercase())
            .collect()
    }
}

/// Converts normalized string tokens into typed tokens for parsing.
///
/// # Token Types
///
/// - **Instruction**: RISC-V mnemonics (ADD, LW, JAL, etc.)
/// - **PseudoInstruction**: Common shortcuts (MV, LI, RET, etc.)
/// - **Register**: Register names (X0-X31, ABI names like SP, RA)
/// - **Value32**: Immediate values (decimal, hex, binary)
/// - **OffsetRegister**: Load/store notation like `4(X2)`
///
/// # Educational Notes
///
/// This demonstrates the tokenization phase of parsing:
/// - Convert strings to semantic tokens
/// - Identify different types of operands
/// - Prepare for syntax validation
pub fn tokenize(input: Vec<String>) -> Result<Vec<Token>, Error> {
    input.into_iter().map(parse_single_token).collect()
}

/// Suggests a possible instruction if the user made a typo.
///
/// # Common Mistakes
///
/// - `JUMP` → Did you mean `JAL`?
/// - `MOVE` → Did you mean `MV`?
/// - `LOAD` → Did you mean `LW`, `LH`, or `LB`?
/// - `STORE` → Did you mean `SW`, `SH`, or `SB`?
///
/// # Educational Notes
///
/// This provides a helpful user experience by:
/// - Recognizing common mnemonics from other architectures
/// - Guiding users to the correct RISC-V syntax
/// - Teaching RISC-V conventions through suggestions
pub fn suggest_instruction(unknown: &str) -> Option<String> {
    match unknown {
        "JUMP" | "JMP" => Some("JAL (Jump And Link) or J (pseudo-instruction)".to_string()),
        "MOVE" | "MOV" => Some("MV (pseudo-instruction: ADDI rd, rs, 0)".to_string()),
        "LOAD" | "LD" => Some("LW (Load Word), LH (Load Half), or LB (Load Byte)".to_string()),
        "STORE" | "ST" => Some("SW (Store Word), SH (Store Half), or SB (Store Byte)".to_string()),
        "BRANCH" | "BR" => Some("BEQ, BNE, BLT, BGE, BLTU, or BGEU".to_string()),
        "RETURN" => Some("RET (pseudo-instruction: JALR x0, x1, 0)".to_string()),
        "PUSH" => Some("No PUSH in RISC-V. Use: ADDI sp, sp, -4; SW reg, 0(sp)".to_string()),
        "POP" => Some("No POP in RISC-V. Use: LW reg, 0(sp); ADDI sp, sp, 4".to_string()),
        "CALL" => Some("JAL or JALR for function calls".to_string()),
        "CMP" => Some("No CMP in RISC-V. Use SLT/SLTU for comparison".to_string()),
        _ => None,
    }
}

/// Parses a single normalized string into a typed token.
///
/// # Token Recognition Order
///
/// 1. **Offset notation**: `100(X2)` for load/store instructions
/// 2. **Instructions**: Match against all RISC-V mnemonics
/// 3. **Pseudo-instructions**: Common assembly shortcuts
/// 4. **Registers**: X0-X31 or ABI names
/// 5. **Immediate values**: Decimal, hex (0x), or binary (0b)
///
/// # Error Handling
///
/// Provides helpful error messages:
/// - Unknown instructions suggest similar valid ones
/// - Invalid registers show the valid range
/// - Malformed numbers explain valid formats
///
/// # Educational Notes
///
/// This function demonstrates pattern matching in parsing:
/// - Try most specific patterns first (offset notation)
/// - Fall through to more general patterns
/// - Provide helpful errors for invalid input
pub fn parse_single_token(input: String) -> Result<Token, Error> {
    // Check for offset(register) pattern first (e.g., "100(X2)" or "0xFF(SP)")
    if let Some(paren_pos) = input.find('(') {
        if let Some(close_paren) = input.find(')') {
            if close_paren > paren_pos {
                let offset_str = &input[..paren_pos];
                let reg_str = &input[paren_pos + 1..close_paren];

                // Parse the offset value
                let offset = parse_number(offset_str).map_err(|e| {
                    Error::Generic(format!("Invalid offset value '{offset_str}': {e}"))
                })?;

                // Parse the register
                let register = parse_register(reg_str).ok_or_else(|| {
                    Error::Generic(format!("Invalid register in offset notation: {reg_str}"))
                })?;

                return Ok(Token::OffsetRegister { offset, register });
            }
        }
        return Err(Error::Generic(format!(
            "Malformed offset(register) notation: {input}"
        )));
    }

    // Try to parse as instruction mnemonic or pseudo-instruction
    // This is done in parse_instruction_or_pseudo function
    if let Some(token) = parse_instruction_or_pseudo(&input) {
        return Ok(token);
    }

    // Try to parse as CSR name
    if let Some(csr_addr) = parse_csr_name(&input) {
        return Ok(Token::Value32(csr_addr));
    }

    // Try to parse as register (both Xn and ABI names)
    if let Some(register) = parse_register(&input) {
        return Ok(Token::Register(register));
    }

    // Try to parse as immediate value
    match parse_value_or_offset(input.clone()) {
        Ok(token) => Ok(token),
        Err(_) => {
            // Check if it might be a misspelled instruction
            if let Some(suggestion) = suggest_instruction(&input) {
                Err(Error::UnknownInstruction {
                    instruction: input,
                    suggestion: Some(suggestion),
                })
            } else {
                Err(Error::Generic(format!(
                    "Invalid token '{input}': not a valid instruction, register, or number"
                )))
            }
        }
    }
}

/// Creates a command from parsed tokens.
///
/// # Command Types
///
/// Commands are determined by the first token in the input:
///
/// - **Register Inspection**: `x1`, `sp`, `PC` → `Command::Inspect(register)`
/// - **Hardware Instructions**: `ADDI`, `LW`, `JAL` → `Command::Exec(instruction)`
/// - **Pseudo-instructions**: `MV`, `LI`, `RET` → `Command::ExecPseudo(pseudo)`
/// - **Invalid**: Raw numbers like `42` → Error (numbers need context)
///
/// # Token Processing
///
/// The function uses a "consume-first" pattern where it removes the first token
/// to determine the command type, then passes the remaining tokens to specialized
/// builders for validation and construction.
///
/// # Educational Notes
///
/// This demonstrates a common compiler pattern: dispatching based on the first
/// token to specialized handlers. Each handler knows how to validate and build
/// its specific command type.
pub fn create_command_from_tokens(tokens: &mut Vec<Token>) -> Result<Command, Error> {
    if tokens.is_empty() {
        return Err(Error::Generic("Empty tokens in build!".to_owned()));
    }

    // Remove and examine the first token to determine command type
    let first_token = tokens.remove(0);

    match first_token {
        // Single register = error (use /regs instead)
        Token::Register(register) => Err(Error::Generic(format!(
            "Direct register inspection is not supported. Use '/regs {register:?}' instead"
        ))),

        // Raw number without context = error (user probably meant something else)
        Token::Value32(value) => Err(Error::Generic(format!("Value: {value}"))),

        // Hardware instruction = build and validate the full instruction
        Token::Instruction(mut i) => {
            // Import the builder functions we need
            use crate::interpreter::builder::build_instruction;
            Ok(Command::Exec(build_instruction(&mut i, tokens)?))
        }

        // Pseudo-instruction = expand to real instruction(s)
        Token::PseudoInstruction(mut p) => {
            use crate::interpreter::builder::build_pseudo_instruction;
            Ok(Command::ExecPseudo(build_pseudo_instruction(
                &mut p, tokens,
            )?))
        }

        Token::OffsetRegister { offset, register } => Err(Error::Generic(format!(
            "Unexpected offset(register) syntax: {offset}({register:?})"
        ))),
    }
}

/// Attempts to parse a string as a numeric value or offset.
///
/// # Supported Formats
///
/// - **Decimal**: `42`, `-100`, `1024`
/// - **Hexadecimal**: `0x1F`, `0xFF00`, `-0x10`
/// - **Binary**: `0b1010`, `0b11111111`, `-0b100`
///
/// # Sign Extension
///
/// All values are sign-extended to 32 bits for RISC-V compatibility.
/// This matches hardware behavior where immediates are sign-extended.
///
/// # Educational Notes
///
/// This demonstrates number parsing with multiple bases:
/// - Detecting format by prefix (0x, 0b)
/// - Handling negative numbers
/// - Sign extension for immediate values
pub fn parse_value_or_offset(input: String) -> Result<Token, Error> {
    parse_number(&input)
        .map(Token::Value32)
        .map_err(|e| Error::Generic(format!("Invalid immediate value '{input}': {e}")))
}

/// Parses a string into a signed 32-bit integer.
///
/// # Format Detection
///
/// - Starts with `0x` or `0X` → Hexadecimal
/// - Starts with `0b` or `0B` → Binary  
/// - Otherwise → Decimal
///
/// # Examples
///
/// - `"42"` → 42
/// - `"0xFF"` → 255
/// - `"0b1010"` → 10
/// - `"-100"` → -100
/// - `"0xFFFF"` → 65535 (not sign-extended here)
pub fn parse_number(input: &str) -> Result<i32, String> {
    let input = input.trim();

    if input.starts_with("0x") || input.starts_with("0X") {
        // Hexadecimal
        i32::from_str_radix(&input[2..], 16)
            .map_err(|_| format!("Invalid hexadecimal number: {input}"))
    } else if input.starts_with("0b") || input.starts_with("0B") {
        // Binary
        i32::from_str_radix(&input[2..], 2).map_err(|_| format!("Invalid binary number: {input}"))
    } else {
        // Decimal
        input
            .parse::<i32>()
            .map_err(|_| format!("Invalid decimal number: {input}"))
    }
}

/// Parses a string as an instruction mnemonic or pseudo-instruction.
///
/// Returns a Token containing the appropriate instruction or pseudo-instruction
/// if the input matches a known mnemonic, otherwise returns None.
fn parse_instruction_or_pseudo(input: &str) -> Option<Token> {
    match input {
        // Hardware Instructions
        "ADD" => Some(Token::Instruction(Instruction::ADD(RType::default()))),
        "ADDI" => Some(Token::Instruction(Instruction::ADDI(IType::default()))),
        "AND" => Some(Token::Instruction(Instruction::AND(RType::default()))),
        "ANDI" => Some(Token::Instruction(Instruction::ANDI(IType::default()))),
        "AUIPC" => Some(Token::Instruction(Instruction::AUIPC(UType::default()))),
        "BEQ" => Some(Token::Instruction(Instruction::BEQ(BType::default()))),
        "BGE" => Some(Token::Instruction(Instruction::BGE(BType::default()))),
        "BGEU" => Some(Token::Instruction(Instruction::BGEU(BType::default()))),
        "BLT" => Some(Token::Instruction(Instruction::BLT(BType::default()))),
        "BLTU" => Some(Token::Instruction(Instruction::BLTU(BType::default()))),
        "BNE" => Some(Token::Instruction(Instruction::BNE(BType::default()))),
        "EBREAK" => Some(Token::Instruction(Instruction::EBREAK(IType::default()))),
        "ECALL" => Some(Token::Instruction(Instruction::ECALL(IType::default()))),
        "FENCE" => Some(Token::Instruction(Instruction::FENCE(IType::default()))),
        "JAL" => Some(Token::Instruction(Instruction::JAL(JType::default()))),
        "JALR" => Some(Token::Instruction(Instruction::JALR(IType::default()))),
        "LB" => Some(Token::Instruction(Instruction::LB(IType::default()))),
        "LBU" => Some(Token::Instruction(Instruction::LBU(IType::default()))),
        "LH" => Some(Token::Instruction(Instruction::LH(IType::default()))),
        "LHU" => Some(Token::Instruction(Instruction::LHU(IType::default()))),
        "LUI" => Some(Token::Instruction(Instruction::LUI(UType::default()))),
        "LW" => Some(Token::Instruction(Instruction::LW(IType::default()))),
        "NOP" => Some(Token::Instruction(Instruction::NOP)),
        "OR" => Some(Token::Instruction(Instruction::OR(RType::default()))),
        "ORI" => Some(Token::Instruction(Instruction::ORI(IType::default()))),
        "SB" => Some(Token::Instruction(Instruction::SB(SType::default()))),
        "SH" => Some(Token::Instruction(Instruction::SH(SType::default()))),
        "SLL" => Some(Token::Instruction(Instruction::SLL(RType::default()))),
        "SLLI" => Some(Token::Instruction(Instruction::SLLI(IType::default()))),
        "SLT" => Some(Token::Instruction(Instruction::SLT(RType::default()))),
        "SLTI" => Some(Token::Instruction(Instruction::SLTI(IType::default()))),
        "SLTIU" => Some(Token::Instruction(Instruction::SLTIU(IType::default()))),
        "SLTU" => Some(Token::Instruction(Instruction::SLTU(RType::default()))),
        "SRA" => Some(Token::Instruction(Instruction::SRA(RType::default()))),
        "SRAI" => Some(Token::Instruction(Instruction::SRAI(IType::default()))),
        "SRL" => Some(Token::Instruction(Instruction::SRL(RType::default()))),
        "SRLI" => Some(Token::Instruction(Instruction::SRLI(IType::default()))),
        "SUB" => Some(Token::Instruction(Instruction::SUB(RType::default()))),
        "SW" => Some(Token::Instruction(Instruction::SW(SType::default()))),
        "XOR" => Some(Token::Instruction(Instruction::XOR(RType::default()))),
        "XORI" => Some(Token::Instruction(Instruction::XORI(IType::default()))),

        // CSR Instructions
        "CSRRW" => Some(Token::Instruction(Instruction::CSRRW(IType::default()))),
        "CSRRS" => Some(Token::Instruction(Instruction::CSRRS(IType::default()))),
        "CSRRC" => Some(Token::Instruction(Instruction::CSRRC(IType::default()))),
        "CSRRWI" => Some(Token::Instruction(Instruction::CSRRWI(IType::default()))),
        "CSRRSI" => Some(Token::Instruction(Instruction::CSRRSI(IType::default()))),
        "CSRRCI" => Some(Token::Instruction(Instruction::CSRRCI(IType::default()))),

        // Pseudo-instructions
        "MV" => Some(Token::PseudoInstruction(PseudoInstruction::MV {
            rd: Register::X0,
            rs: Register::X0,
        })),
        "NOT" => Some(Token::PseudoInstruction(PseudoInstruction::NOT {
            rd: Register::X0,
            rs: Register::X0,
        })),
        "SEQZ" => Some(Token::PseudoInstruction(PseudoInstruction::SEQZ {
            rd: Register::X0,
            rs: Register::X0,
        })),
        "SNEZ" => Some(Token::PseudoInstruction(PseudoInstruction::SNEZ {
            rd: Register::X0,
            rs: Register::X0,
        })),
        "J" => Some(Token::PseudoInstruction(PseudoInstruction::J { offset: 0 })),
        "JR" => Some(Token::PseudoInstruction(PseudoInstruction::JR {
            rs: Register::X0,
        })),
        "RET" => Some(Token::PseudoInstruction(PseudoInstruction::RET)),
        "LI" => Some(Token::PseudoInstruction(PseudoInstruction::LI {
            rd: Register::X0,
            imm: 0,
        })),

        _ => None,
    }
}

/// Parses a CSR (Control and Status Register) name to its address.
///
/// # Supported CSR Names
///
/// - `CYCLE` (0xC00) - Cycle counter (read-only)
/// - `TIME` (0xC01) - Timer (read-only)  
/// - `INSTRET` (0xC02) - Instructions retired (read-only)
/// - `MSTATUS` (0x300) - Machine status register
/// - `MISA` (0x301) - Machine ISA register
/// - `MIE` (0x304) - Machine interrupt enable
/// - `MTVEC` (0x305) - Machine trap vector base address
/// - `MSCRATCH` (0x340) - Machine scratch register
/// - `MEPC` (0x341) - Machine exception program counter
/// - `MCAUSE` (0x342) - Machine trap cause
/// - `MTVAL` (0x343) - Machine trap value
/// - `MIP` (0x344) - Machine interrupt pending
fn parse_csr_name(input: &str) -> Option<i32> {
    match input {
        "CYCLE" => Some(0xC00),    // Cycle counter (read-only)
        "TIME" => Some(0xC01),     // Timer (read-only)
        "INSTRET" => Some(0xC02),  // Instructions retired (read-only)
        "MSTATUS" => Some(0x300),  // Machine status register
        "MISA" => Some(0x301),     // Machine ISA register
        "MIE" => Some(0x304),      // Machine interrupt enable
        "MTVEC" => Some(0x305),    // Machine trap vector base address
        "MSCRATCH" => Some(0x340), // Machine scratch register
        "MEPC" => Some(0x341),     // Machine exception program counter
        "MCAUSE" => Some(0x342),   // Machine trap cause
        "MTVAL" => Some(0x343),    // Machine trap value
        "MIP" => Some(0x344),      // Machine interrupt pending
        _ => None,
    }
}

/// Parses a register name (both numeric and ABI names).
///
/// # Supported Formats
///
/// - **Numeric**: `X0` through `X31` (case-insensitive)
/// - **ABI names**: `ZERO`, `RA`, `SP`, `GP`, `TP`, `T0`-`T6`, `S0`-`S11`, `A0`-`A7`
/// - **Special**: `PC` (program counter)
///
/// # Register Mapping
///
/// - `X0`/`ZERO` - Always zero
/// - `X1`/`RA` - Return address
/// - `X2`/`SP` - Stack pointer
/// - `X8`/`S0`/`FP` - Frame pointer
/// - etc.
///
/// # Educational Notes
///
/// RISC-V has two naming conventions:
/// - Hardware names (X0-X31) used in documentation
/// - ABI names (RA, SP, etc.) used in assembly code
///   Both are supported for flexibility.
pub fn parse_register(input: &str) -> Option<Register> {
    match input.to_uppercase().as_str() {
        "X0" => Some(Register::X0),
        "X1" => Some(Register::X1),
        "X2" => Some(Register::X2),
        "X3" => Some(Register::X3),
        "X4" => Some(Register::X4),
        "X5" => Some(Register::X5),
        "X6" => Some(Register::X6),
        "X7" => Some(Register::X7),
        "X8" => Some(Register::X8),
        "X9" => Some(Register::X9),
        "X10" => Some(Register::X10),
        "X11" => Some(Register::X11),
        "X12" => Some(Register::X12),
        "X13" => Some(Register::X13),
        "X14" => Some(Register::X14),
        "X15" => Some(Register::X15),
        "X16" => Some(Register::X16),
        "X17" => Some(Register::X17),
        "X18" => Some(Register::X18),
        "X19" => Some(Register::X19),
        "X20" => Some(Register::X20),
        "X21" => Some(Register::X21),
        "X22" => Some(Register::X22),
        "X23" => Some(Register::X23),
        "X24" => Some(Register::X24),
        "X25" => Some(Register::X25),
        "X26" => Some(Register::X26),
        "X27" => Some(Register::X27),
        "X28" => Some(Register::X28),
        "X29" => Some(Register::X29),
        "X30" => Some(Register::X30),
        "X31" => Some(Register::X31),

        // ABI names
        "ZERO" => Some(Register::X0),
        "RA" => Some(Register::X1),
        "SP" => Some(Register::X2),
        "GP" => Some(Register::X3),
        "TP" => Some(Register::X4),
        "T0" => Some(Register::X5),
        "T1" => Some(Register::X6),
        "T2" => Some(Register::X7),
        "S0" | "FP" => Some(Register::X8), // S0 is also frame pointer
        "S1" => Some(Register::X9),
        "A0" => Some(Register::X10),
        "A1" => Some(Register::X11),
        "A2" => Some(Register::X12),
        "A3" => Some(Register::X13),
        "A4" => Some(Register::X14),
        "A5" => Some(Register::X15),
        "A6" => Some(Register::X16),
        "A7" => Some(Register::X17),
        "S2" => Some(Register::X18),
        "S3" => Some(Register::X19),
        "S4" => Some(Register::X20),
        "S5" => Some(Register::X21),
        "S6" => Some(Register::X22),
        "S7" => Some(Register::X23),
        "S8" => Some(Register::X24),
        "S9" => Some(Register::X25),
        "S10" => Some(Register::X26),
        "S11" => Some(Register::X27),
        "T3" => Some(Register::X28),
        "T4" => Some(Register::X29),
        "T5" => Some(Register::X30),
        "T6" => Some(Register::X31),

        // Special registers
        "PC" => Some(Register::PC),

        _ => None,
    }
}
