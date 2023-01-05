use crate::rv32_i::{Instruction, Register, ABI, CPU};

pub struct Interpreter {
    cpu: CPU,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            cpu: CPU::default(), // initializes with 1 mebibyte of memory
        }
    }

    pub fn parse(s: &str) -> Result<Input, Error> {
        let target = Self::normalize_input(s);

        match target.as_str() {
            // direct registers
            "PC" => Ok(Input::PC),
            "X0" => Ok(Input::X(Register::X0)),
            "X1" => Ok(Input::X(Register::X1)),
            "X2" => Ok(Input::X(Register::X2)),
            "X3" => Ok(Input::X(Register::X3)),
            "X4" => Ok(Input::X(Register::X4)),
            "X5" => Ok(Input::X(Register::X5)),
            "X6" => Ok(Input::X(Register::X6)),
            "X7" => Ok(Input::X(Register::X7)),
            "X8" => Ok(Input::X(Register::X8)),
            "X9" => Ok(Input::X(Register::X9)),
            "X10" => Ok(Input::X(Register::X10)),
            "X11" => Ok(Input::X(Register::X11)),
            "X12" => Ok(Input::X(Register::X12)),
            "X13" => Ok(Input::X(Register::X13)),
            "X14" => Ok(Input::X(Register::X14)),
            "X15" => Ok(Input::X(Register::X15)),
            "X16" => Ok(Input::X(Register::X16)),
            "X17" => Ok(Input::X(Register::X17)),
            "X18" => Ok(Input::X(Register::X18)),
            "X19" => Ok(Input::X(Register::X19)),
            "X20" => Ok(Input::X(Register::X20)),
            "X21" => Ok(Input::X(Register::X21)),
            "X22" => Ok(Input::X(Register::X22)),
            "X23" => Ok(Input::X(Register::X23)),
            "X24" => Ok(Input::X(Register::X24)),
            "X25" => Ok(Input::X(Register::X25)),
            "X26" => Ok(Input::X(Register::X26)),
            "X27" => Ok(Input::X(Register::X27)),
            "X28" => Ok(Input::X(Register::X28)),
            "X29" => Ok(Input::X(Register::X29)),
            "X30" => Ok(Input::X(Register::X30)),
            "X31" => Ok(Input::X(Register::X31)),

            // ABI registers
            "ZERO" => Ok(Input::X(ABI::Zero.to_register())),
            "RA" => Ok(Input::X(ABI::RA.to_register())),
            "SP" => Ok(Input::X(ABI::SP.to_register())),
            "GP" => Ok(Input::X(ABI::GP.to_register())),
            "TP" => Ok(Input::X(ABI::TP.to_register())),
            "T0" => Ok(Input::X(ABI::T0.to_register())),
            "T1" => Ok(Input::X(ABI::T1.to_register())),
            "T2" => Ok(Input::X(ABI::T2.to_register())),
            "S0" => Ok(Input::X(ABI::S0.to_register())),
            "FP" => Ok(Input::X(ABI::FP.to_register())),
            "S1" => Ok(Input::X(ABI::S1.to_register())),
            "A0" => Ok(Input::X(ABI::A0.to_register())),
            "A1" => Ok(Input::X(ABI::A1.to_register())),
            "A2" => Ok(Input::X(ABI::A2.to_register())),
            "A3" => Ok(Input::X(ABI::A3.to_register())),
            "A4" => Ok(Input::X(ABI::A4.to_register())),
            "A5" => Ok(Input::X(ABI::A5.to_register())),
            "A6" => Ok(Input::X(ABI::A6.to_register())),
            "A7" => Ok(Input::X(ABI::A7.to_register())),
            "S2" => Ok(Input::X(ABI::S2.to_register())),
            "S3" => Ok(Input::X(ABI::S3.to_register())),
            "S4" => Ok(Input::X(ABI::S4.to_register())),
            "S5" => Ok(Input::X(ABI::S5.to_register())),
            "S6" => Ok(Input::X(ABI::S6.to_register())),
            "S7" => Ok(Input::X(ABI::S7.to_register())),
            "S8" => Ok(Input::X(ABI::S8.to_register())),
            "S9" => Ok(Input::X(ABI::S9.to_register())),
            "S10" => Ok(Input::X(ABI::S10.to_register())),
            "S11" => Ok(Input::X(ABI::S11.to_register())),
            "T3" => Ok(Input::X(ABI::T3.to_register())),
            "T4" => Ok(Input::X(ABI::T4.to_register())),
            "T5" => Ok(Input::X(ABI::T5.to_register())),
            "T6" => Ok(Input::X(ABI::T6.to_register())),

            // instructions
            "NOP" => Ok(Input::Exec(Instruction::NOP)),

            e => Err(Error::Generic(format!("I don't understand '{}'", e))),
        }
    }

    fn normalize_input(s: &str) -> String {
        let mut target = s.to_owned();
        target = target.trim().to_owned();
        target.to_uppercase()
    }

    pub fn execute(&mut self, instruction: Instruction) -> Result<String, Error> {
        match self.cpu.execute(instruction) {
            Ok(()) => Ok(format!("{:?}", instruction)),
            e => Err(Error::Generic(format!("{:?}", e))),
        }
    }

    pub fn command(&mut self, input: Input) -> Result<String, Error> {
        match input {
            Input::Exec(instruction) => self.execute(instruction),
            Input::PC => Ok(format!("pc: {} (0x{:x})", self.cpu.pc, self.cpu.pc)),
            Input::X(r) => Ok(format!(
                "{:?}: {:?} (0x{:x})",
                r,
                self.cpu.get_register(r),
                self.cpu.get_register(r)
            )),
        }
    }
}

pub enum Input {
    PC,
    X(Register),
    Exec(Instruction),
}

pub enum Error {
    Generic(String),
}
