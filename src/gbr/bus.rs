use std::{fs, path::PathBuf};

use byteorder::{ByteOrder, LittleEndian};

use super::{
    apu::APU, dma::DMA, interrupts::InterruptHandler, io_registers::IORegisters, mbc::MBC,
    memory_map::*, oam::ObjAttributeMemory, ppu::PPU, serial::Serial, timer::Timer, GbError,
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
    io_registers: IORegisters,
    ppu: PPU,
    apu: APU,
    ir_handler: InterruptHandler,
    timer: Timer,
    mbc: MBC,
    dma: DMA,
    serial: Serial,
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
            io_registers: IORegisters::default(),
            ppu: PPU::new(),
            apu: APU::new(),
            ir_handler: InterruptHandler::default(),
            timer: Timer::default(),
            mbc,
            dma: DMA::new(),
            serial: Serial::default(),
        }
    }

    pub fn step(&mut self, cycles: u8) -> Result<bool, GbError> {
        self.dma.step(&self.ppu, &self.mbc, &mut self.oam, cycles)?;
        self.timer.step(cycles, &mut self.ir_handler);
        self.serial.step(cycles, &mut self.ir_handler);
        self.apu.step(cycles)?;
        self.ppu.step(&mut self.ir_handler, &self.oam, cycles)
    }

    pub fn reset(&mut self) {
        self.ppu.reset();
        self.io_registers = IORegisters::default();
    }

    pub fn ppu(&self) -> &PPU {
        &self.ppu
    }

    pub fn io_registers(&self) -> &IORegisters {
        &self.io_registers
    }

    pub fn mbc(&self) -> &MBC {
        &self.mbc
    }
}

impl BusAccess for Bus {
    fn read_byte(&self, addr: u16) -> Result<u8, GbError> {
        match map_address(addr)? {
            MappedAddress::CartRom(addr) => {
                if self.boot_rom_lock && (addr as usize) < BOOT_ROM_SIZE {
                    Ok(self.boot_rom[addr as usize])
                } else {
                    Ok(self.mbc.read_byte(addr)?)
                }
            }
            MappedAddress::VideoRam(addr) => self.ppu.read_byte(addr),
            MappedAddress::CartRam(addr) => self.mbc.read_byte(addr),
            MappedAddress::WorkRam(addr) => Ok(self.wram[(addr - WRAM_START) as usize]),
            MappedAddress::ObjectAttributeTable(_addr) => Err(GbError::Unimplemented(
                "reading object attribute table".into(),
            )),
            MappedAddress::NotUsable(addr) => {
                log::warn!("Reading byte from unusable addr {:#06X}", addr);
                Ok(0xFF)
            }
            MappedAddress::SerialRegisters => self.serial.read(addr),
            MappedAddress::TimerRegisters(addr) => self.timer.read_reg(addr),
            MappedAddress::ApuRegisters(addr) => self.apu.read_reg(addr),
            MappedAddress::PpuRegisters(addr) => self.ppu.read_reg(addr),
            MappedAddress::DmaRegister => Ok(self.dma.read_reg()),
            MappedAddress::BootRomLockRegister => Err(GbError::IllegalOp(
                "reading from boot rom lock register".into(),
            )),
            MappedAddress::IORegisters(addr) => self.io_registers.read(addr),
            MappedAddress::HighRam(addr) => Ok(self.hram[(addr - HRAM_START) as usize]),
            MappedAddress::InterruptFlagRegister => Ok(self.ir_handler.read_if()),
            MappedAddress::InterruptEnableRegister => Ok(self.ir_handler.read_ie()),
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) -> Result<(), GbError> {
        match map_address(addr)? {
            MappedAddress::CartRom(addr) => self.mbc.write_byte(addr, value)?,
            MappedAddress::VideoRam(addr) => self.ppu.write_byte(addr, value)?,
            MappedAddress::CartRam(addr) => self.mbc.write_byte(addr, value)?,
            MappedAddress::WorkRam(addr) => {
                self.wram[(addr - WRAM_START) as usize] = value;
            }
            MappedAddress::ObjectAttributeTable(addr) => self.oam.write_byte(addr, value)?,
            MappedAddress::NotUsable(addr) => {
                log::warn!("Writing byte {:#04X} to unusable addr {:#06X}", value, addr);
            }
            MappedAddress::SerialRegisters => self.serial.write(addr, value)?,
            MappedAddress::TimerRegisters(addr) => self.timer.write_reg(addr, value)?,
            MappedAddress::ApuRegisters(addr) => self.apu.write_reg(addr, value)?,
            MappedAddress::PpuRegisters(addr) => self.ppu.write_reg(addr, value)?,
            MappedAddress::DmaRegister => self.dma.write_reg(value),
            MappedAddress::BootRomLockRegister => self.boot_rom_lock = false,
            MappedAddress::IORegisters(addr) => self.io_registers.write(addr, value)?,
            MappedAddress::HighRam(addr) => self.hram[(addr - HRAM_START) as usize] = value,
            MappedAddress::InterruptFlagRegister => self.ir_handler.write_if(value),
            MappedAddress::InterruptEnableRegister => self.ir_handler.write_ie(value),
        }

        Ok(())
    }

    fn read_word(&self, addr: u16) -> Result<u16, GbError> {
        match map_address(addr)? {
            MappedAddress::CartRom(addr) => {
                if self.boot_rom_lock && (addr as usize) < BOOT_ROM_SIZE {
                    Ok(LittleEndian::read_u16(&self.boot_rom[addr as usize..]))
                } else {
                    self.mbc.read_word(addr)
                }
            }
            MappedAddress::VideoRam(addr) => self.ppu.read_word(addr),
            MappedAddress::CartRam(_addr) => self.mbc.read_word(addr),
            MappedAddress::WorkRam(_addr) => Ok(LittleEndian::read_u16(
                &self.wram[(addr - WRAM_START) as usize..],
            )),
            MappedAddress::ObjectAttributeTable(_addr) => Err(GbError::Unimplemented(
                "reading sprite attribute table".into(),
            )),
            MappedAddress::NotUsable(addr) => {
                log::warn!("Reading word from unusable addr {:#06X}", addr);
                Ok(0xFFFF)
            }
            MappedAddress::SerialRegisters => {
                Err(GbError::IllegalOp("read word from Serial registers".into()))
            }
            MappedAddress::TimerRegisters(_addr) => {
                Err(GbError::IllegalOp("read word from Timer registers".into()))
            }
            MappedAddress::ApuRegisters(_addr) => {
                Err(GbError::IllegalOp("read word from APU registers".into()))
            }
            MappedAddress::PpuRegisters(_addr) => {
                Err(GbError::IllegalOp("read word from PPU registers".into()))
            }
            MappedAddress::DmaRegister => {
                Err(GbError::IllegalOp("read word from DMA register".into()))
            }
            MappedAddress::BootRomLockRegister => Err(GbError::IllegalOp(
                "reading from boot rom lock register".into(),
            )),
            MappedAddress::IORegisters(_addr) => {
                Err(GbError::Unimplemented("reading IO registers".into()))
            }
            MappedAddress::HighRam(addr) => Ok(LittleEndian::read_u16(
                &self.hram[(addr - HRAM_START) as usize..],
            )),
            MappedAddress::InterruptFlagRegister => {
                Err(GbError::IllegalOp("reading interrupt flag register".into()))
            }
            MappedAddress::InterruptEnableRegister => Err(GbError::IllegalOp(
                "reading interrupt enable register".into(),
            )),
        }
    }

    fn ir_handler(&self) -> &InterruptHandler {
        &self.ir_handler
    }

    fn ir_handler_mut(&mut self) -> &mut InterruptHandler {
        &mut self.ir_handler
    }
}
