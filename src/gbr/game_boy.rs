use crate::gbr::bus::Bus;
use crate::gbr::cpu::CPU;

pub struct GameBoy {
    cpu: CPU,
    bus: Bus,
}

impl GameBoy {
    pub fn new(boot_rom_filename: &std::path::Path) -> Self {
        GameBoy {
            cpu: CPU::new(),
            bus: Bus::new(boot_rom_filename),
        }
    }

    pub fn run(&mut self) {
        loop {
            self.cpu.step(&mut self.bus);
        }
    }
}
