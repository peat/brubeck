/// Variably sized "immediate" values for RISC-V instruction formats (eg: [`IType`](crate::rv32_i::IType))
#[derive(Debug, Copy, Clone, Default)]
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

#[cfg(test)]
mod immediate_tests {
    use super::*;

    #[test]
    fn always_sign_extend() {
        let mut imm = Immediate::new(8);
        let result = imm.set_signed(-128);
        assert!(result.is_ok());
        assert_eq!(imm.value, 0b1111_1111_1111_1111_1111_1111_1000_0000);

        let result = imm.set_unsigned(255);
        assert!(result.is_ok());
        assert_eq!(imm.value, 0b1111_1111_1111_1111_1111_1111_1111_1111);
    }

    #[test]
    fn min_max() {
        let imm = Immediate::new(8);
        assert_eq!(imm.unsigned_max(), u8::MAX as u32);
        assert_eq!(imm.signed_max(), i8::MAX as i32);
        assert_eq!(imm.signed_min(), i8::MIN as i32);
    }

    #[test]
    fn set_signed() {
        let mut imm = Immediate::new(8);
        let result = imm.set_signed(128);
        assert!(result.is_err());

        let result = imm.set_signed(127);
        assert!(result.is_ok());
        assert_eq!(imm.value, 127u32);

        let result = imm.set_signed(-128);
        assert!(result.is_ok());
        assert_eq!(imm.value, 0b1111_1111_1111_1111_1111_1111_1000_0000);
    }

    #[test]
    fn get_signed() {
        let mut imm = Immediate::new(8);

        let result = imm.set_signed(-128);
        assert!(result.is_ok());
        assert_eq!(imm.as_i32(), -128);

        let result = imm.set_unsigned(127);
        assert!(result.is_ok());
        assert_eq!(imm.as_u32(), 127);

        let result = imm.set_unsigned(255);
        assert!(result.is_ok());
        assert_eq!(imm.as_u32(), u32::MAX);
    }

    #[test]
    fn get_unsigned() {
        let mut imm = Immediate::new(8);

        let result = imm.set_unsigned(63);
        assert!(result.is_ok());
        // top bit is zero
        assert_eq!(imm.as_u32(), 63);

        let result = imm.set_unsigned(255);
        assert!(result.is_ok());
        // top bit is one, should be sign extended
        assert_eq!(imm.as_u32(), u32::MAX);

        let result = imm.set_signed(-128);
        assert!(result.is_ok());
        // top bit is one, should be sign extended
        assert_eq!(imm.as_u32(), 0b1111_1111_1111_1111_1111_1111_1000_0000);
    }
}
