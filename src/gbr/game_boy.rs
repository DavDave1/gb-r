use std::path::PathBuf;

use crate::gbr::{bus::Bus, cpu::CPU, ppu::PPU, GbError};

pub struct GameBoy {
    cpu: CPU,
    bus: Bus,
}

impl GameBoy {
    pub fn new(boot_rom_filename: Option<PathBuf>, cart_rom_filename: Option<PathBuf>) -> Self {
        let bus = Bus::new(boot_rom_filename, cart_rom_filename);

        GameBoy {
            cpu: CPU::new(),
            bus,
        }
    }

    pub fn step(&mut self) -> Result<bool, GbError> {
        let cycles = self.cpu.step(&mut self.bus)?;

        self.bus.ppu_mut().step(cycles)
    }

    pub fn reset(&mut self) {}

    pub fn cpu(&self) -> &CPU {
        &self.cpu
    }

    pub fn bus(&self) -> &Bus {
        &self.bus
    }

    pub fn ppu(&self) -> &PPU {
        &self.bus.ppu()
    }
}
