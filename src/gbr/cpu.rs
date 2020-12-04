use std::fmt;

use crate::gbr::alu::ALU;
use crate::gbr::bus::Bus;
use crate::gbr::instruction::{CbOpcode, Opcode};

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

    boot_rom_lock: bool,
    low_power_mode: bool,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            boot_rom_lock: false,
            low_power_mode: false,
            ..Default::default()
        }
    }

    pub fn boot_rom_lock(&self) -> bool {
        self.boot_rom_lock
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

    fn push_stack(&mut self, bus: &mut Bus, value: u16) {
        bus.write_byte(self.reg_sp - 1, (value >> 8) as u8);
        bus.write_byte(self.reg_sp - 2, value as u8);
        self.reg_sp -= 2;
    }

    fn pop_stack(&mut self, bus: &mut Bus) -> u16 {
        let value = bus.read_word(self.reg_sp);
        self.reg_sp += 2;
        value
    }

    pub fn step(&mut self, bus: &mut Bus) {
        let instr = bus.fetch_instruction(self.reg_pc);
        let opcode = match instr.opcode() {
            Some(op) => op,
            None => panic!(
                "Unknown instruction {:#04X}\n\n CPU state: {}",
                bus.read_byte(self.reg_pc),
                self
            ),
        };

        // println!("{:#06X}: {:#?} {:#06X}", self.reg_pc, opcode, instr.word());

        self.reg_pc += instr.length();

        match opcode {
            Opcode::Nop => (),
            Opcode::DecB => self.reg_b = ALU::dec(self, self.reg_b),
            Opcode::IncC => self.reg_c = ALU::inc(self, self.reg_c),
            Opcode::LdBd8 => self.reg_b = instr.byte(),
            Opcode::LdCd8 => self.reg_c = instr.byte(),
            Opcode::Stop => self.low_power_mode = true,
            Opcode::LdDEd16 => self.write_de(instr.word()),
            Opcode::IncDE => self.write_de(self.read_de() + 1),
            Opcode::RlA => {
                self.reg_a = ALU::rlc(self, self.reg_a);
                self.set_zero_flag(false); // investigate: why this special case?
            }
            Opcode::LdADE => self.reg_a = bus.read_byte(self.read_de()),
            Opcode::Jrnz => {
                let offset = instr.byte() as i8;
                if self.get_zero_flag() == false {
                    // TODO: find a better way to to this
                    if offset < 0 {
                        self.reg_pc -= offset.abs() as u16;
                    } else {
                        self.reg_pc += offset as u16;
                    }
                }
            }
            Opcode::LdHLd16 => self.write_hl(instr.word()),
            Opcode::LdHLincA => {
                bus.write_byte(self.read_hl(), self.reg_a);
                self.write_hl(self.read_hl() + 1);
            }
            Opcode::IncHL => self.write_hl(self.read_hl() + 1),
            Opcode::LdSPd16 => self.reg_sp = instr.word(),
            Opcode::LdHLdecA => {
                bus.write_byte(self.read_hl(), self.reg_a);
                self.write_hl(self.read_hl() - 1);
            }
            Opcode::LdAd8 => self.reg_a = instr.byte(),
            Opcode::LdCA => self.reg_c = self.reg_a,
            Opcode::LdHLA => bus.write_byte(self.read_hl(), self.reg_a),
            Opcode::LdAE => self.reg_a = self.reg_e,
            Opcode::AddAB => self.reg_a = ALU::add(self, self.reg_a, self.reg_b),
            Opcode::SubAL => self.reg_a = ALU::sub(self, self.reg_a, self.reg_l),
            Opcode::XorA => self.reg_a = ALU::xor(self, self.reg_a, self.reg_a),
            Opcode::PopBC => {
                let value = self.pop_stack(bus);
                self.write_bc(value);
            }
            Opcode::PushCB => self.push_stack(bus, self.read_bc()),
            Opcode::Ret => self.reg_pc = self.pop_stack(bus),
            Opcode::Prefix => match instr.cb_opcode() {
                CbOpcode::RlcC => self.reg_c = ALU::rlc(self, self.reg_c),
                CbOpcode::SlaB => self.reg_b = ALU::sla(self, self.reg_b),
                CbOpcode::Bit7H => ALU::test_bit(self, self.reg_h, 7),
            },
            Opcode::Calla16 => {
                self.push_stack(bus, self.reg_pc + instr.length());
                self.reg_pc = instr.word();
            }
            Opcode::Ldha8A => bus.write_byte(0xFF00 + instr.byte() as u16, self.reg_a),
            Opcode::LdhCA => bus.write_byte(0xFF00 + self.reg_c as u16, self.reg_a),
        }
    }
}

impl fmt::Display for CPU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Regs: AF {:#06X}, BC {:#06X}, DE {:#06X}, HL {:#06X}, PC {:#06X}, SP {:#06X} | Flags: Z {}, C {}, BCD-N {}, BCD-H {}",
            self.read_af(),
            self.read_bc(),
            self.read_de(),
            self.read_hl(),
            self.reg_pc,
            self.reg_sp,
            self.get_zero_flag(),
            self.get_carry_flag(),
            self.get_bcd_n_flag(),
            self.get_bcd_h_flag()
        )
    }
}
