use std::{fs, path::PathBuf};

use byteorder::{ByteOrder, LittleEndian};

use super::{
    apu::APU, dma::DMA, interrupts::InterruptHandler, joypad::Joypad, mbc::MBC, memory_map::*,
    oam::ObjAttributeMemory, ppu::PPU, serial::Serial, timer::Timer, GbError,
};

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait BusAccess {
    fn read_byte(&self, addr: u16) -> Result<u8, GbError>;
    fn write_byte(&mut self, addr: u16, value: u8) -> Result<(), GbError>;
    fn read_word(&self, addr: u16) -> Result<u16, GbError>;

    fn ir_handler(&self) -> &InterruptHandler;
    fn ir_handler_mut(&mut self) -> &mut InterruptHandler;
}

pub struct Bus {
    boot_rom_lock: bool,
    boot_rom: Box<[u8]>,
    hram: Box<[u8]>,
    wram: Box<[u8]>,
    oam: ObjAttributeMemory,
    ppu: PPU,
    apu: APU,
    ir_handler: InterruptHandler,
    timer: Timer,
    mbc: MBC,
    dma: DMA,
    serial: Serial,
    joypad: Joypad,
}

impl Bus {
    pub fn new(boot_rom_filename: Option<PathBuf>, cart_rom_filename: Option<PathBuf>) -> Self {
        let boot_rom_filename = boot_rom_filename.expect("Boot rom path not provided");
        let boot_rom = fs::read(boot_rom_filename).expect("Failed to read boot rom");

        if boot_rom.len() != BOOT_ROM_SIZE {
            panic!("Wrong boot rom size");
        }

        let mbc = match cart_rom_filename {
            Some(path) => MBC::new(&path).unwrap(),
            None => MBC::default(),
        };

        Bus {
            boot_rom_lock: true,
            boot_rom: boot_rom.into_boxed_slice(),
            hram: vec![0; HRAM_SIZE].into_boxed_slice(),
            wram: vec![0; WRAM_SIZE].into_boxed_slice(),
            oam: ObjAttributeMemory::new(),
            ppu: PPU::new(),
            apu: APU::new(),
            ir_handler: InterruptHandler::default(),
            timer: Timer::default(),
            mbc,
            dma: DMA::new(),
            serial: Serial::default(),
            joypad: Joypad::default(),
        }
    }

    pub fn step(&mut self, cycles: u8) -> Result<bool, GbError> {
        self.dma
            .step(&self.wram, &self.ppu, &self.mbc, &mut self.oam, cycles)?;
        self.timer.step(cycles, &mut self.ir_handler);
        self.serial.step(cycles, &mut self.ir_handler);
        self.apu.step(cycles)?;
        self.ppu
            .step(&mut self.ir_handler, &self.oam, cycles as u16)
    }

    pub fn reset(&mut self) {
        self.ppu.reset();
        self.boot_rom_lock = true;
        self.hram.fill(0);
        self.wram.fill(0);
        self.oam = ObjAttributeMemory::new();
        self.apu = APU::new();
        self.ir_handler = InterruptHandler::default();
        self.timer = Timer::default();
        self.dma = DMA::new();
        self.serial = Serial::default();
        self.joypad = Joypad::default();

        // TODO reset MBC
    }

    pub fn ppu(&self) -> &PPU {
        &self.ppu
    }

    pub fn mbc(&self) -> &MBC {
        &self.mbc
    }

    pub fn oam(&self) -> &ObjAttributeMemory {
        &self.oam
    }

    pub fn joypad(&self) -> &Joypad {
        &self.joypad
    }

    pub fn joypad_mut(&mut self) -> &mut Joypad {
        &mut self.joypad
    }
}

impl BusAccess for Bus {
    fn read_byte(&self, addr: u16) -> Result<u8, GbError> {
        match map_address(addr) {
            MappedAddress::CartRom => {
                if self.boot_rom_lock && (addr as usize) < BOOT_ROM_SIZE {
                    Ok(self.boot_rom[addr as usize])
                } else {
                    Ok(self.mbc.read_byte(addr)?)
                }
            }
            MappedAddress::VideoRam => self.ppu.read_byte(addr),
            MappedAddress::CartRam => self.mbc.read_byte(addr),
            MappedAddress::WorkRam => Ok(self.wram[(addr - WRAM_START) as usize]),
            MappedAddress::EchoRam => Ok(self.wram[(addr - ECHO_RAM_START) as usize]),
            MappedAddress::ObjectAttributeTable => Err(GbError::Unimplemented(
                "reading object attribute table".into(),
            )),
            MappedAddress::NotUsable => {
                log::warn!("Reading byte from unusable addr {:#06X}", addr);
                Ok(0xFF)
            }
            MappedAddress::JoypadRegister => Ok(self.joypad.read()),
            MappedAddress::SerialRegisters => self.serial.read(addr),
            MappedAddress::TimerRegisters => self.timer.read_reg(addr),
            MappedAddress::ApuRegisters => self.apu.read_reg(addr),
            MappedAddress::PpuRegisters => self.ppu.read_reg(addr),
            MappedAddress::DmaRegister => Ok(self.dma.read_reg()),
            MappedAddress::BootRomLockRegister => Err(GbError::IllegalOp(
                "reading from boot rom lock register".into(),
            )),
            MappedAddress::HighRam => Ok(self.hram[(addr - HRAM_START) as usize]),
            MappedAddress::InterruptFlagRegister => Ok(self.ir_handler.read_if()),
            MappedAddress::InterruptEnableRegister => Ok(self.ir_handler.read_ie()),
            MappedAddress::InvalidAddress => Ok(0xFF),
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) -> Result<(), GbError> {
        match map_address(addr) {
            MappedAddress::CartRom => self.mbc.write_byte(addr, value)?,
            MappedAddress::VideoRam => self.ppu.write_byte(addr, value)?,
            MappedAddress::CartRam => self.mbc.write_byte(addr, value)?,
            MappedAddress::WorkRam => {
                self.wram[(addr - WRAM_START) as usize] = value;
            }
            MappedAddress::EchoRam => {
                return Err(GbError::IllegalOp(format!(
                    "Write to echo ram addr {:#06X}",
                    addr
                )));
            }
            MappedAddress::ObjectAttributeTable => self.oam.write_byte(addr, value)?,
            MappedAddress::NotUsable => {
                log::warn!("Writing byte {:#04X} to unusable addr {:#06X}", value, addr);
            }
            MappedAddress::JoypadRegister => self.joypad.write(value),
            MappedAddress::SerialRegisters => self.serial.write(addr, value)?,
            MappedAddress::TimerRegisters => self.timer.write_reg(addr, value)?,
            MappedAddress::ApuRegisters => self.apu.write_reg(addr, value)?,
            MappedAddress::PpuRegisters => self.ppu.write_reg(addr, value)?,
            MappedAddress::DmaRegister => self.dma.write_reg(value),
            MappedAddress::BootRomLockRegister => self.boot_rom_lock = false,
            MappedAddress::HighRam => self.hram[(addr - HRAM_START) as usize] = value,
            MappedAddress::InterruptFlagRegister => self.ir_handler.write_if(value),
            MappedAddress::InterruptEnableRegister => self.ir_handler.write_ie(value),
            MappedAddress::InvalidAddress => (),
        }

        Ok(())
    }

    fn read_word(&self, addr: u16) -> Result<u16, GbError> {
        match map_address(addr) {
            MappedAddress::CartRom => {
                if self.boot_rom_lock && (addr as usize) < BOOT_ROM_SIZE {
                    Ok(LittleEndian::read_u16(&self.boot_rom[addr as usize..]))
                } else {
                    self.mbc.read_word(addr)
                }
            }
            MappedAddress::VideoRam => self.ppu.read_word(addr),
            MappedAddress::CartRam => self.mbc.read_word(addr),
            MappedAddress::WorkRam => Ok(LittleEndian::read_u16(
                &self.wram[(addr - WRAM_START) as usize..],
            )),
            MappedAddress::EchoRam => Ok(LittleEndian::read_u16(
                &self.wram[(addr - ECHO_RAM_START) as usize..],
            )),
            MappedAddress::ObjectAttributeTable => Err(GbError::Unimplemented(
                "reading sprite attribute table".into(),
            )),
            MappedAddress::NotUsable => {
                log::warn!("Reading word from unusable addr {:#06X}", addr);
                Ok(0xFFFF)
            }
            MappedAddress::JoypadRegister => {
                Err(GbError::IllegalOp("read word from Joypad register".into()))
            }
            MappedAddress::SerialRegisters => {
                Err(GbError::IllegalOp("read word from Serial registers".into()))
            }
            MappedAddress::TimerRegisters => {
                Err(GbError::IllegalOp("read word from Timer registers".into()))
            }
            MappedAddress::ApuRegisters => {
                Err(GbError::IllegalOp("read word from APU registers".into()))
            }
            MappedAddress::PpuRegisters => {
                Err(GbError::IllegalOp("read word from PPU registers".into()))
            }
            MappedAddress::DmaRegister => {
                Err(GbError::IllegalOp("read word from DMA register".into()))
            }
            MappedAddress::BootRomLockRegister => Err(GbError::IllegalOp(
                "reading from boot rom lock register".into(),
            )),

            MappedAddress::HighRam => Ok(LittleEndian::read_u16(
                &self.hram[(addr - HRAM_START) as usize..],
            )),
            MappedAddress::InterruptFlagRegister => {
                Err(GbError::IllegalOp("reading interrupt flag register".into()))
            }
            MappedAddress::InterruptEnableRegister => Err(GbError::IllegalOp(
                "reading interrupt enable register".into(),
            )),
            MappedAddress::InvalidAddress => Ok(0xFFFF),
        }
    }

    fn ir_handler(&self) -> &InterruptHandler {
        &self.ir_handler
    }

    fn ir_handler_mut(&mut self) -> &mut InterruptHandler {
        &mut self.ir_handler
    }
}
