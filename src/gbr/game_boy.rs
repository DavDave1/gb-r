use std::io::Error;

use crate::gbr::cpu::CPU;
use crate::gbr::memory::Memory;

pub struct GameBoy {
    cpu: CPU,
    memory: Memory,
}

impl GameBoy {
    pub fn new() -> Self {
        GameBoy {
            cpu: CPU::new(),
            memory: Memory::default(),
        }
    }
    pub fn load_boot_rom(&mut self, boot_rom_filename: &std::path::Path) -> Result<(), Error> {
        self.memory.load_boot_rom(boot_rom_filename)
    }

    pub fn run(&mut self) {
        loop {
            self.step();
            //    println!("CPU: {}", self.cpu);
        }
    }

    fn step(&mut self) {
        let opcode: u8 = self.memory.read_byte(self.cpu.reg_pc);
        let byte = self.memory.read_byte(self.cpu.reg_pc + 1);
        let word = self.memory.read_word(self.cpu.reg_pc + 1);

        println!("{:#06X}: {:#04X} {:#06X}", self.cpu.reg_pc, opcode, word);

        self.cpu.reg_pc += 1;

        match opcode {
            0x0C => {
                // INC C
                let overflow = self.cpu.reg_c & 0x03 != 0;
                self.cpu.reg_c += 1;
                self.cpu.set_zero_flag(self.cpu.reg_c == 0);
                self.cpu.set_bcd_n_flag(false);
                self.cpu.set_bcd_h_flag(overflow);
            }
            0x0E => {
                // LD C, d8
                self.cpu.reg_c = byte;
                self.cpu.reg_pc += 1;
            }
            0x3E => {
                // LD A, d8
                self.cpu.reg_a = byte;
                self.cpu.reg_pc += 1;
            }
            0x20 => {
                // JR, NZ
                let offset = byte as i8;
                self.cpu.reg_pc += 1;

                if self.cpu.get_zero_flag() == false {
                    // TODO: find a better way to to this
                    if offset < 0 {
                        self.cpu.reg_pc -= offset.abs() as u16;
                    } else {
                        self.cpu.reg_pc += offset as u16;
                    }
                }
            }
            0x21 => {
                // LD HL, nn
                self.cpu.write_hl(word);
                self.cpu.reg_pc += 2;
            }
            0x31 => {
                // load SP immediate
                self.cpu.reg_sp = word;
                self.cpu.reg_pc += 2;
            }
            0x32 => {
                // LD HL-, A
                self.memory.write_byte(self.cpu.read_hl(), self.cpu.reg_a);
                self.cpu.write_hl(self.cpu.read_hl() - 1);
            }
            0x77 => self.memory.write_byte(self.cpu.read_hl(), self.cpu.reg_a), // LD HL, A
            0xAF => {
                // xor A
                self.cpu.reg_a ^= self.cpu.reg_a;
                self.cpu.set_zero_flag(self.cpu.reg_a == 0);
            }
            0xCB => {
                let cb_opcode = byte;
                match cb_opcode {
                    0x20 => {
                        // SLA B
                        let ext_b = (self.cpu.reg_b as u16) << 1;

                        self.cpu.set_carry_flag(ext_b & 0x0100 != 0);
                        self.cpu.set_zero_flag(ext_b & 0x00FF == 0);

                        self.cpu.reg_b = ext_b as u8;
                    }
                    0x7C => {
                        // BIT 7,H
                        self.cpu.set_zero_flag(self.cpu.reg_h & 0b10000000 == 0);
                        self.cpu.set_bcd_h_flag(true);
                    }
                    _ => panic!("Unknown CB instruction {:#04X}", cb_opcode),
                }
                self.cpu.reg_pc += 1;
            }
            0xE0 => {
                // LDH a8, A
                self.memory.write_byte(0xFF00 + byte as u16, self.cpu.reg_a);
                self.cpu.reg_pc += 1;
            }
            0xE2 => self.cpu.reg_c = self.cpu.reg_a, // LD C, A

            _ => panic!("Unknown instruction {:#04X}", opcode),
        }
    }
}
