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
            println!("CPU: {}", self.cpu);
        }
    }

    fn step(&mut self) {
        let opcode: u8 = self.memory.read_byte(self.cpu.reg_pc);
        let byte = self.memory.read_byte(self.cpu.reg_pc + 1);
        let word = self.memory.read_word(self.cpu.reg_pc + 1);

        self.cpu.reg_pc += 1;
        println!("{:#06x}: {:#04x} {:#06x}", self.cpu.reg_pc, opcode, word);

        match opcode {
            0x20 => {
                // JR, NZ
                let offset = byte as u16;
                if (self.cpu.get_zero_flag() == false) {
                    self.cpu.reg_pc += offset;
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
                // LD (HL-), A
                self.memory.write_byte(self.cpu.read_hl(), self.cpu.reg_a);
                self.cpu.write_hl(self.cpu.read_hl() - 1);
            }
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
                    _ => panic!("Unknown CB instruction {:#x}", cb_opcode),
                }
                self.cpu.reg_pc += 1;
            }
            _ => panic!("Unknown instruction {:#x}", opcode),
        }
    }
}
