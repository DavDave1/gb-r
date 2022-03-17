use std::fmt;

use crate::gbr::alu::ALU;
use crate::gbr::bus::Bus;
use crate::gbr::instruction::{CbOpcode, Opcode};
use crate::gbr::GbError;

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

    fn jump(&mut self, offset: i8) {
        // TODO: find a better way to to this
        if offset < 0 {
            self.reg_pc -= offset.abs() as u16;
        } else {
            self.reg_pc += offset as u16;
        }
    }

    pub fn step(&mut self, bus: &mut Bus) -> Result<(), GbError> {
        let instr = bus.fetch_instruction(self.reg_pc)?;
        let opcode = match instr.opcode() {
            Some(op) => op,
            None => {
                let byte = bus.read_byte(self.reg_pc)?;
                return Err(GbError::UnknownInstruction(byte));
            }
        };

        self.reg_pc += instr.length()?;

        match opcode {
            Opcode::Nop => (),
            Opcode::DecB => self.reg_b = ALU::dec(self, self.reg_b),
            Opcode::IncB => self.reg_c = ALU::inc(self, self.reg_b),
            Opcode::IncC => self.reg_c = ALU::inc(self, self.reg_c),
            Opcode::DecC => self.reg_c = ALU::dec(self, self.reg_c),
            Opcode::LdBd8 => self.reg_b = instr.byte(),
            Opcode::LdCd8 => self.reg_c = instr.byte(),
            Opcode::LdEd8 => self.reg_e = instr.byte(),
            Opcode::Stop => self.low_power_mode = true,
            Opcode::LdDEd16 => self.write_de(instr.word()),
            Opcode::IncDE => self.write_de(self.read_de() + 1),
            Opcode::RlA => {
                self.reg_a = ALU::rlc(self, self.reg_a);
                self.set_zero_flag(false); // investigate: why this special case?
            }
            Opcode::LdADE => self.reg_a = bus.read_byte(self.read_de())?,
            Opcode::Jr => self.jump(instr.byte() as i8),
            Opcode::Jrnz => {
                if self.get_zero_flag() == false {
                    self.jump(instr.byte() as i8);
                }
            }
            Opcode::Jrz => {
                if self.get_zero_flag() == true {
                    self.jump(instr.byte() as i8);
                }
            }
            Opcode::LdHLd16 => self.write_hl(instr.word()),
            Opcode::LdHLincA => {
                bus.write_byte(self.read_hl(), self.reg_a)?;
                self.write_hl(self.read_hl() + 1);
            }
            Opcode::IncHL => self.write_hl(self.read_hl() + 1),
            Opcode::LdLd8 => self.reg_l = instr.byte(),
            Opcode::LdSPd16 => self.reg_sp = instr.word(),
            Opcode::LdHLdecA => {
                bus.write_byte(self.read_hl(), self.reg_a)?;
                self.write_hl(self.read_hl() - 1);
            }
            Opcode::DecA => self.reg_a = ALU::dec(self, self.reg_a),
            Opcode::LdAd8 => self.reg_a = instr.byte(),
            Opcode::LdCA => self.reg_c = self.reg_a,
            Opcode::LdDA => self.reg_d = self.reg_a,
            Opcode::LdHA => self.reg_h = self.reg_a,
            Opcode::LdHLA => bus.write_byte(self.read_hl(), self.reg_a)?,
            Opcode::LdAE => self.reg_a = self.reg_e,
            Opcode::AddAB => self.reg_a = ALU::add(self, self.reg_a, self.reg_b),
            Opcode::SubAL => self.reg_a = ALU::sub(self, self.reg_a, self.reg_l),
            Opcode::XorA => self.reg_a = ALU::xor(self, self.reg_a, self.reg_a),
            Opcode::PopBC => {
                let value = self.pop_stack(bus)?;
                self.write_bc(value);
            }
            Opcode::PushCB => self.push_stack(bus, self.read_bc())?,
            Opcode::Ret => self.reg_pc = self.pop_stack(bus)?,
            Opcode::Prefix => match instr.cb_opcode() {
                Some(CbOpcode::RlcC) => self.reg_c = ALU::rlc(self, self.reg_c),
                Some(CbOpcode::SlaB) => self.reg_b = ALU::sla(self, self.reg_b),
                Some(CbOpcode::Bit7H) => ALU::test_bit(self, self.reg_h, 7),
                None => {
                    return Err(GbError::UnknownCbInstruction(instr.byte()));
                }
            },
            Opcode::Calla16 => {
                let instr_len = instr.length()?;
                self.push_stack(bus, self.reg_pc + instr_len)?;
                self.reg_pc = instr.word();
            }
            Opcode::Ldha8A => bus.write_byte(0xFF00 + instr.byte() as u16, self.reg_a)?,
            Opcode::Lda16A => self.reg_a = bus.read_byte(instr.word())?,
            Opcode::LdhCA => bus.write_byte(0xFF00 + self.reg_c as u16, self.reg_a)?,
            Opcode::LdhAa8 => self.reg_a = bus.read_byte(0xFF00 + instr.byte() as u16)?,
            Opcode::Cpd8 => ALU::cp(self, self.reg_a, instr.byte()),
        };
        Ok(())
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
