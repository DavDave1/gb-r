use std::path::PathBuf;

use crate::gbr::{bus::Bus, cpu::CPU, ppu::PPU, GbError};

use super::{
    bus::BusAccess, cpu::CpuState, interrupts::InterruptHandlerState, joypad::Joypad,
    mbc::MbcState, oam::ObjAttribute, ppu::PpuState,
};

#[derive(Default, Clone)]
pub struct GbState {
    pub cpu: CpuState,
    pub ir_handler: InterruptHandlerState,
    pub ppu: PpuState,
    pub mbc: MbcState,
    pub oam: Vec<ObjAttribute>,
    pub joypad: Joypad,
}

pub struct GameBoy {
    cpu: CPU,
    bus: Bus,
}

impl GameBoy {
    pub fn new(boot_rom_filename: Option<PathBuf>, cart_rom_filename: Option<PathBuf>) -> Self {
        GameBoy {
            cpu: CPU::new(),
            bus: Bus::new(boot_rom_filename, cart_rom_filename),
        }
    }

    pub fn step(&mut self) -> Result<bool, GbError> {
        let cycles = self.cpu.step(&mut self.bus)?;

        self.bus.step(4 * cycles)
    }

    pub fn reset(&mut self) {
        self.cpu = CPU::new();
        self.bus.reset();
    }

    pub fn cpu(&self) -> &CPU {
        &self.cpu
    }

    pub fn bus(&self) -> &Bus {
        &self.bus
    }

    pub fn ppu(&self) -> &PPU {
        &self.bus.ppu()
    }

    pub fn collect_state(&self) -> GbState {
        GbState {
            cpu: self.cpu.state(),
            ir_handler: self.bus.ir_handler().state(),
            ppu: self.bus.ppu().state(),
            mbc: self.bus.mbc().state(),
            oam: self.bus.oam().state(),
            joypad: self.bus.joypad().clone(),
        }
    }
}
