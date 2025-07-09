//! Implementation of the RISC-V 32bit Integer Base (RV32I) ISA
//!
//! This module provides a complete implementation of the RV32I base integer
//! instruction set, including:
//! 
//! - The [CPU] emulator with registers and memory
//! - All 47 RV32I [instructions](Instruction) 
//! - Instruction encoding [formats](formats) (R, I, S, B, U, J types)
//! - [Register](Register) definitions with ABI names
//! - Common [pseudo-instructions](pseudo_instructions) that expand to RV32I instructions

pub mod cpu;
pub mod formats;
pub mod instructions;
pub mod pseudo_instructions;
pub mod registers;

pub use cpu::*;
pub use formats::*;
pub use instructions::*;
pub use pseudo_instructions::*;
pub use registers::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nop() {
        let mut cpu = CPU::default();
        let nop = Instruction::NOP;

        // start from zero in the PC
        let result = cpu.execute(nop);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, 4);

        // incrementing PC
        let result = cpu.execute(nop);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, 8);
    }

    #[test]
    fn add_sub() {
        let mut cpu = CPU::default();
        let mut inst = RType::default();

        inst.rd = Register::X1;
        inst.rs1 = Register::X2;
        inst.rs2 = Register::X3;

        let add = Instruction::ADD(inst);
        let sub = Instruction::SUB(inst);

        // zero values
        let result = cpu.execute(add);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 0);

        // non-overflowing add and sub
        cpu.set_register(Register::X2, 8);
        cpu.set_register(Register::X3, 4);

        let result = cpu.execute(add);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 12);

        let result = cpu.execute(sub);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 4);

        // overflowing addition
        cpu.set_register(Register::X2, 3);
        cpu.set_register(Register::X3, u32::MAX - 1);

        let result = cpu.execute(add);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 1);

        let result = cpu.execute(sub);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 5);
    }

    #[test]
    fn addi() {
        let mut cpu = CPU::default();
        let mut inst = IType::new();

        inst.rd = Register::X1;
        inst.rs1 = Register::X1;
        inst.imm.set_unsigned(0).unwrap();

        let addi = Instruction::ADDI(inst);

        // zero value
        let result = cpu.execute(addi);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 0);

        // positive values
        inst.imm.set_unsigned(5).unwrap();
        let addi = Instruction::ADDI(inst);
        let result = cpu.execute(addi);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 5);

        // negative values; this is a mess!
        let result = inst.imm.set_signed(-3);
        assert!(result.is_ok());
        let addi = Instruction::ADDI(inst);
        let result = cpu.execute(addi);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 2);
    }

    #[test]
    fn slti() {
        let mut cpu = CPU::default();
        let mut inst = IType::default();

        inst.rd = Register::X1;
        inst.rs1 = Register::X2;
        inst.imm.set_unsigned(0).unwrap();

        let slti = Instruction::SLTI(inst);

        // zero / equal value
        let result = cpu.execute(slti);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 0);
        assert_eq!(cpu.pc, Instruction::LENGTH);

        // greater than value
        inst.imm.set_signed(1).unwrap();
        let slti = Instruction::SLTI(inst);
        let result = cpu.execute(slti);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 1);
        assert_eq!(cpu.pc, Instruction::LENGTH * 2);

        // less than value (negative, just for kicks)
        inst.imm.set_signed(-1).unwrap();
        let slti = Instruction::SLTI(inst);
        let result = cpu.execute(slti);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 0);
        assert_eq!(cpu.pc, Instruction::LENGTH * 3);
    }

    #[test]
    fn sltiu() {
        let mut cpu = CPU::default();
        let mut inst = IType::default();

        cpu.x2 = 255; // initial value to compare against

        inst.rd = Register::X1;
        inst.rs1 = Register::X2;

        // equal value
        inst.imm.set_unsigned(255).unwrap();
        let sltiu = Instruction::SLTIU(inst);
        let result = cpu.execute(sltiu);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 0);
        assert_eq!(cpu.pc, Instruction::LENGTH);

        // greater than value
        inst.imm.set_unsigned(256).unwrap();
        let sltiu = Instruction::SLTIU(inst);
        let result = cpu.execute(sltiu);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 1);
        assert_eq!(cpu.pc, Instruction::LENGTH * 2);

        // less than value
        inst.imm.set_unsigned(254).unwrap();
        let sltiu = Instruction::SLTIU(inst);
        let result = cpu.execute(sltiu);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 0);
        assert_eq!(cpu.pc, Instruction::LENGTH * 3);
    }

    #[test]
    fn andi_ori_xori() {
        let mut cpu = CPU::default();
        let mut inst = IType::default();

        inst.rd = Register::X1;
        inst.rs1 = Register::X2;

        // all 1s across the register and imm
        let result = inst.imm.set_unsigned(inst.imm.unsigned_max());
        assert!(result.is_ok());
        cpu.x2 = u32::MAX;

        let andi = Instruction::ANDI(inst);
        let result = cpu.execute(andi);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, u32::MAX);

        let ori = Instruction::ORI(inst);
        let result = cpu.execute(ori);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, u32::MAX);

        let xori = Instruction::XORI(inst);
        let result = cpu.execute(xori);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 0);

        // all 0s in imm
        let result = inst.imm.set_unsigned(0);
        assert!(result.is_ok());
        cpu.x2 = u32::MAX;

        let andi = Instruction::ANDI(inst);
        let result = cpu.execute(andi);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 0);

        let ori = Instruction::ORI(inst);
        let result = cpu.execute(ori);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, u32::MAX);

        let xori = Instruction::XORI(inst);
        let result = cpu.execute(xori);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, u32::MAX);
    }

    #[test]
    fn lui() {
        let mut cpu = CPU::default();
        let mut inst = UType::default();

        inst.rd = Register::X1;
        let result = inst.imm.set_unsigned(1);
        assert!(result.is_ok());

        let lui = Instruction::LUI(inst);
        let result = cpu.execute(lui);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 0b0000_0000_0000_0000_0001_0000_0000_0000);
    }

    #[test]
    fn auipc() {
        let mut cpu = CPU::default();
        let mut inst = UType::default();

        inst.rd = Register::X1;
        let result = inst.imm.set_unsigned(1);
        assert!(result.is_ok());

        // from PC 0
        let auipc = Instruction::AUIPC(inst);
        let result = cpu.execute(auipc);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 0b0000_0000_0000_0000_0001_0000_0000_0000);

        // from 0 + RV32I::LENGTH
        let result = cpu.execute(auipc);
        assert!(result.is_ok());
        assert_eq!(cpu.x1, 0b0000_0000_0000_0000_0001_0000_0000_0100);
    }

    #[test]
    fn jal() {
        let mut cpu = CPU::default();
        let mut inst = JType::default();

        inst.rd = Register::X1;
        let result = inst.imm.set_unsigned(4);
        assert!(result.is_ok());

        let jal = Instruction::JAL(inst);
        let result = cpu.execute(jal);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, 8); // current pc (0) + (4 * 2)
        assert_eq!(cpu.x1, 4); // current pc (0) + RV32I::LENGTH

        // misalignment check!
        let result = inst.imm.set_unsigned(1);
        assert!(result.is_ok());
        let jal = Instruction::JAL(inst);
        let result = cpu.execute(jal);
        assert!(result.is_err());
    }

    #[test]
    fn jalr() {
        let mut cpu = CPU::default();
        let mut inst = IType::default();

        inst.rs1 = Register::X2;
        inst.rd = Register::X1;
        let result = inst.imm.set_unsigned(12);
        assert!(result.is_ok());

        let jalr = Instruction::JALR(inst);
        let result = cpu.execute(jalr);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, 12);
        assert_eq!(cpu.x1, 4);

        cpu.pc = 0;
        cpu.x2 = 24;
        let result = inst.imm.set_signed(-12);
        assert!(result.is_ok());

        let jalr = Instruction::JALR(inst);
        let result = cpu.execute(jalr);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, 12);
        assert_eq!(cpu.x1, 4);
    }

    #[test]
    fn beq() {
        let mut cpu = CPU::default();
        let mut inst = BType::default();

        cpu.x1 = 24;
        cpu.x2 = 24;
        cpu.pc = 0;

        inst.rs1 = Register::X1;
        inst.rs2 = Register::X2;

        inst.imm.set_signed(64).unwrap();
        let beq = Instruction::BEQ(inst);
        let result = cpu.execute(beq);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, 128); // doubled

        inst.imm.set_signed(-128).unwrap();
        let beq = Instruction::BEQ(inst);
        let result = cpu.execute(beq);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, -128i32 as u32); // doubled

        inst.rs1 = Register::X3;
        cpu.pc = 0;

        inst.imm.set_signed(64).unwrap();
        let beq = Instruction::BEQ(inst);
        let result = cpu.execute(beq);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, Instruction::LENGTH); // skipped
    }

    #[test]
    fn bne() {
        let mut cpu = CPU::default();
        let mut inst = BType::default();

        cpu.x1 = 23;
        cpu.x2 = 24;
        cpu.pc = 0;

        inst.rs1 = Register::X1;
        inst.rs2 = Register::X2;

        inst.imm.set_signed(64).unwrap();
        let bne = Instruction::BNE(inst);
        let result = cpu.execute(bne);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, 128); // doubled

        inst.imm.set_signed(-128).unwrap();
        let bne = Instruction::BNE(inst);
        let result = cpu.execute(bne);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, -128i32 as u32); // doubled

        cpu.x1 = 24; // should be equal now
        cpu.pc = 0;

        inst.imm.set_signed(64).unwrap();
        let bne = Instruction::BNE(inst);
        let result = cpu.execute(bne);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, Instruction::LENGTH); // skipped
    }

    #[test]
    fn blt() {
        let mut cpu = CPU::default();
        let mut inst = BType::default();

        cpu.x1 = 23;
        cpu.x2 = 24;
        cpu.pc = 0;

        inst.rs1 = Register::X1;
        inst.rs2 = Register::X2;

        inst.imm.set_signed(64).unwrap();
        let blt = Instruction::BLT(inst);
        let result = cpu.execute(blt);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, 128); // doubled

        inst.imm.set_signed(-128).unwrap();
        let blt = Instruction::BLT(inst);
        let result = cpu.execute(blt);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, -128i32 as u32); // doubled

        cpu.x1 = 24; // should be equal now
        cpu.pc = 0;

        inst.imm.set_signed(64).unwrap();
        let blt = Instruction::BLT(inst);
        let result = cpu.execute(blt);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, Instruction::LENGTH); // skipped
    }

    #[test]
    fn bltu() {
        let mut cpu = CPU::default();
        let mut inst = BType::default();

        cpu.x1 = 23;
        cpu.x2 = 24;
        cpu.pc = 0;

        inst.rs1 = Register::X1;
        inst.rs2 = Register::X2;

        inst.imm.set_unsigned(64).unwrap();
        let bltu = Instruction::BLTU(inst);
        let result = cpu.execute(bltu);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, 128); // doubled

        inst.imm.set_unsigned(0).unwrap();
        let bltu = Instruction::BLTU(inst);
        let result = cpu.execute(bltu);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, 128i32 as u32); // doubled

        cpu.x1 = 24; // should be equal now
        cpu.pc = 0;

        inst.imm.set_unsigned(64).unwrap();
        let bltu = Instruction::BLTU(inst);
        let result = cpu.execute(bltu);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, Instruction::LENGTH); // skipped
    }

    #[test]
    fn bge() {
        let mut cpu = CPU::default();
        let mut inst = BType::default();

        cpu.x1 = 24;
        cpu.x2 = 23;
        cpu.pc = 0;

        inst.rs1 = Register::X1;
        inst.rs2 = Register::X2;

        inst.imm.set_signed(64).unwrap();
        let bge = Instruction::BGE(inst);
        let result = cpu.execute(bge);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, 128); // doubled

        inst.imm.set_signed(-128).unwrap();
        let bge = Instruction::BGE(inst);
        let result = cpu.execute(bge);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, -128i32 as u32); // doubled

        cpu.x2 = 24; // should be equal now
        cpu.pc = 0;

        inst.imm.set_signed(64).unwrap();
        let bge = Instruction::BGE(inst);
        let result = cpu.execute(bge);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, 128); // equal, taken
    }

    #[test]
    fn bgeu() {
        let mut cpu = CPU::default();
        let mut inst = BType::default();

        cpu.x1 = 24;
        cpu.x2 = 23;
        cpu.pc = 0;

        inst.rs1 = Register::X1;
        inst.rs2 = Register::X2;

        inst.imm.set_unsigned(64).unwrap();
        let bgeu = Instruction::BGEU(inst);
        let result = cpu.execute(bgeu);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, 128); // doubled

        inst.imm.set_unsigned(0).unwrap();
        let bgeu = Instruction::BGEU(inst);
        let result = cpu.execute(bgeu);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, 128i32 as u32); // doubled

        cpu.x2 = 24; // should be equal now
        cpu.pc = 0;

        inst.imm.set_unsigned(64).unwrap();
        let bgeu = Instruction::BGEU(inst);
        let result = cpu.execute(bgeu);
        assert!(result.is_ok());
        assert_eq!(cpu.pc, 128); // equal, taken
    }

    #[test]
    fn lw_lh_lb() {
        let mut cpu = CPU::default();
        let mut inst = IType::default();

        cpu.memory[1024] = 1;
        cpu.memory[1025] = 2;
        cpu.memory[1026] = 3;
        cpu.memory[1027] = 4;

        cpu.x1 = 1024;

        inst.rs1 = Register::X1;
        inst.rd = Register::X2;

        inst.imm.set_unsigned(0).unwrap(); // zero offset
        let lw = Instruction::LW(inst);
        let result = cpu.execute(lw);
        assert!(result.is_ok());
        let lw_target = u32::from_le_bytes([1, 2, 3, 4]);
        assert_eq!(cpu.x2, lw_target);

        inst.imm.set_unsigned(2).unwrap(); // +2 offset
        let lw = Instruction::LW(inst);
        let result = cpu.execute(lw);
        assert!(result.is_ok());
        let lw_target = u32::from_le_bytes([3, 4, 0, 0]);
        assert_eq!(cpu.x2, lw_target);

        inst.imm.set_unsigned(0).unwrap(); // zero offset
        let lh = Instruction::LH(inst);
        let result = cpu.execute(lh);
        assert!(result.is_ok());
        let lh_target = u32::from_le_bytes([1, 2, 0, 0]);
        assert_eq!(cpu.x2, lh_target);

        inst.imm.set_unsigned(1).unwrap(); // +1 offset
        let lh = Instruction::LH(inst);
        let result = cpu.execute(lh);
        assert!(result.is_ok());
        let lh_target = u32::from_le_bytes([2, 3, 0, 0]);
        assert_eq!(cpu.x2, lh_target);

        inst.imm.set_unsigned(0).unwrap(); // zero offset
        let lb = Instruction::LB(inst);
        let result = cpu.execute(lb);
        assert!(result.is_ok());
        let lb_target = u32::from_le_bytes([1, 0, 0, 0]);
        assert_eq!(cpu.x2, lb_target);

        inst.imm.set_unsigned(1).unwrap(); // +1 offset
        let lb = Instruction::LB(inst);
        let result = cpu.execute(lb);
        assert!(result.is_ok());
        let lb_target = u32::from_le_bytes([2, 0, 0, 0]);
        assert_eq!(cpu.x2, lb_target);
    }

    #[test]
    fn sw_sh_sb() {
        let mut cpu = CPU::default();
        let mut inst = SType::default();

        cpu.x1 = 100; // base address
        cpu.x2 = 0b1111_1111_1111_1110_1111_1100_1111_1000; // value to store

        inst.rs1 = Register::X1;
        inst.rs2 = Register::X2;

        inst.imm.set_unsigned(0).unwrap(); // zero offset
        let sw = Instruction::SW(inst);
        let result = cpu.execute(sw);
        assert!(result.is_ok());
        assert_eq!(cpu.memory[100], 0b1111_1000);
        assert_eq!(cpu.memory[101], 0b1111_1100);
        assert_eq!(cpu.memory[102], 0b1111_1110);
        assert_eq!(cpu.memory[103], 0b1111_1111);

        cpu.x1 = 200; // base address
        let sh = Instruction::SH(inst);
        let result = cpu.execute(sh);
        assert!(result.is_ok());
        assert_eq!(cpu.memory[200], 0b1111_1000);
        assert_eq!(cpu.memory[201], 0b1111_1100);

        cpu.x1 = 300; // base address
        let sb = Instruction::SB(inst);
        let result = cpu.execute(sb);
        assert!(result.is_ok());
        assert_eq!(cpu.memory[300], 0b1111_1000);
    }

    #[test]
    fn sw_lw_roundtrip() {
        let mut cpu = CPU::default();

        cpu.x1 = 100; // base address
        cpu.x2 = 0b1111_1111_1111_1110_1111_1100_1111_1000; // value to store

        let mut store_inst = SType::default();
        store_inst.rs1 = Register::X1;
        store_inst.rs2 = Register::X2;

        let sw = Instruction::SW(store_inst);
        let result = cpu.execute(sw);
        assert!(result.is_ok());

        let mut load_inst = IType::default();
        load_inst.rs1 = Register::X1; // base address
        load_inst.rd = Register::X3; // destination register

        let lw = Instruction::LW(load_inst);
        let result = cpu.execute(lw);
        assert!(result.is_ok());
        assert_eq!(cpu.x2, cpu.x3);
    }

    #[test]
    fn sh_lh_roundtrip() {
        let mut cpu = CPU::default();

        cpu.x1 = 100; // base address
        cpu.x2 = 0b1111_1111_1111_1110_1111_1100_1111_1000; // value to store

        let mut store_inst = SType::default();
        store_inst.rs1 = Register::X1;
        store_inst.rs2 = Register::X2;

        let sh = Instruction::SH(store_inst);
        let result = cpu.execute(sh);
        assert!(result.is_ok());

        let mut load_inst = IType::default();
        load_inst.rs1 = Register::X1; // base address
        load_inst.rd = Register::X3; // destination register

        // Use LHU for unsigned roundtrip since the value has sign bit set
        let lhu = Instruction::LHU(load_inst);
        let result = cpu.execute(lhu);
        assert!(result.is_ok());
        assert_eq!(cpu.x3, 0b1111_1100_1111_1000);
    }

    #[test]
    fn sb_lb_roundtrip() {
        let mut cpu = CPU::default();

        cpu.x1 = 100; // base address
        cpu.x2 = 0b1111_1111_1111_1110_1111_1100_1111_1000; // value to store

        let mut store_inst = SType::default();
        store_inst.rs1 = Register::X1;
        store_inst.rs2 = Register::X2;

        let sb = Instruction::SB(store_inst);
        let result = cpu.execute(sb);
        assert!(result.is_ok());

        let mut load_inst = IType::default();
        load_inst.rs1 = Register::X1; // base address
        load_inst.rd = Register::X3; // destination register

        // Use LBU for unsigned roundtrip since the value has sign bit set
        let lbu = Instruction::LBU(load_inst);
        let result = cpu.execute(lbu);
        assert!(result.is_ok());
        assert_eq!(cpu.x3, 0b1111_1000);
    }
}
