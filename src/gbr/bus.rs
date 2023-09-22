use std::{fs, path::PathBuf};

use byteorder::{ByteOrder, LittleEndian};

use super::{
    apu::APU, instruction::Instruction, io_registers::IORegisters, memory_map::*, ppu::PPU, GbError,
};

pub struct Bus {
    boot_rom_lock: bool,
    boot_rom: Box<[u8]>,
    cart_rom: Box<[u8]>,
    hram: Box<[u8]>,
    io_registers: IORegisters,
    ppu: PPU,
    apu: APU,
}

impl Bus {
    pub fn new(boot_rom_filename: Option<PathBuf>, cart_rom_filename: Option<PathBuf>) -> Self {
        let boot_rom_filename = boot_rom_filename.expect("Boot rom path not provided");
        let boot_rom = fs::read(boot_rom_filename).expect("Failed to read boot rom");

        if boot_rom.len() != BOOT_ROM_SIZE {
            panic!("Wrong boot rom size");
        }

        let cart_rom = match cart_rom_filename {
            Some(path) => fs::read(path)
                .map_err(|err| {
                    log::error!("Failed to read cart ROM: {}", err);
                    Vec::<u8>::new()
                })
                .unwrap(),
            None => vec![],
        };

        Bus {
            boot_rom_lock: true,
            boot_rom: boot_rom.into_boxed_slice(),
            cart_rom: cart_rom.into_boxed_slice(),
            hram: vec![0; HIGH_RAM_SIZE].into_boxed_slice(),
            io_registers: IORegisters::default(),
            ppu: PPU::new(),
            apu: APU::new(),
        }
    }

    pub fn step(&mut self, cycles: u8) -> Result<bool, GbError> {
        self.apu.step(cycles)?;
        self.ppu.step(cycles)
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

    pub fn fetch_instruction(&self, addr: u16) -> Result<Instruction, GbError> {
        match map_address(addr)? {
            MappedAddress::RomBank0(addr) => {
                if self.boot_rom_lock {
                    Instruction::decode(&self.boot_rom[addr as usize..addr as usize + 3])
                } else {
                    Err(GbError::Unimplemented(
                        "fetching instruction from cart".into(),
                    ))
                }
            }
            _ => Err(GbError::Unimplemented(
                "fetching instruction outside form bank 0".into(),
            )),
        }
    }

    pub fn read_byte(&self, addr: u16) -> Result<u8, GbError> {
        match map_address(addr)? {
            MappedAddress::RomBank0(addr) => {
                if self.boot_rom_lock && (addr as usize) < BOOT_ROM_SIZE {
                    Ok(self.boot_rom[addr as usize])
                } else if !self.cart_rom.is_empty() {
                    Ok(self.cart_rom[addr as usize])
                } else {
                    Ok(0xFF)
                }
            }
            MappedAddress::RomActiveBank(_addr) => Err(GbError::Unimplemented(
                "reading from cart active bank".into(),
            )),
            MappedAddress::VideoRam(addr) => self.ppu.read_byte(addr),
            MappedAddress::ExternalRam(_addr) => {
                Err(GbError::Unimplemented("reading from external ram".into()))
            }
            MappedAddress::WorkRamBank0(_addr) => Err(GbError::Unimplemented(
                "reading from work ram bank 0".into(),
            )),
            MappedAddress::WorkRamActiveBank(_addr) => Err(GbError::Unimplemented(
                "reading from work ram active bank".into(),
            )),
            MappedAddress::SpriteAttributeTable(_addr) => Err(GbError::Unimplemented(
                "reading sprite attribute table".into(),
            )),
            MappedAddress::ApuRegisters(addr) => self.apu.read_reg(addr),
            MappedAddress::PpuRegisters(addr) => self.ppu.read_reg(addr),
            MappedAddress::IORegisters(addr) => self.io_registers.read(addr),
            MappedAddress::HighRam(addr) => Ok(self.hram[addr as usize]),
            MappedAddress::InterruptEnableRegister => Err(GbError::Unimplemented(
                "reading interrupr enable register".into(),
            )),
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) -> Result<(), GbError> {
        match map_address(addr)? {
            MappedAddress::RomBank0(_addr) => Err(GbError::IllegalOp("write to rom bank 0".into())),
            MappedAddress::RomActiveBank(_addr) => {
                Err(GbError::IllegalOp("write to rom active bank".into()))
            }
            MappedAddress::VideoRam(addr) => self.ppu.write_byte(addr, value),
            MappedAddress::ExternalRam(_addr) => {
                Err(GbError::Unimplemented("writing to external ram".into()))
            }
            MappedAddress::WorkRamBank0(_addr) => {
                Err(GbError::Unimplemented("writing to work ram 0".into()))
            }
            MappedAddress::WorkRamActiveBank(_addr) => Err(GbError::Unimplemented(
                "writing to work ram active bank".into(),
            )),
            MappedAddress::SpriteAttributeTable(_addr) => Err(GbError::Unimplemented(
                "writing to sprite attribute table".into(),
            )),
            MappedAddress::ApuRegisters(addr) => self.apu.write_reg(addr, value),
            MappedAddress::PpuRegisters(addr) => self.ppu.write_reg(addr, value),
            MappedAddress::IORegisters(addr) => self.io_registers.write(addr, value),
            MappedAddress::HighRam(addr) => {
                self.hram[addr as usize] = value;
                Ok(())
            }
            MappedAddress::InterruptEnableRegister => Err(GbError::Unimplemented(
                "writing interrubt enable register".into(),
            )),
        }
    }

    pub fn read_word(&self, addr: u16) -> Result<u16, GbError> {
        match map_address(addr)? {
            MappedAddress::RomBank0(addr) => {
                if self.boot_rom_lock {
                    Ok(LittleEndian::read_u16(&self.boot_rom[addr as usize..]))
                } else {
                    Err(GbError::Unimplemented("reading from cart bank 0".into()))
                }
            }
            MappedAddress::RomActiveBank(_addr) => Err(GbError::Unimplemented(
                "reading from cart active bank".into(),
            )),
            MappedAddress::VideoRam(addr) => self.ppu.read_word(addr),
            MappedAddress::ExternalRam(_addr) => {
                Err(GbError::Unimplemented("reading from external ram".into()))
            }
            MappedAddress::WorkRamBank0(_addr) => {
                Err(GbError::Unimplemented("reading from work ram 0".into()))
            }
            MappedAddress::WorkRamActiveBank(_addr) => Err(GbError::Unimplemented(
                "reading from work ram active bank".into(),
            )),
            MappedAddress::SpriteAttributeTable(_addr) => Err(GbError::Unimplemented(
                "reading sprite attribute table".into(),
            )),
            MappedAddress::ApuRegisters(_addr) => Err(GbError::Unimplemented(
                "read word from APU registers".into(),
            )),
            MappedAddress::PpuRegisters(_addr) => Err(GbError::Unimplemented(
                "read word from PPU registers".into(),
            )),
            MappedAddress::IORegisters(_addr) => {
                Err(GbError::Unimplemented("reading IO registers".into()))
            }
            MappedAddress::HighRam(addr) => Ok(LittleEndian::read_u16(&self.hram[addr as usize..])),
            MappedAddress::InterruptEnableRegister => Err(GbError::Unimplemented(
                "reading interrupt enable register".into(),
            )),
        }
    }
}
