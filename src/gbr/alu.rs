use crate::gbr::{cpu::CPU, instruction::CompareType};

use super::instruction::ArithmeticType;

#[derive(Default)]
pub struct ALU;

impl ALU {
    pub fn exec(cpu: &mut CPU, op: &ArithmeticType) {
        match op {
            ArithmeticType::Add(dst, src) => {
                let res = ALU::add(cpu, cpu.read_single_reg(dst), cpu.read_single_reg(src));
                cpu.write_single_reg(dst, res);
            }
            ArithmeticType::Sub(dst, src) => {
                let res = ALU::sub(cpu, cpu.read_single_reg(dst), cpu.read_single_reg(src));
                cpu.write_single_reg(dst, res);
            }
            ArithmeticType::Inc8(dst) => {
                let res = ALU::inc(cpu, cpu.read_single_reg(dst));
                cpu.write_single_reg(dst, res);
            }
            ArithmeticType::Inc16(dst) => cpu.write_double_reg(dst, cpu.read_double_reg(dst) + 1),
            ArithmeticType::Dec(dst) => {
                let res = ALU::dec(cpu, cpu.read_single_reg(dst));
                cpu.write_single_reg(dst, res);
            }
            ArithmeticType::Cmp(dst, com_type) => {
                let cmp_val = match com_type {
                    CompareType::Imm(v) => *v,
                    CompareType::Reg(src) => cpu.read_single_reg(src),
                };

                ALU::cp(cpu, cpu.read_single_reg(dst), cmp_val);
            }
            ArithmeticType::Rlc(reg) => {
                let res = ALU::rlc(cpu, cpu.read_single_reg(reg));
                cpu.write_single_reg(reg, res);
            }
            ArithmeticType::Rl(reg) => {
                let res = ALU::rlc(cpu, cpu.read_single_reg(reg));
                cpu.write_single_reg(reg, res);
                cpu.set_zero_flag(false); // investigate: why this special case?
            }
            ArithmeticType::Sla(reg) => {
                let res = ALU::sla(cpu, cpu.read_single_reg(reg));
                cpu.write_single_reg(reg, res);
            }
            ArithmeticType::TestBit(reg, bit) => {
                ALU::test_bit(cpu, cpu.read_single_reg(reg), *bit);
            }
            ArithmeticType::Xor(dst, src) => {
                let res = ALU::xor(cpu, cpu.read_single_reg(dst), cpu.read_single_reg(src));
                cpu.write_single_reg(dst, res);
            }
        }
    }

    pub fn dec(cpu: &mut CPU, value: u8) -> u8 {
        let result = value.wrapping_sub(1);

        cpu.set_bcd_h_flag(value == 0x10);
        cpu.set_bcd_n_flag(true);
        cpu.set_zero_flag(result == 0);

        result
    }

    pub fn inc(cpu: &mut CPU, value: u8) -> u8 {
        let result = value.wrapping_add(1);

        cpu.set_bcd_h_flag(value == 0x0F);
        cpu.set_bcd_n_flag(false);
        cpu.set_zero_flag(result == 0);

        result
    }

    pub fn add(cpu: &mut CPU, left: u8, right: u8) -> u8 {
        let result_ext = left as u16 + right as u16;
        let result = result_ext as u8;

        cpu.set_bcd_h_flag(result & 0xF0 > left & 0xF0);
        cpu.set_bcd_n_flag(false);
        cpu.set_carry_flag(result_ext & 0xFF00 != 0);
        cpu.set_zero_flag(result == 0);

        result
    }

    pub fn sub(cpu: &mut CPU, left: u8, right: u8) -> u8 {
        let result = left.wrapping_sub(right);

        cpu.set_bcd_h_flag(result & 0xF0 < left & 0xF0);
        cpu.set_bcd_n_flag(true);
        cpu.set_carry_flag(result > left);
        cpu.set_zero_flag(result == 0);

        result
    }

    pub fn xor(cpu: &mut CPU, left: u8, right: u8) -> u8 {
        let result = left ^ right;

        cpu.set_bcd_h_flag(false);
        cpu.set_bcd_n_flag(true);
        cpu.set_carry_flag(false);
        cpu.set_zero_flag(result == 0);
        result
    }

    pub fn rlc(cpu: &mut CPU, value: u8) -> u8 {
        let carry_in = cpu.get_carry_flag();
        let carry_out = value & 0b10000000 != 0;

        let mut result = value.wrapping_shl(1);

        if carry_in {
            result = result | 0b00000001;
        }

        cpu.set_bcd_h_flag(false);
        cpu.set_bcd_n_flag(false);
        cpu.set_carry_flag(carry_out);
        cpu.set_zero_flag(result == 0);

        result
    }

    pub fn sla(cpu: &mut CPU, value: u8) -> u8 {
        let ext_result = (value as u16) << 1;
        let result = ext_result as u8;

        cpu.set_bcd_h_flag(false);
        cpu.set_bcd_n_flag(false);
        cpu.set_carry_flag(ext_result & 0xFF00 != 0);
        cpu.set_zero_flag(result == 0);

        result
    }

    pub fn test_bit(cpu: &mut CPU, value: u8, bit: u8) {
        let mask = 0b1_u8 << bit;

        cpu.set_bcd_h_flag(true);
        cpu.set_bcd_n_flag(false);
        cpu.set_zero_flag(value & mask == 0);
    }

    pub fn cp(cpu: &mut CPU, left: u8, right: u8) {
        ALU::sub(cpu, left, right);
    }
}
