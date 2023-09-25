use std::fmt;

use crate::gbr::alu::ALU;
use crate::gbr::bus::Bus;
use crate::gbr::GbError;

use super::instruction::{
    CallMode, DestType, DoubleRegType, GenericRegType, InstructionType, JumpCondition, JumpType,
    PostStore, SingleRegType, SourceType,
};

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
}

impl fmt::Display for CpuState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Regsisters:\n
            AF {:#06X}, BC {:#06X}, DE {:#06X}, HL {:#06X}, PC {:#06X}, SP {:#06X}\n
            Flags:\n
            Z {}, C {}, BCD-N {}, BCD-H {}",
            self.af,
            self.bc,
            self.de,
            self.hl,
            self.pc,
            self.sp,
            self.zero,
            self.carry,
            self.bcd_n,
            self.bcd_h
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
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            low_power_mode: false,
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
            SingleRegType::F => self.reg_f,
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
            SingleRegType::F => self.reg_f = value,
            SingleRegType::H => self.reg_h = value,
            SingleRegType::L => self.reg_l = value,
        }
    }

    pub fn read_double_reg(&self, reg: &DoubleRegType) -> u16 {
        match reg {
            DoubleRegType::BC => self.read_bc(),
            DoubleRegType::DE => self.read_de(),
            DoubleRegType::HL => self.read_hl(),
            DoubleRegType::SP => self.reg_sp,
        }
    }

    pub fn write_double_reg(&mut self, reg: &DoubleRegType, value: u16) {
        match reg {
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

    fn push_stack(&mut self, bus: &mut Bus, value: u16) -> Result<(), GbError> {
        bus.write_byte(self.reg_sp - 1, (value >> 8) as u8)?;
        bus.write_byte(self.reg_sp - 2, value as u8)?;
        self.reg_sp -= 2;
        Ok(())
    }

    fn pop_stack(&mut self, bus: &mut Bus) -> Result<u16, GbError> {
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
            match *jump_type {
                JumpType::Relative(offset) => {
                    if offset < 0 {
                        self.reg_pc -= offset.abs() as u16;
                    } else {
                        self.reg_pc += offset as u16;
                    }
                }
                JumpType::Absolute(addr) => self.reg_pc = addr,
            }

            return true;
        }

        false
    }

    fn load(
        &mut self,
        bus: &Bus,
        reg: &GenericRegType,
        source: &SourceType,
    ) -> Result<(), GbError> {
        match reg {
            GenericRegType::Double(reg) => match source {
                SourceType::Imm16(val) => self.write_double_reg(reg, *val),
                _ => return Err(GbError::IllegalOp("load u8 into double register".into())),
            },
            GenericRegType::Single(reg) => {
                let val = match source {
                    SourceType::Addr(addr) => bus.read_byte(*addr)?,
                    SourceType::Imm8(imm) => *imm,
                    SourceType::Imm16(_) => {
                        return Err(GbError::IllegalOp("load imm16 into 8bit register".into()))
                    }
                    SourceType::RegImm(src_reg) => self.read_single_reg(src_reg),
                    SourceType::RegAddr(src_reg) => bus.read_byte(self.read_double_reg(src_reg))?,
                    SourceType::IoPortImm(imm) => bus.read_byte(0xFF00 + *imm as u16)?,
                    SourceType::IoPortReg(src_reg) => {
                        bus.read_byte(0xFF00 + self.read_single_reg(src_reg) as u16)?
                    }
                };

                self.write_single_reg(reg, val);
            }
        }
        Ok(())
    }

    fn store(
        &mut self,
        bus: &mut Bus,
        dest: &DestType,
        src: &SourceType,
        post_store: &PostStore,
    ) -> Result<(), GbError> {
        let addr = match dest {
            DestType::Addr(addr) => *addr,
            DestType::RegAddr(reg_addr) => self.read_double_reg(reg_addr),
            DestType::IoPort(offset) => 0xFF00 + *offset as u16,
            DestType::IoPortReg(reg_offset) => 0xFF00 + self.read_single_reg(reg_offset) as u16,
        };

        let val = match src {
            SourceType::Imm8(v) => *v,
            SourceType::Imm16(_) => {
                return Err(GbError::IllegalOp("store from imm16 source".into()))
            }
            SourceType::RegImm(reg) => self.read_single_reg(reg),
            SourceType::RegAddr(reg) => bus.read_byte(self.read_double_reg(reg))?,
            SourceType::Addr(addr) => bus.read_byte(*addr)?,
            SourceType::IoPortReg(reg) => {
                bus.read_byte(0xFF00 + self.read_single_reg(reg) as u16)?
            }
            SourceType::IoPortImm(offs) => bus.read_byte(0xFF00 + *offs as u16)?,
        };

        bus.write_byte(addr, val)?;

        match post_store {
            PostStore::Inc => self.write_hl(self.read_hl() + 1),
            PostStore::Dec => self.write_hl(self.read_hl() - 1),
            PostStore::None => (),
        }

        Ok(())
    }

    fn call(&mut self, bus: &mut Bus, call_mode: &CallMode) -> Result<(), GbError> {
        self.push_stack(bus, self.reg_pc)?;

        match call_mode {
            CallMode::Absolute(addr) => self.reg_pc = *addr,
        }

        Ok(())
    }

    pub fn step(&mut self, bus: &mut Bus) -> Result<u8, GbError> {
        let instr = bus.fetch_instruction(self.reg_pc)?;

        self.reg_pc += instr.len() as u16;

        let mut jumped = false;

        match instr.instr_type() {
            InstructionType::Nop => (),
            InstructionType::Stop => self.low_power_mode = true,
            InstructionType::MasterInterrupt(enable) => bus.ir_handler_mut().set_ime(*enable),
            InstructionType::Arithmetic(ar_type) => ALU::exec(ar_type, self, bus)?,
            InstructionType::Jump(condition, jump_type) => {
                jumped = self.jump(condition, jump_type);
            }
            InstructionType::Load(reg, source) => self.load(bus, reg, source)?,
            InstructionType::Store(dest, src, post_store) => {
                self.store(bus, dest, src, post_store)?
            }
            InstructionType::Push(reg_type) => {
                self.push_stack(bus, self.read_double_reg(reg_type))?
            }
            InstructionType::Pop(reg) => {
                let value = self.pop_stack(bus)?;
                self.write_double_reg(reg, value);
            }
            InstructionType::Call(call_mode) => self.call(bus, call_mode)?,
            InstructionType::Ret => self.reg_pc = self.pop_stack(bus)?,
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
        }
    }
}
