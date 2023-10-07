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
            AddSP(offset) => Ok(ALU::add_sp(cpu, *offset)),
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
            Da(reg) => Ok(ALU::da(cpu, reg)),
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
            ALU::check_h_carry_sub(value, 1, 0),
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
            GenericRegType::Double(reg) => {
                cpu.write_double_reg(reg, cpu.read_double_reg(reg).wrapping_sub(1))
            }
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
            ALU::check_h_carry_sum(value, 1, 0),
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
            GenericRegType::Double(reg) => {
                cpu.write_double_reg(reg, cpu.read_double_reg(reg).wrapping_add(1))
            }
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
        let left = cpu.read_single_reg(dst);
        let right = ALU::val_from_operand(src, cpu, bus)?;

        let carry = if with_carry {
            cpu.get_carry_flag() as u8
        } else {
            0
        };

        let result_ext = left as u16 + right as u16 + carry as u16;
        let result = result_ext as u8;

        cpu.set_flags(
            result == 0,
            false,
            ALU::check_h_carry_sum(left, right, carry),
            result_ext & 0xFF00 != 0,
        );

        cpu.write_single_reg(dst, result);
        Ok(())
    }

    fn add16(cpu: &mut CPU, dst: &DoubleRegType, src: &DoubleRegType) {
        let left = cpu.read_double_reg(dst);
        let right = cpu.read_double_reg(src);

        let result_ext = left as u32 + right as u32;
        let result = result_ext as u16;

        cpu.set_flags(
            cpu.get_zero_flag(),
            false,
            ALU::check_h_carry_sum16(left, right),
            result_ext & 0xFFFF0000 != 0,
        );

        cpu.write_double_reg(dst, result);
    }

    pub fn add_sp(cpu: &mut CPU, offset: i8) {
        let a = cpu.read_sp();
        let b = offset as i16 as u16; // signextend offset

        cpu.set_flags(
            false,
            false,
            (a & 0x000F) + (b & 0x000F) > 0x000F,
            (a & 0x00FF) + (b & 0x00FF) > 0x00FF,
        );

        cpu.write_sp(a.wrapping_add(b));
    }

    fn sub(
        cpu: &mut CPU,
        bus: &dyn BusAccess,
        dst: &SingleRegType,
        src: &Operand,
        with_carry: bool,
    ) -> Result<(), GbError> {
        let left = cpu.read_single_reg(dst);
        let right = ALU::val_from_operand(src, cpu, bus)?;

        let carry = if with_carry {
            cpu.get_carry_flag() as u8
        } else {
            0
        };

        let result = left.wrapping_sub(right).wrapping_sub(carry);

        cpu.set_flags(
            result == 0,
            true,
            ALU::check_h_carry_sub(left, right, carry),
            (left as i16 - right as i16 - carry as i16) < 0,
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

        cpu.set_flags(result == 0, false, false, false);

        cpu.write_to_reg_or_addr(bus, src, result)
    }

    // Impl from https://forums.nesdev.org/viewtopic.php?t=15944
    fn da(cpu: &mut CPU, src: &SingleRegType) {
        let mut val = cpu.read_single_reg(src);
        // BCD addition
        if !cpu.get_bcd_n_flag() {
            if cpu.get_carry_flag() || val > 0x99 {
                val += 0x60;
                cpu.set_carry_flag(true);
            }
            if cpu.get_bcd_h_flag() || ((val & 0x0F) > 0x09) {
                val += 0x06;
            }
        }
        // BCP subtraction
        else {
            if cpu.get_carry_flag() {
                val -= 0x60;
            }
            if cpu.get_bcd_h_flag() {
                val -= 0x06;
            }
        }

        cpu.write_single_reg(src, val);
        cpu.set_zero_flag(val == 0);
        cpu.set_bcd_n_flag(false);
    }

    fn cpl(cpu: &mut CPU, reg: &SingleRegType) {
        cpu.write_single_reg(reg, !cpu.read_single_reg(reg));

        cpu.set_bcd_n_flag(true);
        cpu.set_bcd_h_flag(true);
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

        let result = value.wrapping_shl(1) | cpu.get_carry_flag() as u8;

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

        let result = value.wrapping_shr(1) | ((cpu.get_carry_flag() as u8) << 7);

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

        // shift right and keep bit 7 unchanged
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
            ALU::check_h_carry_sub(left, right, 0),
            (left as i16 - right as i16) < 0,
        );

        Ok(())
    }

    fn check_h_carry_sum(lv: u8, rv: u8, carry: u8) -> bool {
        ((lv & 0x0F) + (rv & 0x0F) + (carry & 0x0F)) > 0x0F
    }

    fn check_h_carry_sum16(lv: u16, rv: u16) -> bool {
        ((lv & 0x0FFF) + (rv & 0x0FFF)) > 0x0FFF
    }

    fn check_h_carry_sub(lv: u8, rv: u8, carry: u8) -> bool {
        (lv & 0x0F)
            .wrapping_sub(rv & 0x0F)
            .wrapping_sub(carry & 0x0F)
            > 0x0F
    }
}

#[cfg(test)]
mod test {
    use mockall::predicate::eq;

    use super::*;

    use crate::gbr::instruction::{
        ArithmeticType::*, DoubleRegType::*, GenericRegType::*, SingleRegType::*,
    };
    use crate::gbr::{bus::MockBusAccess, cpu::CPU};

    struct AluTester {
        cpu: CPU,
        bus: MockBusAccess,
    }

    impl AluTester {
        fn new() -> Self {
            Self {
                cpu: CPU::new(),
                bus: MockBusAccess::new(),
            }
        }

        fn check_flags(&self, z: bool, n: bool, h: bool, c: bool) {
            assert_eq!(z, self.cpu.get_zero_flag(), "zero flag");
            assert_eq!(n, self.cpu.get_bcd_n_flag(), "bcd n flag");
            assert_eq!(h, self.cpu.get_bcd_h_flag(), "bcd h flag");
            assert_eq!(c, self.cpu.get_carry_flag(), "carry flag");
        }

        fn exec(&mut self, instr: ArithmeticType) {
            ALU::exec(&instr, &mut self.cpu, &mut self.bus).unwrap();
        }
    }

    #[test]
    fn check_h_carry_sum() {
        let left = 0b00001111_u8;
        let right = 0b00000001_u8;

        assert_eq!(ALU::check_h_carry_sum(left, right, 0), true);

        let left = 0b00000111_u8;
        let right = 0b0000001_u8;

        assert_eq!(ALU::check_h_carry_sum(left, right, 0), false);

        let left = 0b11110000_u8;
        let right = 0b00000001_u8;

        assert_eq!(ALU::check_h_carry_sum(left, right, 0), false);

        let left = 0b11111111_u8;
        let right = 0b00000001_u8;

        assert_eq!(ALU::check_h_carry_sum(left, right, 0), true);

        let left = 0x00;
        let right = 0x0F;
        let carry = 1;

        assert_eq!(ALU::check_h_carry_sum(left, right, carry), true);
    }

    #[test]
    fn check_h_carry_sub() {
        let left = 0b00000000_u8;
        let right = 0b00000001_u8;

        assert_eq!(ALU::check_h_carry_sub(left, right, 0), true);

        let left = 0b00001111_u8;
        let right = 0b00000001_u8;

        assert_eq!(ALU::check_h_carry_sub(left, right, 0), false);

        let left = 0;
        let right = 0;
        let carry = 1;

        assert_eq!(ALU::check_h_carry_sub(left, right, carry), true);
    }

    #[test]
    fn decrement_at_address() {
        const TEST_ADDR: u16 = 0x00FF;

        let mut tester = AluTester::new();
        tester.cpu.write_bc(TEST_ADDR);

        println!("Standard Test");
        tester.bus.expect_read_byte().return_once(|_addr| Ok(10));

        tester
            .bus
            .expect_write_byte()
            .with(eq(TEST_ADDR), eq(9))
            .return_once(|_addr, _val| Ok(()));

        tester.exec(DecAddr(BC));

        tester.check_flags(false, true, false, false);

        tester.bus.checkpoint();

        println!("Test that decrementing zero wraps around");
        tester.bus.expect_read_byte().return_once(|_addr| Ok(0));

        tester
            .bus
            .expect_write_byte()
            .with(eq(TEST_ADDR), eq(0xFF))
            .return_once(|_addr, _val| Ok(()));

        tester.exec(DecAddr(BC));

        tester.check_flags(false, true, true, false);
    }

    #[test]
    fn decrement_single_register() {
        let mut tester = AluTester::new();

        tester.cpu.write_single_reg(&A, 1);

        tester.exec(Dec(Single(A)));

        assert_eq!(tester.cpu.read_single_reg(&A), 0);

        tester.check_flags(true, true, false, false);

        tester.exec(Dec(Single(A)));

        assert_eq!(tester.cpu.read_single_reg(&A), 0xFF);

        tester.check_flags(false, true, true, false);
    }

    #[test]
    fn decrement_double_register() {
        let mut tester = AluTester::new();

        tester.cpu.write_bc(1);

        tester.exec(Dec(Double(BC)));

        assert_eq!(tester.cpu.read_bc(), 0);

        tester.check_flags(false, false, false, false);

        tester.exec(Dec(Double(BC)));

        assert_eq!(tester.cpu.read_bc(), 0xFFFF);
    }

    #[test]
    fn increment_at_address() {
        const TEST_ADDR: u16 = 0x00FF;

        let mut tester = AluTester::new();
        tester.cpu.write_bc(TEST_ADDR);

        println!("Test that incrementing sets half carry flag");
        tester.bus.expect_read_byte().return_once(|_addr| Ok(0x0F));

        tester
            .bus
            .expect_write_byte()
            .with(eq(TEST_ADDR), eq(0x10))
            .return_once(|_addr, _val| Ok(()));

        tester.exec(IncAddr(BC));

        tester.check_flags(false, false, true, false);

        tester.bus.checkpoint();

        println!("Test that incrementing with overflow sets zero flag");
        tester.bus.expect_read_byte().return_once(|_addr| Ok(0xFF));

        tester
            .bus
            .expect_write_byte()
            .with(eq(TEST_ADDR), eq(0x0))
            .return_once(|_addr, _val| Ok(()));

        tester.exec(IncAddr(BC));

        tester.check_flags(true, false, true, false);
    }

    #[test]
    fn increment_single_register() {
        let mut tester = AluTester::new();

        tester.cpu.write_single_reg(&A, 0x0F);

        tester.exec(Inc(Single(A)));

        assert_eq!(tester.cpu.read_single_reg(&A), 0x10);

        tester.check_flags(false, false, true, false);

        tester.cpu.write_single_reg(&A, 0xFF);

        tester.exec(Inc(Single(A)));

        assert_eq!(tester.cpu.read_single_reg(&SingleRegType::A), 0x00);

        tester.check_flags(true, false, true, false);
    }

    #[test]
    fn increment_double_register() {
        let mut tester = AluTester::new();

        tester.cpu.write_bc(0xFFFF);

        tester.exec(Inc(Double(BC)));

        assert_eq!(tester.cpu.read_bc(), 0);

        tester.check_flags(false, false, false, false);
    }

    #[test]
    fn addition() {
        let mut tester = AluTester::new();

        tester.cpu.write_single_reg(&A, 0x01);

        tester.exec(Add(A, Operand::Imm(0x0F)));

        assert_eq!(tester.cpu.read_single_reg(&A), 0x10);

        tester.check_flags(false, false, true, false);

        tester.cpu.write_single_reg(&B, 0xFF);

        tester.exec(Add(A, Operand::Reg(B)));

        assert_eq!(
            tester.cpu.read_single_reg(&A),
            0x10_u8.wrapping_add(0xFF_u8)
        );

        tester.check_flags(false, false, false, true);

        tester.cpu.write_single_reg(&C, 0xFF);
        tester.cpu.write_double_reg(&HL, 0x0FA);

        tester
            .bus
            .expect_read_byte()
            .with(eq(0x0FA))
            .return_once(|_| Ok(0x01));

        tester.exec(Add(C, Operand::RegAddr(HL)));

        assert_eq!(tester.cpu.read_single_reg(&C), 0x0);

        tester.check_flags(true, false, true, true);
    }

    #[test]
    fn addition_with_carry() {
        let mut tester = AluTester::new();

        tester.cpu.write_single_reg(&A, 0x00);
        tester.cpu.write_single_reg(&B, 0x0F);
        tester.cpu.set_carry_flag(true);

        tester.exec(Adc(A, Operand::Reg(B)));

        assert_eq!(tester.cpu.read_single_reg(&A), 0x10);

        tester.check_flags(false, false, true, false);
    }

    #[test]
    fn addition16() {
        let mut tester = AluTester::new();

        // Test that half carry flag is set for 16 bit addition
        tester.cpu.write_bc(0x0FFF);
        tester.cpu.write_hl(0x0001);

        tester.exec(Add16(BC, HL));

        assert_eq!(tester.cpu.read_bc(), 0x1000);

        tester.check_flags(false, false, true, false);

        // Test that adding one to 0xFFFF overflows and sets the carry flag
        tester.cpu.write_bc(0xFFFF);

        tester.exec(Add16(BC, HL));

        assert_eq!(tester.cpu.read_bc(), 0x0);

        tester.check_flags(false, false, true, true);
    }

    #[test]
    fn subtraction() {
        let mut tester = AluTester::new();

        // Test that if left < right subtraction wraps around and sets half
        // carry and carry flags
        tester.cpu.write_single_reg(&A, 0x01);

        tester.exec(Sub(A, Operand::Imm(0x0F)));

        assert_eq!(tester.cpu.read_single_reg(&A), 0x01_u8.wrapping_sub(0x0F));

        tester.check_flags(false, true, true, true);

        // Test that if left == right, subtraction sets zero flag
        tester.cpu.write_single_reg(&A, 0x0F);
        tester.cpu.write_single_reg(&B, 0x0F);

        tester.exec(Sub(A, Operand::Reg(B)));

        assert_eq!(tester.cpu.read_single_reg(&A), 0);

        tester.check_flags(true, true, false, false);

        tester.cpu.write_single_reg(&C, 0xFF);
        tester.cpu.write_double_reg(&HL, 0x0FA);

        tester
            .bus
            .expect_read_byte()
            .with(eq(0x0FA))
            .return_once(|_| Ok(0x01));

        tester.exec(Sub(C, Operand::RegAddr(HL)));

        assert_eq!(tester.cpu.read_single_reg(&C), 0xFE);

        tester.check_flags(false, true, false, false);
    }

    #[test]
    fn subtraction_with_carry() {
        let mut tester = AluTester::new();

        tester.cpu.write_single_reg(&A, 0x00);
        tester.cpu.write_single_reg(&B, 0x00);
        tester.cpu.set_carry_flag(true);

        tester.exec(Sbc(A, Operand::Reg(B)));

        assert_eq!(tester.cpu.read_single_reg(&A), 0xFF);

        tester.check_flags(false, true, true, true);
    }
    #[test]
    fn bitwise_and() {
        let mut tester = AluTester::new();

        tester.cpu.write_single_reg(&A, 0xF0);

        tester.exec(And(A, Operand::Imm(0x0F)));

        assert_eq!(tester.cpu.read_single_reg(&A), 0);

        tester.check_flags(true, false, true, false);

        tester.cpu.write_single_reg(&A, 0x0A);
        tester.cpu.write_single_reg(&H, 0x08);

        tester.exec(And(A, Operand::Reg(H)));

        assert_eq!(tester.cpu.read_single_reg(&A), 0x0A & 0x08);

        tester.check_flags(false, false, true, false);

        tester.cpu.write_single_reg(&A, 0x0A);
        tester.cpu.write_double_reg(&DE, 0xFF00);

        tester.bus.expect_read_byte().return_once(|_| Ok(0x01));

        tester.exec(And(A, Operand::RegAddr(DE)));

        assert_eq!(tester.cpu.read_single_reg(&A), 0x0A & 0x01);

        tester.check_flags(true, false, true, false);
    }

    #[test]
    fn bitwise_or() {
        let mut tester = AluTester::new();

        tester.cpu.write_single_reg(&A, 0xF0);

        tester.exec(Or(A, Operand::Imm(0x0F)));

        assert_eq!(tester.cpu.read_single_reg(&A), 0xFF);

        tester.check_flags(false, false, false, false);

        tester.cpu.write_single_reg(&A, 0x00);
        tester.cpu.write_single_reg(&H, 0x00);

        tester.exec(Or(A, Operand::Reg(H)));

        assert_eq!(tester.cpu.read_single_reg(&A), 0);

        tester.check_flags(true, false, false, false);

        tester.cpu.write_single_reg(&A, 0x0A);

        tester.bus.expect_read_byte().return_once(|_| Ok(0x01));

        tester.exec(Or(A, Operand::RegAddr(DE)));

        assert_eq!(tester.cpu.read_single_reg(&A), 0x0B);

        tester.check_flags(false, false, false, false);
    }

    #[test]
    fn bitwise_xor() {
        let mut tester = AluTester::new();

        tester.cpu.write_single_reg(&A, 0xF0);

        tester.exec(Xor(A, Operand::Imm(0x0F)));

        assert_eq!(tester.cpu.read_single_reg(&A), 0xFF);

        tester.check_flags(false, false, false, false);

        tester.cpu.write_single_reg(&A, 0x01);
        tester.cpu.write_single_reg(&H, 0x01);

        tester.exec(Xor(A, Operand::Reg(H)));

        assert_eq!(tester.cpu.read_single_reg(&A), 0);

        tester.check_flags(true, false, false, false);

        tester.cpu.write_single_reg(&A, 0x0A);

        tester.bus.expect_read_byte().return_once(|_| Ok(0x01));

        tester.exec(Xor(A, Operand::RegAddr(DE)));

        assert_eq!(tester.cpu.read_single_reg(&A), 0x0B);

        tester.check_flags(false, false, false, false);
    }

    #[test]
    fn bitwise_swap() {
        let mut tester = AluTester::new();

        tester.cpu.write_single_reg(&H, 0xF1);

        tester.exec(Swap(Single(H)));

        assert_eq!(tester.cpu.read_single_reg(&H), 0x1F);

        tester.check_flags(false, false, false, false);

        tester.cpu.write_de(0x0A0A);

        tester.bus.expect_read_byte().return_once(|_| Ok(0xAB));
        tester
            .bus
            .expect_write_byte()
            .with(eq(0x0A0A), eq(0xBA))
            .return_once(|_, _| Ok(()));

        tester.exec(Swap(Double(DE)));

        tester.check_flags(false, false, false, false);
    }

    #[test]
    fn complement() {
        let mut tester = AluTester::new();

        tester.cpu.write_single_reg(&B, 0x00);

        tester.exec(Cpl(B));

        assert_eq!(tester.cpu.read_single_reg(&B), 0xFF);

        tester.check_flags(false, true, true, false);
    }

    #[test]
    fn rotate_left_carry() {
        let mut tester = AluTester::new();

        tester.cpu.write_single_reg(&B, 0b01111111);

        tester.exec(Rlc(Single(B), true));

        assert_eq!(tester.cpu.read_single_reg(&B), 0b11111110);

        tester.check_flags(false, false, false, false);

        tester.cpu.write_single_reg(&B, 0b10000000);
        tester.exec(Rlc(Single(B), true));

        assert_eq!(tester.cpu.read_single_reg(&B), 1);

        tester.check_flags(false, false, false, true);

        tester.cpu.write_hl(0x00AA);
        tester.bus.expect_read_byte().return_once(|_| Ok(0));
        tester
            .bus
            .expect_write_byte()
            .with(eq(0x00AA), eq(0))
            .return_once(|_, _| Ok(()));

        tester.exec(Rlc(Double(HL), false));

        tester.check_flags(true, false, false, false);
    }

    #[test]
    fn rotate_left() {
        let mut tester = AluTester::new();

        tester.cpu.write_single_reg(&B, 0b01111111);

        tester.exec(Rl(Single(B), true));

        assert_eq!(tester.cpu.read_single_reg(&B), 0b11111110);

        tester.check_flags(false, false, false, false);

        tester.cpu.write_single_reg(&B, 0b10000000);
        tester.exec(Rl(Single(B), true));

        assert_eq!(tester.cpu.read_single_reg(&B), 0);

        tester.check_flags(false, false, false, true);

        tester.cpu.write_hl(0x00AA);
        tester.bus.expect_read_byte().return_once(|_| Ok(0));
        tester
            .bus
            .expect_write_byte()
            .with(eq(0x00AA), eq(1))
            .return_once(|_, _| Ok(()));

        tester.exec(Rl(Double(HL), false));

        tester.check_flags(false, false, false, false);
    }

    #[test]
    fn rotate_right_carry() {
        let mut tester = AluTester::new();

        tester.cpu.write_single_reg(&B, 0b00000010);

        tester.exec(Rrc(Single(B), true));

        assert_eq!(tester.cpu.read_single_reg(&B), 0b00000001);

        tester.check_flags(false, false, false, false);

        tester.exec(Rrc(Single(B), true));

        assert_eq!(tester.cpu.read_single_reg(&B), 0b10000000);

        tester.check_flags(false, false, false, true);

        tester.cpu.write_hl(0x00AA);
        tester.bus.expect_read_byte().return_once(|_| Ok(0));
        tester
            .bus
            .expect_write_byte()
            .with(eq(0x00AA), eq(0))
            .return_once(|_, _| Ok(()));

        tester.exec(Rrc(Double(HL), false));

        tester.check_flags(true, false, false, false);
    }

    #[test]
    fn rotate_right() {
        let mut tester = AluTester::new();

        tester.cpu.write_single_reg(&B, 0b00000010);

        tester.exec(Rr(Single(B), true));

        assert_eq!(tester.cpu.read_single_reg(&B), 0b00000001);

        tester.check_flags(false, false, false, false);

        tester.exec(Rr(Single(B), true));

        assert_eq!(tester.cpu.read_single_reg(&B), 0b00000000);

        tester.check_flags(false, false, false, true);

        tester.cpu.write_hl(0x00AA);
        tester.bus.expect_read_byte().return_once(|_| Ok(0));
        tester
            .bus
            .expect_write_byte()
            .with(eq(0x00AA), eq(0b10000000))
            .return_once(|_, _| Ok(()));

        tester.exec(Rr(Double(HL), false));

        tester.check_flags(false, false, false, false);
    }

    #[test]
    fn shift_left() {
        let mut tester = AluTester::new();

        tester.cpu.write_single_reg(&B, 0b01000000);

        tester.exec(Sla(Single(B)));

        assert_eq!(tester.cpu.read_single_reg(&B), 0b10000000);

        tester.check_flags(false, false, false, false);

        tester.exec(Sla(Single(B)));

        assert_eq!(tester.cpu.read_single_reg(&B), 0b00000000);

        tester.check_flags(true, false, false, true);

        tester.cpu.write_hl(0x00AA);
        tester.bus.expect_read_byte().return_once(|_| Ok(0));
        tester
            .bus
            .expect_write_byte()
            .with(eq(0x00AA), eq(0))
            .return_once(|_, _| Ok(()));

        tester.exec(Sla(Double(HL)));

        tester.check_flags(true, false, false, false);
    }

    #[test]
    fn shift_right() {
        let mut tester = AluTester::new();

        tester.cpu.write_single_reg(&B, 0b00000010);

        tester.exec(Srl(Single(B)));

        assert_eq!(tester.cpu.read_single_reg(&B), 0b00000001);

        tester.check_flags(false, false, false, false);

        tester.exec(Srl(Single(B)));

        assert_eq!(tester.cpu.read_single_reg(&B), 0b00000000);

        tester.check_flags(true, false, false, true);

        tester.cpu.write_hl(0x00AA);
        tester.bus.expect_read_byte().return_once(|_| Ok(0));
        tester
            .bus
            .expect_write_byte()
            .with(eq(0x00AA), eq(0))
            .return_once(|_, _| Ok(()));

        tester.exec(Srl(Double(HL)));

        tester.check_flags(true, false, false, false);
    }

    #[test]
    fn shift_right_arith() {
        let mut tester = AluTester::new();

        tester.cpu.write_single_reg(&B, 0b10000010);

        tester.exec(Sra(Single(B)));

        assert_eq!(tester.cpu.read_single_reg(&B), 0b11000001);

        tester.check_flags(false, false, false, false);

        tester.exec(Sra(Single(B)));

        assert_eq!(tester.cpu.read_single_reg(&B), 0b11100000);

        tester.check_flags(false, false, false, true);

        tester.cpu.write_hl(0x00AA);
        tester.bus.expect_read_byte().return_once(|_| Ok(0));
        tester
            .bus
            .expect_write_byte()
            .with(eq(0x00AA), eq(0))
            .return_once(|_, _| Ok(()));

        tester.exec(Srl(Double(HL)));

        tester.check_flags(true, false, false, false);
    }

    #[test]
    fn test_bit() {
        let mut tester = AluTester::new();

        tester.cpu.write_single_reg(&B, 0b01010101);
        for i in 0..8 {
            tester.exec(TestBit(Single(B), i));

            tester.check_flags(
                (tester.cpu.read_single_reg(&B) >> i) & 0b1 == 0,
                false,
                true,
                false,
            );
        }
    }

    #[test]
    fn set_bit() {
        let mut tester = AluTester::new();

        for i in 0..8 {
            tester.exec(SetBit(Single(B), i));

            assert_eq!((tester.cpu.read_single_reg(&B) >> i) & 0b1, 0b1);
        }

        assert_eq!(tester.cpu.read_single_reg(&B), 0xFF);
    }

    #[test]
    fn reset_bit() {
        let mut tester = AluTester::new();

        tester.cpu.write_single_reg(&B, 0xFF);
        for i in 0..8 {
            tester.exec(ResetBit(Single(B), i));

            assert_eq!((tester.cpu.read_single_reg(&B) >> i) & 0b1, 0b0);
        }

        assert_eq!(tester.cpu.read_single_reg(&B), 0x00);
    }

    #[test]
    fn compare() {
        let mut tester = AluTester::new();

        tester.cpu.write_single_reg(&A, 0xF);
        tester.exec(Cmp(A, Operand::Imm(0xF)));

        tester.check_flags(true, true, false, false);

        assert_eq!(tester.cpu.read_single_reg(&A), 0xF);
    }
}
