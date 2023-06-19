/// Used to access [CPU](crate::rv32_i::CPU) registers via `get_register()`
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
/// ```
/// use brubeck::rv32_i::*;
///
/// let mut cpu = CPU::default();
/// let nop = Instruction::NOP;
/// let result = cpu.execute(nop);
///
/// // successful execution is ok!
/// assert!(result.is_ok());
///
/// // PC should be incremented by the length of the NOP instruction
/// assert_eq!(cpu.pc, Instruction::LENGTH);
/// ```
pub enum Register {
    #[default]
    X0,
    X1,
    X2,
    X3,
    X4,
    X5,
    X6,
    X7,
    X8,
    X9,
    X10,
    X11,
    X12,
    X13,
    X14,
    X15,
    X16,
    X17,
    X18,
    X19,
    X20,
    X21,
    X22,
    X23,
    X24,
    X25,
    X26,
    X27,
    X28,
    X29,
    X30,
    X31,
    PC,
}

/// ABI ("application binary interface") mapping for [CPU](crate::rv32_i::CPU) registers.
#[derive(Debug, Copy, Clone)]
pub enum ABI {
    /// Always zero; X0 register
    Zero,
    /// Return address; X1 register
    RA,
    /// Stack pointer; X2 register
    SP,
    /// Global pointer; X3 register
    GP,
    /// Thread pointer; X4 register
    TP,
    /// Temporary / alternate link register; X5 register
    T0,
    /// Temporaries; X6 register
    T1,
    /// Temporaries; X7 register
    T2,
    /// Saved register / frame pointer; X8 register
    S0,
    /// Saved register / frame pointer; X8 register
    FP,
    /// Saved register; X9 register
    S1,
    /// Function arguments / return values; X10 register
    A0,
    /// Function arguments / return values; X11 register
    A1,
    /// Function arguments; X12 register
    A2,
    /// Function arguments; X13 register
    A3,
    /// Function arguments; X14 register
    A4,
    /// Function arguments; X15 register
    A5,
    /// Function arguments; X16 register
    A6,
    /// Function arguments; X17 register
    A7,
    /// Saved registers; X18 register
    S2,
    /// Saved registers; X19 register
    S3,
    /// Saved registers; X20 register
    S4,
    /// Saved registers; X21 register
    S5,
    /// Saved registers; X22 register
    S6,
    /// Saved registers; X23 register
    S7,
    /// Saved registers; X24 register
    S8,
    /// Saved registers; X25 register
    S9,
    /// Saved registers; X26 register
    S10,
    /// Saved registers; X27 register
    S11,
    /// Temporaries; X28 register
    T3,
    /// Temporaries; X29 register
    T4,
    /// Temporaries; X30 register
    T5,
    /// Temporaries; X31 register
    T6,
}

impl ABI {
    /// Provides the cooresponding CPU register for the ABI register
    pub fn to_register(&self) -> Register {
        match self {
            Self::Zero => Register::X0,
            Self::RA => Register::X1,
            Self::SP => Register::X2,
            Self::GP => Register::X3,
            Self::TP => Register::X4,
            Self::T0 => Register::X5,
            Self::T1 => Register::X6,
            Self::T2 => Register::X7,
            Self::S0 => Register::X8,
            Self::FP => Register::X8,
            Self::S1 => Register::X9,
            Self::A0 => Register::X10,
            Self::A1 => Register::X11,
            Self::A2 => Register::X12,
            Self::A3 => Register::X13,
            Self::A4 => Register::X14,
            Self::A5 => Register::X15,
            Self::A6 => Register::X16,
            Self::A7 => Register::X17,
            Self::S2 => Register::X18,
            Self::S3 => Register::X19,
            Self::S4 => Register::X20,
            Self::S5 => Register::X21,
            Self::S6 => Register::X22,
            Self::S7 => Register::X23,
            Self::S8 => Register::X24,
            Self::S9 => Register::X25,
            Self::S10 => Register::X26,
            Self::S11 => Register::X27,
            Self::T3 => Register::X28,
            Self::T4 => Register::X29,
            Self::T5 => Register::X30,
            Self::T6 => Register::X31,
        }
    }
}
