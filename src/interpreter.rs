use std::fmt::Display;

use crate::{
    rv32_i::{BType, IType, Instruction, JType, RType, Register, SType, UType, ABI, CPU},
    Immediate,
};

#[derive(Default)]
pub struct Interpreter {
    cpu: CPU,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            cpu: CPU::default(), // initializes with 1 mebibyte of memory
        }
    }

    pub fn interpret(&mut self, input: &str) -> Result<String, Error> {
        let command = parse(input)?;

        self.run_command(command)
    }

    pub fn execute(&mut self, instruction: Instruction) -> Result<String, Error> {
        match self.cpu.execute(instruction) {
            Ok(()) => Ok(format!("{:?}", instruction)),
            e => Err(Error::Generic(format!("{:?}", e))),
        }
    }

    pub fn run_command(&mut self, input: Command) -> Result<String, Error> {
        match input {
            Command::Exec(instruction) => self.execute(instruction),
            Command::PC => Ok(format!("pc: {} (0x{:x})", self.cpu.pc, self.cpu.pc)),
            Command::Inspect(r) => Ok(format!(
                "{:?}: {:?} (0x{:x})",
                r,
                self.cpu.get_register(r),
                self.cpu.get_register(r)
            )),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Command {
    PC,
    Inspect(Register),
    Exec(Instruction),
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Register(Register),
    Instruction(Instruction),
    Value32(u32),
}

#[derive(Debug)]
pub enum Error {
    Generic(String),
    UnrecognizedToken(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err_string = match self {
            Self::Generic(s) => s.to_owned(),
            Self::UnrecognizedToken(s) => format!("Unrecognized token: '{}'", s),
        };

        write!(f, "{err_string}")
    }
}

fn parse(input: &str) -> Result<Command, Error> {
    // clean up whitespace, punctuation, capitalization, etc ...
    let normalized = normalize(input);

    // convert the normalized input into recognized tokens
    let mut tokens = tokenize(normalized)?;

    // build a command from those tokens
    build_command(&mut tokens)
}

fn build_command(tokens: &mut Vec<Token>) -> Result<Command, Error> {
    if tokens.is_empty() {
        return Err(Error::Generic("Empty tokens in build!".to_owned()));
    }

    let first_token = tokens.remove(0);

    match first_token {
        Token::Register(register) => Ok(Command::Inspect(register)),
        Token::Value32(value) => Err(Error::Generic(format!("Value: {}", value))),
        Token::Instruction(mut i) => Ok(Command::Exec(build_instruction(&mut i, tokens)?)),
    }
}

fn build_instruction(instruction: &mut Instruction, args: &[Token]) -> Result<Instruction, Error> {
    let output = match instruction {
        // build instructions
        Instruction::ADD(mut rtype) => Instruction::ADD(build_rtype(&mut rtype, args)?),
        Instruction::ADDI(mut itype) => Instruction::ADDI(build_itype(&mut itype, args)?),

        // unrecognized instruction
        e => {
            return Err(Error::Generic(format!(
                "build_instruction - not implemented: {:?}",
                e
            )))
        }
    };

    Ok(output)
}

fn build_itype(itype: &mut IType, args: &[Token]) -> Result<IType, Error> {
    if let [Token::Register(rd), Token::Register(rs1), Token::Value32(imm)] = args {
        itype.rd = *rd;
        itype.rs1 = *rs1;
        itype
            .imm
            .set_unsigned(*imm)
            .map_err(|e| Error::Generic(format!("{:?}", e)))?;
        Ok(*itype)
    } else {
        Err(Error::Generic(format!(
            "Invalid RType arguments: {:?}",
            args
        )))
    }
}

fn build_rtype(rtype: &mut RType, args: &[Token]) -> Result<RType, Error> {
    if let [Token::Register(rd), Token::Register(rs1), Token::Register(rs2)] = args {
        rtype.rd = *rd;
        rtype.rs1 = *rs1;
        rtype.rs2 = *rs2;
        Ok(*rtype)
    } else {
        Err(Error::Generic(format!(
            "Invalid RType arguments: {:?}",
            args
        )))
    }
}

fn tokenize(input: Vec<String>) -> Result<Vec<Token>, Error> {
    input.into_iter().map(tokenize_one).collect()
}

fn tokenize_one(input: String) -> Result<Token, Error> {
    let token = match input.as_str() {
        // registers
        "PC" => Token::Register(Register::PC),
        "X0" => Token::Register(Register::X0),
        "X1" => Token::Register(Register::X1),
        "X2" => Token::Register(Register::X2),
        "X3" => Token::Register(Register::X3),
        "X4" => Token::Register(Register::X4),
        "X5" => Token::Register(Register::X5),
        "X6" => Token::Register(Register::X6),
        "X7" => Token::Register(Register::X7),
        "X8" => Token::Register(Register::X8),
        "X9" => Token::Register(Register::X9),
        "X10" => Token::Register(Register::X10),
        "X11" => Token::Register(Register::X11),
        "X12" => Token::Register(Register::X12),
        "X13" => Token::Register(Register::X13),
        "X14" => Token::Register(Register::X14),
        "X15" => Token::Register(Register::X15),
        "X16" => Token::Register(Register::X16),
        "X17" => Token::Register(Register::X17),
        "X18" => Token::Register(Register::X18),
        "X19" => Token::Register(Register::X19),
        "X20" => Token::Register(Register::X20),
        "X21" => Token::Register(Register::X21),
        "X22" => Token::Register(Register::X22),
        "X23" => Token::Register(Register::X23),
        "X24" => Token::Register(Register::X24),
        "X25" => Token::Register(Register::X25),
        "X26" => Token::Register(Register::X26),
        "X27" => Token::Register(Register::X27),
        "X28" => Token::Register(Register::X28),
        "X29" => Token::Register(Register::X29),
        "X30" => Token::Register(Register::X30),
        "X31" => Token::Register(Register::X31),

        // ABI-named registers
        "ZERO" => Token::Register(ABI::Zero.to_register()),
        "RA" => Token::Register(ABI::RA.to_register()),
        "SP" => Token::Register(ABI::SP.to_register()),
        "GP" => Token::Register(ABI::GP.to_register()),
        "TP" => Token::Register(ABI::TP.to_register()),
        "T0" => Token::Register(ABI::T0.to_register()),
        "T1" => Token::Register(ABI::T1.to_register()),
        "T2" => Token::Register(ABI::T2.to_register()),
        "S0" => Token::Register(ABI::S0.to_register()),
        "FP" => Token::Register(ABI::FP.to_register()),
        "S1" => Token::Register(ABI::S1.to_register()),
        "A0" => Token::Register(ABI::A0.to_register()),
        "A1" => Token::Register(ABI::A1.to_register()),
        "A2" => Token::Register(ABI::A2.to_register()),
        "A3" => Token::Register(ABI::A3.to_register()),
        "A4" => Token::Register(ABI::A4.to_register()),
        "A5" => Token::Register(ABI::A5.to_register()),
        "A6" => Token::Register(ABI::A6.to_register()),
        "A7" => Token::Register(ABI::A7.to_register()),
        "S2" => Token::Register(ABI::S2.to_register()),
        "S3" => Token::Register(ABI::S3.to_register()),
        "S4" => Token::Register(ABI::S4.to_register()),
        "S5" => Token::Register(ABI::S5.to_register()),
        "S6" => Token::Register(ABI::S6.to_register()),
        "S7" => Token::Register(ABI::S7.to_register()),
        "S8" => Token::Register(ABI::S8.to_register()),
        "S9" => Token::Register(ABI::S9.to_register()),
        "S10" => Token::Register(ABI::S10.to_register()),
        "S11" => Token::Register(ABI::S11.to_register()),
        "T3" => Token::Register(ABI::T3.to_register()),
        "T4" => Token::Register(ABI::T4.to_register()),
        "T5" => Token::Register(ABI::T5.to_register()),
        "T6" => Token::Register(ABI::T6.to_register()),

        // instructions
        "ADD" => Token::Instruction(Instruction::ADD(RType::default())),
        "ADDI" => Token::Instruction(Instruction::ADDI(IType::default())),
        "AND" => Token::Instruction(Instruction::AND(RType::default())),
        "ANDI" => Token::Instruction(Instruction::ANDI(IType::default())),
        "AUIPC" => Token::Instruction(Instruction::AUIPC(UType::default())),
        "BEQ" => Token::Instruction(Instruction::BEQ(BType::default())),
        "BGE" => Token::Instruction(Instruction::BGE(BType::default())),
        "BGEU" => Token::Instruction(Instruction::BGEU(BType::default())),
        "BLT" => Token::Instruction(Instruction::BLT(BType::default())),
        "BLTU" => Token::Instruction(Instruction::BLTU(BType::default())),
        "BNE" => Token::Instruction(Instruction::BNE(BType::default())),
        "EBREAK" => Token::Instruction(Instruction::EBREAK(IType::default())),
        "ECALL" => Token::Instruction(Instruction::ECALL(IType::default())),
        "FENCE" => Token::Instruction(Instruction::FENCE(IType::default())),
        "JAL" => Token::Instruction(Instruction::JAL(JType::default())),
        "JALR" => Token::Instruction(Instruction::JALR(IType::default())),
        "LB" => Token::Instruction(Instruction::LB(IType::default())),
        "LBU" => Token::Instruction(Instruction::LBU(IType::default())),
        "LH" => Token::Instruction(Instruction::LH(IType::default())),
        "LHU" => Token::Instruction(Instruction::LHU(IType::default())),
        "LUI" => Token::Instruction(Instruction::LUI(UType::default())),
        "LW" => Token::Instruction(Instruction::LW(IType::default())),
        "NOP" => Token::Instruction(Instruction::NOP),
        "OR" => Token::Instruction(Instruction::OR(RType::default())),
        "ORI" => Token::Instruction(Instruction::ORI(IType::default())),
        "SB" => Token::Instruction(Instruction::SB(SType::default())),
        "SH" => Token::Instruction(Instruction::SH(SType::default())),
        "SLL" => Token::Instruction(Instruction::SLL(RType::default())),
        "SLLI" => Token::Instruction(Instruction::SLLI(IType::default())),
        "SLT" => Token::Instruction(Instruction::SLT(RType::default())),
        "SLTI" => Token::Instruction(Instruction::SLTI(IType::default())),
        "SLTIU" => Token::Instruction(Instruction::SLTIU(IType::default())),
        "SLTU" => Token::Instruction(Instruction::SLTU(RType::default())),
        "SRA" => Token::Instruction(Instruction::SRA(RType::default())),
        "SRAI" => Token::Instruction(Instruction::SRAI(IType::default())),
        "SRL" => Token::Instruction(Instruction::SRL(RType::default())),
        "SRLI" => Token::Instruction(Instruction::SRLI(IType::default())),
        "SUB" => Token::Instruction(Instruction::SUB(RType::default())),
        "SW" => Token::Instruction(Instruction::SW(SType::default())),
        "XOR" => Token::Instruction(Instruction::XOR(RType::default())),
        "XORI" => Token::Instruction(Instruction::XORI(IType::default())),

        // everything else could be a value
        _ => parse_value(input)?,
    };

    Ok(token)
}

fn parse_value(input: String) -> Result<Token, Error> {
    // it's gotta be a number; we might build something more NASM-complete later
    match input.parse::<i32>() {
        Ok(value) => Ok(Token::Value32(value as u32)),
        Err(_) => Err(Error::UnrecognizedToken(input)),
    }
}

fn normalize(input: &str) -> Vec<String> {
    let mut output = vec![];

    // split on whitespace and commas, uppercase
    for ws in input.to_uppercase().split_whitespace() {
        for t in ws.split(',') {
            // ignore empty tokens
            if t.is_empty() {
                continue;
            }
            output.push(t.to_owned());
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_input() {
        let a = "whitespace is   weird \t and can be dumb";
        let b = "commas ,are, ok\t,too";

        assert_eq!(
            normalize(a),
            vec!["WHITESPACE", "IS", "WEIRD", "AND", "CAN", "BE", "DUMB"]
        );
        assert_eq!(normalize(b), vec!["COMMAS", "ARE", "OK", "TOO"]);
    }

    #[test]
    fn tokenize_input() {
        let a = "ADD x1, x2, x3";

        let normalized = normalize(a);
        let result = tokenize(normalized);

        assert!(result.is_ok());

        let tokens = result.unwrap();

        assert_eq!(
            tokens,
            vec![
                Token::Instruction(Instruction::ADD(RType::default())),
                Token::Register(Register::X1),
                Token::Register(Register::X2),
                Token::Register(Register::X3)
            ]
        );
    }

    #[test]
    fn parse_command() {
        let a = "ADD x1, x2, x3";
        let result = parse(a);

        assert!(result.is_ok());

        let rtype = RType {
            rd: Register::X1,
            rs1: Register::X2,
            rs2: Register::X3,
            ..Default::default()
        };

        assert_eq!(result.unwrap(), Command::Exec(Instruction::ADD(rtype)));
    }

    #[test]
    fn trivial_add() {
        let mut i = Interpreter::default();
        i.cpu.x2 = 3;
        i.cpu.x3 = 5;

        assert_eq!(i.cpu.x1, 0);

        let input = "ADD x1, x2, x3";
        assert!(i.interpret(input).is_ok());

        assert_eq!(i.cpu.x1, 8);
    }
}
