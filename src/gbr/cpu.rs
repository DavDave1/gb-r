use std::fmt;

use crate::gbr::alu::ALU;
use crate::gbr::GbError;

use super::{
    bus::BusAccess,
    instruction::{
        Dest, DoubleRegType, GenericRegType, InstructionType, JumpCondition, JumpType, PostLoad,
        SingleRegType, Source,
    },
    interrupts::InterruptType,
    memory_map::INTERRUPTS_ENABLE_REGISTER,
};

const VBLANK_IR_ADDRESS: u16 = 0x0040;
const LCD_STAT_IR_ADDRESS: u16 = 0x0048;
const TIMER_IR_ADDRESS: u16 = 0x0050;
const SERIAL_IR_ADDRESS: u16 = 0x0058;
const JOYPAD_IR_ADDRESS: u16 = 0x0060;

#[derive(Default, Clone)]
pub struct CpuState {
    pub af: u16,
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
    pub pc: u16,
    pub sp: u16,

    pub zero: bool,
    pub carry: bool,
    pub bcd_n: bool,
    pub bcd_h: bool,

    pub halted: bool,
}

impl fmt::Display for CpuState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Regsisters:\n
            AF {:#06X}, BC {:#06X}, DE {:#06X}, HL {:#06X}, PC {:#06X}, SP {:#06X}\n
            Flags:\n
            Z {}, C {}, BCD-N {}, BCD-H {}\n
            Halted: {}",
            self.af,
            self.bc,
            self.de,
            self.hl,
            self.pc,
            self.sp,
            self.zero,
            self.carry,
            self.bcd_n,
            self.bcd_h,
            self.halted
        )
    }
}

#[derive(Default)]
pub struct CPU {
    // 8bit general purpose registers
    reg_a: u8,
    reg_b: u8,
    reg_c: u8,
    reg_d: u8,
    reg_e: u8,
    reg_h: u8,
    reg_l: u8,
    reg_f: u8, //8bit flag register

    //16bit special purpose registers
    reg_pc: u16, // program counter
    reg_sp: u16, // stack pointer

    low_power_mode: bool,
    halted: bool,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            low_power_mode: false,
            halted: false,
            ..Default::default()
        }
    }

    pub fn read_af(&self) -> u16 {
        (self.reg_a as u16) << 8 | self.reg_f as u16
    }

    pub fn write_af(&mut self, value: u16) {
        self.reg_a = (value >> 8) as u8;
        self.reg_f = value as u8;
    }

    pub fn read_bc(&self) -> u16 {
        (self.reg_b as u16) << 8 | self.reg_c as u16
    }

    pub fn write_bc(&mut self, value: u16) {
        self.reg_b = (value >> 8) as u8;
        self.reg_c = value as u8;
    }

    pub fn read_de(&self) -> u16 {
        (self.reg_d as u16) << 8 | self.reg_e as u16
    }

    pub fn write_de(&mut self, value: u16) {
        self.reg_d = (value >> 8) as u8;
        self.reg_e = value as u8;
    }

    pub fn read_hl(&self) -> u16 {
        (self.reg_h as u16) << 8 | self.reg_l as u16
    }

    pub fn write_hl(&mut self, value: u16) {
        self.reg_h = (value >> 8) as u8;
        self.reg_l = value as u8;
    }

    pub fn read_single_reg(&self, reg: &SingleRegType) -> u8 {
        match reg {
            SingleRegType::A => self.reg_a,
            SingleRegType::B => self.reg_b,
            SingleRegType::C => self.reg_c,
            SingleRegType::D => self.reg_d,
            SingleRegType::E => self.reg_e,
            SingleRegType::H => self.reg_h,
            SingleRegType::L => self.reg_l,
        }
    }

    pub fn write_single_reg(&mut self, reg: &SingleRegType, value: u8) {
        match reg {
            SingleRegType::A => self.reg_a = value,
            SingleRegType::B => self.reg_b = value,
            SingleRegType::C => self.reg_c = value,
            SingleRegType::D => self.reg_d = value,
            SingleRegType::E => self.reg_e = value,
            SingleRegType::H => self.reg_h = value,
            SingleRegType::L => self.reg_l = value,
        }
    }

    pub fn read_double_reg(&self, reg: &DoubleRegType) -> u16 {
        match reg {
            DoubleRegType::AF => self.read_af(),
            DoubleRegType::BC => self.read_bc(),
            DoubleRegType::DE => self.read_de(),
            DoubleRegType::HL => self.read_hl(),
            DoubleRegType::SP => self.reg_sp,
        }
    }

    pub fn write_double_reg(&mut self, reg: &DoubleRegType, value: u16) {
        match reg {
            DoubleRegType::AF => self.write_af(value),
            DoubleRegType::BC => self.write_bc(value),
            DoubleRegType::DE => self.write_de(value),
            DoubleRegType::HL => self.write_hl(value),
            DoubleRegType::SP => self.reg_sp = value,
        }
    }

    pub fn read_pc(&self) -> u16 {
        self.reg_pc
    }

    pub fn read_sp(&self) -> u16 {
        self.reg_sp
    }

    pub fn write_sp(&mut self, value: u16) {
        self.reg_sp = value;
    }

    pub fn read_from_reg_or_addr(
        &self,
        bus: &dyn BusAccess,
        src: &GenericRegType,
    ) -> Result<u8, GbError> {
        let val = match src {
            GenericRegType::Single(reg) => self.read_single_reg(reg),
            GenericRegType::Double(reg_addr) => bus.read_byte(self.read_double_reg(reg_addr))?,
        };

        Ok(val)
    }

    pub fn write_to_reg_or_addr(
        &mut self,
        bus: &mut dyn BusAccess,
        src: &GenericRegType,
        value: u8,
    ) -> Result<(), GbError> {
        match src {
            GenericRegType::Single(reg) => self.write_single_reg(reg, value),
            GenericRegType::Double(reg_addr) => {
                bus.write_byte(self.read_double_reg(reg_addr), value)?
            }
        }

        Ok(())
    }

    pub fn get_zero_flag(&self) -> bool {
        self.reg_f & 0b10000000 != 0
    }

    pub fn set_zero_flag(&mut self, set: bool) {
        if set {
            self.reg_f = self.reg_f | 0b10000000
        } else {
            self.reg_f = self.reg_f & 0b01111111;
        }
    }

    pub fn get_carry_flag(&self) -> bool {
        self.reg_f & 0b00010000 != 0
    }

    pub fn set_carry_flag(&mut self, set: bool) {
        if set {
            self.reg_f = self.reg_f | 0b00010000
        } else {
            self.reg_f = self.reg_f & 0b11101111;
        }
    }

    pub fn get_bcd_n_flag(&self) -> bool {
        self.reg_f & 0b01000000 != 0
    }

    pub fn set_bcd_n_flag(&mut self, set: bool) {
        if set {
            self.reg_f = self.reg_f | 0b01000000
        } else {
            self.reg_f = self.reg_f & 0b10111111;
        }
    }

    pub fn get_bcd_h_flag(&self) -> bool {
        self.reg_f & 0b00100000 != 0
    }

    pub fn set_bcd_h_flag(&mut self, set: bool) {
        if set {
            self.reg_f = self.reg_f | 0b00100000
        } else {
            self.reg_f = self.reg_f & 0b11011111;
        }
    }

    pub fn set_flags(&mut self, z: bool, n: bool, h: bool, c: bool) {
        self.set_zero_flag(z);
        self.set_bcd_n_flag(n);
        self.set_bcd_h_flag(h);
        self.set_carry_flag(c);
    }

    fn push_stack(&mut self, bus: &mut dyn BusAccess, value: u16) -> Result<(), GbError> {
        bus.write_byte(self.reg_sp - 1, (value >> 8) as u8)?;
        bus.write_byte(self.reg_sp - 2, value as u8)?;
        self.reg_sp -= 2;
        Ok(())
    }

    fn pop_stack(&mut self, bus: &dyn BusAccess) -> Result<u16, GbError> {
        let value = bus.read_word(self.reg_sp)?;
        self.reg_sp += 2;
        Ok(value)
    }

    fn test_condition(&self, condition: &JumpCondition) -> bool {
        match condition {
            JumpCondition::Always => true,
            JumpCondition::Carry => self.get_carry_flag() == true,
            JumpCondition::NotCarry => self.get_carry_flag() == false,
            JumpCondition::Zero => self.get_zero_flag() == true,
            JumpCondition::NotZero => self.get_zero_flag() == false,
        }
    }

    fn jump(&mut self, condition: &JumpCondition, jump_type: &JumpType) -> bool {
        if self.test_condition(condition) {
            match jump_type {
                JumpType::Offset(offset) => {
                    if *offset < 0 {
                        self.reg_pc -= offset.abs() as u16;
                    } else {
                        self.reg_pc += *offset as u16;
                    }
                }
                JumpType::Addr(addr) => self.reg_pc = *addr,
                JumpType::RegAddr(reg) => self.reg_pc = self.read_double_reg(reg),
            }

            return true;
        }

        false
    }

    fn call(
        &mut self,
        bus: &mut dyn BusAccess,
        addr: u16,
        cond: &JumpCondition,
    ) -> Result<bool, GbError> {
        if self.test_condition(cond) {
            self.push_stack(bus, self.reg_pc)?;
            self.reg_pc = addr;

            return Ok(true);
        }

        Ok(false)
    }

    fn ret(&mut self, bus: &mut dyn BusAccess, cond: &JumpCondition) -> Result<bool, GbError> {
        if self.test_condition(cond) {
            self.reg_pc = self.pop_stack(bus)?;
            return Ok(true);
        }

        Ok(false)
    }

    fn load(
        &mut self,
        bus: &dyn BusAccess,
        reg: &GenericRegType,
        source: &Source,
    ) -> Result<(), GbError> {
        match reg {
            GenericRegType::Double(reg) => match source {
                Source::Imm16(val) => self.write_double_reg(reg, *val),
                Source::SpWithOffset(offset) => {
                    if *offset < 0 {
                        self.write_double_reg(reg, self.reg_sp - *offset as u16);
                    } else {
                        self.write_double_reg(reg, self.reg_sp + *offset as u16);
                    }
                }
                _ => return Err(GbError::IllegalOp("load u8 into double register".into())),
            },
            GenericRegType::Single(reg) => {
                let val = match source {
                    Source::Addr(addr) => bus.read_byte(*addr)?,
                    Source::Imm8(imm) => *imm,
                    Source::Imm16(_) | Source::SpWithOffset(_) => {
                        return Err(GbError::IllegalOp("load imm16 into 8bit register".into()))
                    }
                    Source::RegImm(src_reg) => self.read_single_reg(src_reg),
                    Source::RegAddr(src_reg) => bus.read_byte(self.read_double_reg(src_reg))?,
                    Source::IoPortImm(imm) => bus.read_byte(0xFF00 + *imm as u16)?,
                    Source::IoPortReg(src_reg) => {
                        bus.read_byte(0xFF00 + self.read_single_reg(src_reg) as u16)?
                    }
                };

                self.write_single_reg(reg, val);
            }
        }

        Ok(())
    }

    fn store(&mut self, bus: &mut dyn BusAccess, dest: &Dest, src: &Source) -> Result<(), GbError> {
        let addr = match dest {
            Dest::Addr(addr) => *addr,
            Dest::RegAddr(reg_addr) => self.read_double_reg(reg_addr),
            Dest::IoPort(offset) => 0xFF00 + *offset as u16,
            Dest::IoPortReg(reg_offset) => 0xFF00 + self.read_single_reg(reg_offset) as u16,
        };

        let val = match src {
            Source::Imm8(v) => *v,
            Source::Imm16(_) | Source::SpWithOffset(_) => {
                return Err(GbError::IllegalOp("store from imm16 source".into()))
            }
            Source::RegImm(reg) => self.read_single_reg(reg),
            Source::RegAddr(reg) => bus.read_byte(self.read_double_reg(reg))?,
            Source::Addr(addr) => bus.read_byte(*addr)?,
            Source::IoPortReg(reg) => bus.read_byte(0xFF00 + self.read_single_reg(reg) as u16)?,
            Source::IoPortImm(offs) => bus.read_byte(0xFF00 + *offs as u16)?,
        };

        bus.write_byte(addr, val)?;

        Ok(())
    }

    fn post_op(&mut self, post_op: &PostLoad) {
        match post_op {
            PostLoad::Inc => self.write_hl(self.read_hl() + 1),
            PostLoad::Dec => self.write_hl(self.read_hl() - 1),
        }
    }

    fn goto_interrupt(&mut self, bus: &mut dyn BusAccess, ir_addr: u16) -> Result<(), GbError> {
        bus.ir_handler_mut().set_ime(false);
        self.push_stack(bus, self.reg_pc)?;
        self.reg_pc = ir_addr;
        Ok(())
    }

    fn check_wakeup(&mut self, bus: &mut dyn BusAccess) -> Result<(), GbError> {
        let ir_handler = bus.ir_handler();

        if ir_handler.ime() && ir_handler.any_pending_interrupt() {
            self.halted = false;
        }
        Ok(())
    }

    fn check_interrupts(&mut self, bus: &mut dyn BusAccess) -> Result<bool, GbError> {
        let ir_handler = bus.ir_handler_mut();
        if ir_handler.ime() {
            if ir_handler.test(InterruptType::VBlank) {
                ir_handler.clear(InterruptType::VBlank);
                self.goto_interrupt(bus, VBLANK_IR_ADDRESS)?;
                log::debug!("Handling VBLANK interrupt");
                return Ok(true);
            }

            if ir_handler.test(InterruptType::LcdStat) {
                ir_handler.clear(InterruptType::LcdStat);
                self.goto_interrupt(bus, LCD_STAT_IR_ADDRESS)?;
                return Ok(true);
            }

            if ir_handler.test(InterruptType::Timer) {
                ir_handler.clear(InterruptType::Timer);
                self.goto_interrupt(bus, TIMER_IR_ADDRESS)?;
                return Ok(true);
            }

            if ir_handler.test(InterruptType::Serial) {
                ir_handler.clear(InterruptType::Serial);
                self.goto_interrupt(bus, SERIAL_IR_ADDRESS)?;
                return Ok(true);
            }

            if ir_handler.test(InterruptType::Joypad) {
                ir_handler.clear(InterruptType::Joypad);
                self.goto_interrupt(bus, JOYPAD_IR_ADDRESS)?;
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn halt(&mut self, bus: &mut dyn BusAccess) {
        let ir_handler = bus.ir_handler();

        if ir_handler.ime() {
            self.halted = true;
        } else if !ir_handler.any_pending_interrupt() {
            self.halted = true;
        } else {
            // TODO halt bug
            self.halted = false;
        }
    }
    pub fn step(&mut self, bus: &mut dyn BusAccess) -> Result<u8, GbError> {
        if self.halted {
            self.check_wakeup(bus)?;
            return Ok(1);
        }

        if self.check_interrupts(bus)? {
            return Ok(5);
        }

        let instr = bus.fetch_instruction(self.reg_pc)?;

        self.reg_pc += instr.len() as u16;

        let mut jumped = false;

        match instr.instr_type() {
            InstructionType::Nop => (),
            InstructionType::Stop => self.low_power_mode = true,
            InstructionType::Halt => self.halt(bus),
            InstructionType::FlipCarry => self.set_carry_flag(!self.get_carry_flag()),
            InstructionType::ClearCarry => self.set_carry_flag(false),
            InstructionType::MasterInterrupt(enable) => bus.ir_handler_mut().set_ime(*enable),
            InstructionType::Arithmetic(ar_type) => ALU::exec(ar_type, self, bus)?,
            InstructionType::Jump(condition, jump_type) => {
                jumped = self.jump(condition, jump_type);
            }
            InstructionType::Load(reg, source) => self.load(bus, reg, source)?,
            InstructionType::LoadWithOp(reg, source, post_load) => {
                self.load(bus, reg, source)?;
                self.post_op(post_load);
            }
            InstructionType::LoadSP(reg) => self.reg_sp = self.read_double_reg(reg),
            InstructionType::Store(dest, src) => self.store(bus, dest, src)?,
            InstructionType::StoreWithOp(dest, src, post_store) => {
                self.store(bus, dest, src)?;
                self.post_op(post_store);
            }
            InstructionType::StoreSP(addr) => {
                bus.write_byte(*addr, self.reg_sp as u8)?;
                bus.write_byte(*addr + 1, (self.reg_sp >> 8) as u8)?;
            }
            InstructionType::Push(reg_type) => {
                self.push_stack(bus, self.read_double_reg(reg_type))?
            }
            InstructionType::Pop(reg) => {
                let value = self.pop_stack(bus)?;
                self.write_double_reg(reg, value);
            }
            InstructionType::Call(addr, cond) => jumped = self.call(bus, *addr, cond)?,
            InstructionType::Ret(cond) => jumped = self.ret(bus, cond)?,
            InstructionType::RetI => {
                bus.ir_handler_mut().set_ime(true);
                jumped = self.ret(bus, &JumpCondition::Always)?;
            }
        }

        Ok(instr.cycles(jumped))
    }

    pub fn state(&self) -> CpuState {
        CpuState {
            af: self.read_af(),
            bc: self.read_bc(),
            de: self.read_de(),
            hl: self.read_hl(),
            pc: self.read_pc(),
            sp: self.read_sp(),
            zero: self.get_zero_flag(),
            carry: self.get_carry_flag(),
            bcd_h: self.get_bcd_h_flag(),
            bcd_n: self.get_bcd_n_flag(),
            halted: self.halted,
        }
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;

    use crate::gbr::{
        bus::MockBusAccess,
        instruction::{opcode::Opcode, GenericRegType::*, Instruction, SingleRegType::*, Source},
        interrupts::InterruptHandler,
    };

    use super::CPU;

    struct CpuTester {
        cpu: CPU,
        bus: MockBusAccess,
    }

    impl CpuTester {
        fn new() -> Self {
            let mut bus = MockBusAccess::new();

            bus.expect_ir_handler()
                .return_const(InterruptHandler::default());

            bus.expect_ir_handler_mut()
                .return_var(InterruptHandler::default());

            Self {
                cpu: CPU::new(),
                bus: MockBusAccess::new(),
            }
        }

        fn exec(&mut self, opcode: Opcode, data: &[u8]) -> u8 {
            let mut instr_data = vec![0; data.len() + 1];
            instr_data[0] = opcode as u8;
            instr_data[1..].copy_from_slice(data);

            let instr = Instruction::decode(&instr_data).unwrap();
            self.bus
                .expect_fetch_instruction()
                .return_once(|_| Ok(instr));

            for d in data {
                let v = *d;
                self.bus.expect_read_byte().return_once(move |_| Ok(v));
            }

            self.cpu.step(&mut self.bus).unwrap()
        }
    }

    #[test]
    fn stop() {
        let mut tester = CpuTester::new();

        let cycles = tester.exec(Opcode::Stop, &[0; 0]);

        assert_eq!(cycles, 1);
        assert_eq!(tester.cpu.low_power_mode, true);
    }

    #[test]
    fn load_byte_imm() {
        let mut tester = CpuTester::new();

        let cycles = tester.exec(Opcode::LdAd8, &[0xAB]);

        assert_eq!(cycles, 2);
        assert_eq!(tester.cpu.read_single_reg(&A), 0xAB);
    }

    #[test]
    fn load_word_imm() {
        let mut tester = CpuTester::new();

        let cycles = tester.exec(Opcode::LdHLd16, &[0xAB, 0xBA]);

        assert_eq!(cycles, 3);
        assert_eq!(tester.cpu.read_hl(), 0xBAAB);
    }

    #[test]
    fn load_byte_from_addr() {
        let mut tester = CpuTester::new();

        tester
            .bus
            .expect_read_byte()
            .with(eq(0xBBAA))
            .return_once(|_| Ok(0x12));

        let cycles = tester.exec(Opcode::LdAa16, &[0xAA, 0xBB]);

        assert_eq!(cycles, 4);
        assert_eq!(tester.cpu.read_single_reg(&A), 0x12);
    }

    #[test]
    fn copy_single_reg() {
        let mut tester = CpuTester::new();

        let single_regs = [A, B, C, D, E, H, L];

        for src in &single_regs {
            tester.cpu.write_single_reg(src, 0xA);

            for dst in &single_regs {
                if dst != src {
                    tester.cpu.write_single_reg(dst, 0x00);
                }

                tester
                    .cpu
                    .load(&mut tester.bus, &Single(*dst), &Source::RegImm(*src))
                    .unwrap();

                assert_eq!(
                    tester.cpu.read_single_reg(dst),
                    0xA,
                    "Reg {} = {}",
                    dst,
                    src
                );
            }
        }
    }
}
