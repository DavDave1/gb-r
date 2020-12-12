use crate::gbr::bus::Bus;
use crate::gbr::cpu::CPU;

#[macro_export]
macro_rules! gbr_panic {
    () =>  ({Err(())});
    ($msg:expr) => ({ log::error!($msg); Err(()) });
    ($msg:expr,) => ({ log::error!($msg); Err(()) });
    ($fmt:expr, $($arg:tt)+) => ({
        log::error!($fmt, $($arg)+); Err(())
    });
}

pub struct GameBoy {
    cpu: CPU,
    bus: Bus,
}

impl GameBoy {
    pub fn new(boot_rom_filename: &std::path::Path, cart_rom_filename: &std::path::Path) -> Self {
        GameBoy {
            cpu: CPU::new(),
            bus: Bus::new(boot_rom_filename, cart_rom_filename),
        }
    }

    pub fn step(&mut self) -> Result<(), ()> {
        self.cpu.step(&mut self.bus)
    }

    pub fn cpu(&self) -> &CPU {
        &self.cpu
    }

    pub fn bus(&self) -> &Bus {
        &self.bus
    }
}
