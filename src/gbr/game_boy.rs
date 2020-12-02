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
            memory: Memory::default(),
            cpu: CPU::new(),
        }
    }
    pub fn load_boot_rom(&mut self, boot_rom_filename: &std::path::Path) -> Result<(), Error> {
        self.memory.load_boot_rom(boot_rom_filename)
    }

    pub fn run(&mut self) {
        loop {
            self.cpu.step(&mut self.memory);
            //    println!("CPU: {}", self.cpu);
        }
    }
}
