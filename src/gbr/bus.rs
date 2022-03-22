use std::fs;

use byteorder::{ByteOrder, LittleEndian};

use crate::gbr::instruction::Instruction;
use crate::gbr::io_registers::IORegisters;
use crate::gbr::memory_map::*;
use crate::gbr::GbError;

#[derive(PartialEq)]
pub enum ComponentType {
    CPU,
    PPU,
}

pub struct Bus {
    boot_rom_lock: bool,
    boot_rom: Box<[u8]>,
    cart_rom: Box<[u8]>,
    vram: Box<[u8]>,
    hram: Box<[u8]>,
    io_registers: IORegisters,
}

impl Bus {
    pub fn new(boot_rom_filename: &std::path::Path, cart_rom_filename: &std::path::Path) -> Self {
        let boot_rom = fs::read(boot_rom_filename).expect("Failed to read boot rom");

        if boot_rom.len() != BOOT_ROM_SIZE {
            panic!("Wrong boot rom size");
        }

        let cart_rom = fs::read(cart_rom_filename).expect("Failed to read catridge rom");

        Bus {
            boot_rom_lock: true,
            boot_rom: boot_rom.into_boxed_slice(),
            cart_rom: cart_rom.into_boxed_slice(),
            vram: vec![0; VIDEO_RAM_SIZE].into_boxed_slice(),
            hram: vec![0; HIGH_RAM_SIZE].into_boxed_slice(),
            io_registers: IORegisters::default(),
        }
    }

    pub fn io_registers(&self) -> &IORegisters {
        &self.io_registers
    }

    pub fn fetch_instruction(&self, addr: u16) -> Result<Instruction, GbError> {
        match map_address(addr)? {
            MappedAddress::RomBank0(addr) => {
                if self.boot_rom_lock {
                    Ok(Instruction::new(
                        &self.boot_rom[addr as usize..addr as usize + 3],
                    ))
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

    pub fn read_byte(&self, addr: u16, comp: ComponentType) -> Result<u8, GbError> {
        match map_address(addr)? {
            MappedAddress::RomBank0(addr) => {
                if self.boot_rom_lock && (addr as usize) < BOOT_ROM_SIZE {
                    Ok(self.boot_rom[addr as usize])
                } else {
                    Ok(self.cart_rom[addr as usize])
                }
            }
            MappedAddress::RomActiveBank(addr) => Err(GbError::Unimplemented(
                "reading from cart active bank".into(),
            )),
            MappedAddress::VideoRam(addr) => {
                if comp == ComponentType::CPU && self.io_registers.lcd_control().display_enable() {
                    return Err(GbError::IllegalOp(
                        "reading vram while lcd is active".into(),
                    ));
                }

                Ok(self.vram[addr as usize])
            }
            MappedAddress::ExternalRam(addr) => {
                Err(GbError::Unimplemented("reading from external ram".into()))
            }
            MappedAddress::WorkRamBank0(addr) => Err(GbError::Unimplemented(
                "reading from work ram bank 0".into(),
            )),
            MappedAddress::WorkRamActiveBank(addr) => Err(GbError::Unimplemented(
                "reading from work ram active bank".into(),
            )),
            MappedAddress::SpriteAttributeTable(addr) => Err(GbError::Unimplemented(
                "reading sprite attribute table".into(),
            )),
            MappedAddress::IORegisters(addr) => self.io_registers.read(addr),
            MappedAddress::HighRam(addr) => Ok(self.hram[addr as usize]),
            MappedAddress::InterruptEnableRegister => Err(GbError::Unimplemented(
                "reading interrupr enable register".into(),
            )),
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8, comp: ComponentType) -> Result<(), GbError> {
        match map_address(addr)? {
            MappedAddress::RomBank0(addr) => Err(GbError::IllegalOp("write to rom bank 0".into())),
            MappedAddress::RomActiveBank(addr) => {
                Err(GbError::IllegalOp("write to rom active bank".into()))
            }
            MappedAddress::VideoRam(addr) => {
                if comp == ComponentType::CPU && self.io_registers.lcd_control().display_enable() {
                    return Err(GbError::IllegalOp(
                        "write to vram while lcd is active".into(),
                    ));
                }

                self.vram[addr as usize] = value;
                Ok(())
            }
            MappedAddress::ExternalRam(addr) => {
                Err(GbError::Unimplemented("writing to external ram".into()))
            }
            MappedAddress::WorkRamBank0(addr) => {
                Err(GbError::Unimplemented("writing to work ram 0".into()))
            }
            MappedAddress::WorkRamActiveBank(addr) => Err(GbError::Unimplemented(
                "writing to work ram active bank".into(),
            )),
            MappedAddress::SpriteAttributeTable(addr) => Err(GbError::Unimplemented(
                "writing to sprite attribute table".into(),
            )),
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

    pub fn read_word(&self, addr: u16, comp: ComponentType) -> Result<u16, GbError> {
        match map_address(addr)? {
            MappedAddress::RomBank0(addr) => {
                if self.boot_rom_lock {
                    Ok(LittleEndian::read_u16(&self.boot_rom[addr as usize..]))
                } else {
                    Err(GbError::Unimplemented("reading from cart bank 0".into()))
                }
            }
            MappedAddress::RomActiveBank(addr) => Err(GbError::Unimplemented(
                "reading from cart active bank".into(),
            )),
            MappedAddress::VideoRam(addr) => {
                if comp == ComponentType::CPU && self.io_registers.lcd_control().display_enable() {
                    return Err(GbError::IllegalOp(
                        "read from vram while lcd is active".into(),
                    ));
                }
                Ok(LittleEndian::read_u16(&self.vram[addr as usize..]))
            }
            MappedAddress::ExternalRam(addr) => {
                Err(GbError::Unimplemented("reading from external ram".into()))
            }
            MappedAddress::WorkRamBank0(addr) => {
                Err(GbError::Unimplemented("reading from work ram 0".into()))
            }
            MappedAddress::WorkRamActiveBank(addr) => Err(GbError::Unimplemented(
                "reading from work ram active bank".into(),
            )),
            MappedAddress::SpriteAttributeTable(addr) => Err(GbError::Unimplemented(
                "reading sprite attribute table".into(),
            )),
            MappedAddress::IORegisters(addr) => {
                Err(GbError::Unimplemented("reading IO registers".into()))
            }
            MappedAddress::HighRam(addr) => Ok(LittleEndian::read_u16(&self.hram[addr as usize..])),
            MappedAddress::InterruptEnableRegister => Err(GbError::Unimplemented(
                "reading interrupt enable register".into(),
            )),
        }
    }

    pub fn cpu_read_byte(&self, addr: u16) -> Result<u8, GbError> {
        self.read_byte(addr, ComponentType::CPU)
    }

    pub fn cpu_write_byte(&mut self, addr: u16, val: u8) -> Result<(), GbError> {
        self.write_byte(addr, val, ComponentType::CPU)
    }

    pub fn cpu_read_word(&self, addr: u16) -> Result<u16, GbError> {
        self.read_word(addr, ComponentType::CPU)
    }

    pub fn ppu_read_byte(&self, addr: u16) -> Result<u8, GbError> {
        self.read_byte(addr, ComponentType::PPU)
    }

    pub fn ppu_write_byte(&mut self, addr: u16, val: u8) -> Result<(), GbError> {
        self.write_byte(addr, val, ComponentType::PPU)
    }

    pub fn ppu_read_word(&self, addr: u16) -> Result<u16, GbError> {
        self.read_word(addr, ComponentType::PPU)
    }
}
