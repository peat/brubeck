/// Variably sized "immediate" values for RISC-V instruction formats (eg: [`IType`](crate::rv32_i::IType))
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Immediate {
    value: u32,
    bits: u8,
}

impl Immediate {
    pub fn new(bits: u8) -> Self {
        Self { value: 0, bits }
    }

    fn extend_sign(&mut self, value: u32) {
        let top_bit_mask: u32 = 1 << (self.bits - 1);
        // if the top bit is 1 extend it, otherwise, just store it as is
        if value & top_bit_mask > 0 {
            let bit_extension: u32 = u32::MAX << (self.bits - 1);
            self.value = value | bit_extension;
        } else {
            self.value = value
        }
    }

    pub fn set_unsigned(&mut self, value: u32) -> Result<(), Error> {
        if value > self.unsigned_max() {
            return Err(Error::OutOfRange(format!(
                "Unsigned value {} is too big for {} bits.",
                value, self.bits
            )));
        }

        self.extend_sign(value);
        Ok(())
    }

    pub fn set_signed(&mut self, value: i32) -> Result<(), Error> {
        if value > self.signed_max() {
            return Err(Error::OutOfRange(format!(
                "Signed value {} is too big for {} bits.",
                value, self.bits
            )));
        }

        if value < self.signed_min() {
            return Err(Error::OutOfRange(format!(
                "Signed value {} is too small for {} bits.",
                value, self.bits
            )));
        }

        self.extend_sign(value as u32);
        Ok(())
    }

    pub fn as_u32(&self) -> u32 {
        self.value
    }

    pub fn as_i32(&self) -> i32 {
        self.value as i32
    }

    pub fn unsigned_max(&self) -> u32 {
        2u32.pow(self.bits as u32) - 1
    }

    pub fn signed_max(&self) -> i32 {
        2i32.pow(self.bits as u32 - 1) - 1
    }

    pub fn signed_min(&self) -> i32 {
        0 - 2i32.pow(self.bits as u32 - 1)
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    OutOfRange(String),
}

// Tests have been moved to tests/unit/components/immediate.rs
