use super::{
    bus::BusAccess,
    cpu::CPU,
    instruction::{ArithmeticType, DoubleRegType, GenericRegType, Operand, SingleRegType},
    GbError,
};

#[derive(Default)]
pub struct ALU;

impl ALU {
    pub fn exec(
        op: &ArithmeticType,
        cpu: &mut CPU,
        bus: &mut dyn BusAccess,
    ) -> Result<(), GbError> {
        use ArithmeticType::*;

        match op {
            Add(dst, src) => ALU::add(cpu, bus, dst, src, false),
            Add16(dst, src) => Ok(ALU::add16(cpu, dst, src)),
            AddSP(offset) => {
                let mut val = cpu.read_sp();
                if *offset > 0 {
                    val += *offset as u16;
                } else {
                    val -= offset.abs() as u16;
                }

                cpu.write_sp(val);
                Ok(())
            }
            Adc(dst, src) => ALU::add(cpu, bus, dst, src, true),
            Sub(dst, src) => ALU::sub(cpu, bus, dst, src, false),
            Sbc(dst, src) => ALU::sub(cpu, bus, dst, src, true),
            Inc(dst) => Ok(ALU::inc(cpu, dst)),
            IncAddr(src) => ALU::inc_addr(cpu, bus, src),
            DecAddr(src) => ALU::dec_addr(cpu, bus, src),
            Dec(dst) => Ok(ALU::dec(cpu, dst)),
            Cmp(dst, src) => ALU::cp(cpu, bus, dst, src),
            Rlc(reg, clear_z_flag) => ALU::rlc(cpu, bus, reg, *clear_z_flag),
            Rl(reg, clear_z_flag) => ALU::rl(cpu, bus, reg, *clear_z_flag),
            Rrc(reg, clear_z_flag) => ALU::rrc(cpu, bus, reg, *clear_z_flag),
            Rr(reg, clear_z_flag) => ALU::rr(cpu, bus, reg, *clear_z_flag),
            Sla(reg) => ALU::sla(cpu, bus, reg),
            Sra(reg) => ALU::sra(cpu, bus, reg),
            Srl(reg) => ALU::srl(cpu, bus, reg),
            TestBit(src, bit) => ALU::test_bit(cpu, bus, src, *bit),
            ResetBit(src, bit) => ALU::reset_bit(cpu, bus, src, *bit),
            SetBit(src, bit) => ALU::set_bit(cpu, bus, src, *bit),
            And(dst, src) => ALU::and(cpu, bus, dst, src),
            Or(dst, src) => ALU::or(cpu, bus, dst, src),
            Xor(dst, src) => ALU::xor(cpu, bus, dst, src),
            Cpl(reg) => Ok(ALU::cpl(cpu, reg)),
            Swap(reg) => ALU::swap(cpu, bus, reg),
        }
    }

    fn val_from_operand(
        operand_type: &Operand,
        cpu: &CPU,
        bus: &dyn BusAccess,
    ) -> Result<u8, GbError> {
        let cmp_val = match operand_type {
            Operand::Imm(v) => *v,
            Operand::Reg(src) => cpu.read_single_reg(src),
            Operand::RegAddr(src) => bus.read_byte(cpu.read_double_reg(src))?,
        };

        Ok(cmp_val)
    }

    fn dec8(cpu: &mut CPU, value: u8) -> u8 {
        let result = value.wrapping_sub(1);

        cpu.set_flags(
            result == 0,
            true,
            ALU::check_h_carry_sub(value as i16, -1),
            cpu.get_carry_flag(),
        );

        result
    }

    fn dec(cpu: &mut CPU, reg: &GenericRegType) {
        match reg {
            GenericRegType::Single(reg) => {
                let value = cpu.read_single_reg(reg);
                let result = ALU::dec8(cpu, value);

                cpu.write_single_reg(reg, result);
            }
            GenericRegType::Double(reg) => cpu.write_double_reg(reg, cpu.read_double_reg(reg) - 1),
        }
    }

    fn dec_addr(
        cpu: &mut CPU,
        bus: &mut dyn BusAccess,
        reg: &DoubleRegType,
    ) -> Result<(), GbError> {
        let addr = cpu.read_double_reg(reg);
        let val = bus.read_byte(addr)?;

        let result = ALU::dec8(cpu, val);

        bus.write_byte(addr, result)
    }

    fn inc8(cpu: &mut CPU, value: u8) -> u8 {
        let result = value.wrapping_add(1);

        cpu.set_flags(
            result == 0,
            false,
            ALU::check_h_carry_sum(value as u16, 1),
            cpu.get_carry_flag(),
        );

        result
    }

    fn inc(cpu: &mut CPU, reg: &GenericRegType) {
        match reg {
            GenericRegType::Single(reg) => {
                let value = cpu.read_single_reg(reg);

                let result = ALU::inc8(cpu, value);

                cpu.write_single_reg(reg, result);
            }
            GenericRegType::Double(reg) => cpu.write_double_reg(reg, cpu.read_double_reg(reg) + 1),
        }
    }

    fn inc_addr(
        cpu: &mut CPU,
        bus: &mut dyn BusAccess,
        reg: &DoubleRegType,
    ) -> Result<(), GbError> {
        let addr = cpu.read_double_reg(reg);
        let val = bus.read_byte(addr)?;

        let result = ALU::inc8(cpu, val);

        bus.write_byte(addr, result)
    }

    fn add(
        cpu: &mut CPU,
        bus: &dyn BusAccess,
        dst: &SingleRegType,
        src: &Operand,
        with_carry: bool,
    ) -> Result<(), GbError> {
        let left = cpu.read_single_reg(dst) as u16;
        let mut right = ALU::val_from_operand(src, cpu, bus)? as u16;

        if with_carry {
            right += cpu.get_carry_flag() as u16;
        }

        let result_ext = left + right;
        let result = result_ext as u8;

        cpu.set_flags(
            result == 0,
            false,
            ALU::check_h_carry_sum(left, right),
            result_ext & 0xFF00 != 0,
        );

        cpu.write_single_reg(dst, result);
        Ok(())
    }

    fn add16(cpu: &mut CPU, dst: &DoubleRegType, src: &DoubleRegType) {
        let left = cpu.read_double_reg(dst) as u32;
        let right = cpu.read_double_reg(src) as u32;

        let result_ext = left + right;
        let result = result_ext as u16;

        cpu.set_flags(
            cpu.get_zero_flag(),
            false,
            ALU::check_h_carry_sum16(left, right),
            result_ext & 0xFFFF0000 != 0,
        );

        cpu.write_double_reg(dst, result);
    }

    fn sub(
        cpu: &mut CPU,
        bus: &dyn BusAccess,
        dst: &SingleRegType,
        src: &Operand,
        with_carry: bool,
    ) -> Result<(), GbError> {
        let left = cpu.read_single_reg(dst);
        let mut right = ALU::val_from_operand(src, cpu, bus)?;

        if with_carry {
            right += cpu.get_carry_flag() as u8;
        }

        let result = left.wrapping_sub(right);

        cpu.set_flags(
            result == 0,
            true,
            ALU::check_h_carry_sub(left as i16, right as i16),
            (left as i16 - right as i16) < 0,
        );

        cpu.write_single_reg(dst, result);
        Ok(())
    }

    fn and(
        cpu: &mut CPU,
        bus: &dyn BusAccess,
        left: &SingleRegType,
        right: &Operand,
    ) -> Result<(), GbError> {
        let result = cpu.read_single_reg(left) & ALU::val_from_operand(right, cpu, bus)?;

        cpu.set_flags(result == 0, false, true, false);

        cpu.write_single_reg(left, result);

        Ok(())
    }

    fn or(
        cpu: &mut CPU,
        bus: &dyn BusAccess,
        left: &SingleRegType,
        right: &Operand,
    ) -> Result<(), GbError> {
        let result = cpu.read_single_reg(left) | ALU::val_from_operand(right, cpu, bus)?;

        cpu.set_flags(result == 0, false, false, false);

        cpu.write_single_reg(left, result);

        Ok(())
    }

    fn xor(
        cpu: &mut CPU,
        bus: &dyn BusAccess,
        left: &SingleRegType,
        right: &Operand,
    ) -> Result<(), GbError> {
        let result = cpu.read_single_reg(left) ^ ALU::val_from_operand(right, cpu, bus)?;

        cpu.set_flags(result == 0, false, false, false);

        cpu.write_single_reg(left, result);

        Ok(())
    }

    fn swap(cpu: &mut CPU, bus: &mut dyn BusAccess, src: &GenericRegType) -> Result<(), GbError> {
        let value = cpu.read_from_reg_or_addr(bus, src)?;

        let high = value & 0b11110000;

        let result = value << 4 | high >> 4;

        cpu.set_zero_flag(result == 0);

        cpu.write_to_reg_or_addr(bus, src, result)
    }

    fn cpl(cpu: &mut CPU, reg: &SingleRegType) {
        cpu.set_bcd_n_flag(true);
        cpu.set_bcd_h_flag(true);

        cpu.write_single_reg(reg, !cpu.read_single_reg(reg));
    }

    fn rlc(
        cpu: &mut CPU,
        bus: &mut dyn BusAccess,
        src: &GenericRegType,
        clear_z_flag: bool,
    ) -> Result<(), GbError> {
        let value = cpu.read_from_reg_or_addr(bus, src)?;

        let carry_out = (value & 0b10000000) != 0;

        let result = value.rotate_left(1);

        let z_flag = if clear_z_flag { false } else { result == 0 };

        cpu.set_flags(z_flag, false, false, carry_out);

        cpu.write_to_reg_or_addr(bus, src, result)
    }

    fn rl(
        cpu: &mut CPU,
        bus: &mut dyn BusAccess,
        src: &GenericRegType,
        clear_z_flag: bool,
    ) -> Result<(), GbError> {
        let value = cpu.read_from_reg_or_addr(bus, src)?;

        let carry_out = (value & 0b10000000) != 0;

        let mut result = value.wrapping_shl(1);

        result = result | cpu.get_carry_flag() as u8;

        let z_flag = if clear_z_flag { false } else { result == 0 };

        cpu.set_flags(z_flag, false, false, carry_out);

        cpu.write_to_reg_or_addr(bus, src, result)
    }

    fn rrc(
        cpu: &mut CPU,
        bus: &mut dyn BusAccess,
        src: &GenericRegType,
        clear_z_flag: bool,
    ) -> Result<(), GbError> {
        let value = cpu.read_from_reg_or_addr(bus, src)?;

        let carry_out = (value & 0b00000001) != 0;

        let result = value.rotate_right(1);

        let z_flag = if clear_z_flag { false } else { result == 0 };

        cpu.set_flags(z_flag, false, false, carry_out);

        cpu.write_to_reg_or_addr(bus, src, result)
    }

    fn rr(
        cpu: &mut CPU,
        bus: &mut dyn BusAccess,
        src: &GenericRegType,
        clear_z_flag: bool,
    ) -> Result<(), GbError> {
        let value = cpu.read_from_reg_or_addr(bus, src)?;

        let carry_out = (value & 0b00000001) != 0;

        let mut result = value.wrapping_shr(1);

        result = result | cpu.get_carry_flag() as u8;

        let z_flag = if clear_z_flag { false } else { result == 0 };

        cpu.set_flags(z_flag, false, false, carry_out);

        cpu.write_to_reg_or_addr(bus, src, result)
    }

    fn sla(cpu: &mut CPU, bus: &mut dyn BusAccess, src: &GenericRegType) -> Result<(), GbError> {
        let value = cpu.read_from_reg_or_addr(bus, src)?;

        let will_carry = value & 0b10000000 != 0;

        let result = value.wrapping_shl(1);

        cpu.set_flags(result == 0, false, false, will_carry);

        cpu.write_to_reg_or_addr(bus, src, result)
    }

    fn srl(cpu: &mut CPU, bus: &mut dyn BusAccess, src: &GenericRegType) -> Result<(), GbError> {
        let value = cpu.read_from_reg_or_addr(bus, src)?;

        let will_carry = value & 0b00000001 != 0;

        let result = value.wrapping_shr(1);

        cpu.set_flags(result == 0, false, false, will_carry);

        cpu.write_to_reg_or_addr(bus, src, result)
    }

    fn sra(cpu: &mut CPU, bus: &mut dyn BusAccess, src: &GenericRegType) -> Result<(), GbError> {
        let value = cpu.read_from_reg_or_addr(bus, src)?;

        let will_carry = value & 0b00000001 != 0;

        // shift right and keep but 7 or reg unchanged
        let result = value.wrapping_shr(1) | value & 0b10000000;

        cpu.set_flags(result == 0, false, false, will_carry);

        cpu.write_to_reg_or_addr(bus, src, result)
    }

    fn test_bit(
        cpu: &mut CPU,
        bus: &dyn BusAccess,
        src: &GenericRegType,
        bit: u8,
    ) -> Result<(), GbError> {
        let value = cpu.read_from_reg_or_addr(bus, src)?;

        let mask = 0b1_u8 << bit;

        cpu.set_flags(value & mask == 0, false, true, cpu.get_carry_flag());

        Ok(())
    }

    fn reset_bit(
        cpu: &mut CPU,
        bus: &mut dyn BusAccess,
        src: &GenericRegType,
        bit: u8,
    ) -> Result<(), GbError> {
        let value = cpu.read_from_reg_or_addr(bus, src)?;

        let mask = !(0b1 << bit);

        cpu.write_to_reg_or_addr(bus, src, value & mask)
    }

    fn set_bit(
        cpu: &mut CPU,
        bus: &mut dyn BusAccess,
        src: &GenericRegType,
        bit: u8,
    ) -> Result<(), GbError> {
        let value = cpu.read_from_reg_or_addr(bus, src)?;

        let mask = 0b1 << bit;

        cpu.write_to_reg_or_addr(bus, src, value | mask)
    }

    fn cp(
        cpu: &mut CPU,
        bus: &dyn BusAccess,
        dst: &SingleRegType,
        src: &Operand,
    ) -> Result<(), GbError> {
        let left = cpu.read_single_reg(dst);
        let right = ALU::val_from_operand(src, cpu, bus)?;

        let result = left.wrapping_sub(right);

        cpu.set_flags(
            result == 0,
            true,
            ALU::check_h_carry_sub(left as i16, right as i16),
            (left as i16 - right as i16) < 0,
        );

        Ok(())
    }

    fn check_h_carry_sum(lv: u16, rv: u16) -> bool {
        ((lv & 0xFF) + (rv & 0xFF)) > 0xF
    }

    fn check_h_carry_sum16(lv: u32, rv: u32) -> bool {
        ((lv & 0xFFFF) + (rv & 0xFFFF)) > 0xFF
    }

    fn check_h_carry_sub(lv: i16, rv: i16) -> bool {
        ((lv & 0xFF) - (rv & 0xFF)) < 0
    }
}
