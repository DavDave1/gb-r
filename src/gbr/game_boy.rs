use crate::gbr::{bus::Bus, cpu::CPU, ppu::PPU, GbError};

pub struct GameBoy {
    cpu: CPU,
    bus: Bus,
    ppu: PPU,
}

impl GameBoy {
    pub fn new(boot_rom_filename: &std::path::Path, cart_rom_filename: &std::path::Path) -> Self {
        GameBoy {
            cpu: CPU::new(),
            bus: Bus::new(boot_rom_filename, cart_rom_filename),
            ppu: PPU::new(),
        }
    }

    pub fn step(&mut self) -> Result<(), GbError> {
        if self.bus().io_registers().lcd_control().display_enable() {
            self.ppu.render(&self.bus)?;
        }
        self.cpu.step(&mut self.bus)
    }

    pub fn cpu(&self) -> &CPU {
        &self.cpu
    }

    pub fn bus(&self) -> &Bus {
        &self.bus
    }

    pub fn ppu(&self) -> &PPU {
        &self.ppu
    }
}
