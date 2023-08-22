use std::sync::Arc;

use crate::gbr::{bus::Bus, cpu::CPU, ppu::PPU, GbError};

pub struct GameBoy {
    cpu: CPU,
    bus: Bus,
    ppu: Arc<PPU>,
}

impl GameBoy {
    pub fn new(boot_rom_filename: &std::path::Path, cart_rom_filename: &std::path::Path) -> Self {
        let ppu = Arc::new(PPU::new());
        let bus = Bus::new(boot_rom_filename, cart_rom_filename, ppu.clone());

        GameBoy {
            cpu: CPU::new(),
            bus,
            ppu,
        }
    }

    pub fn step(&mut self) -> Result<(), GbError> {
        let _cycles = self.cpu.step(&mut self.bus)?;

        if self.bus().io_registers().lcd_control().display_enable() {
            self.ppu.render()?;
        }

        Ok(())
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
