use log::error;
use std::fs;

use byteorder::{ByteOrder, LittleEndian};

use crate::gbr::instruction::Instruction;
use crate::gbr::io_registers::IORegisters;
use crate::gbr::memory_map::*;

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

    pub fn fetch_instruction(&self, addr: u16) -> Result<Instruction, ()> {
        match map_address(addr)? {
            MappedAddress::RomBank0(addr) => {
                if self.boot_rom_lock {
                    Ok(Instruction::new(
                        &self.boot_rom[addr as usize..addr as usize + 3],
                    ))
                } else {
                    error!("Fetching instruction from cart not implemented");
                    Err(())
                }
            }
            _ => {
                error!("Fetching instruction ouside rom bank 0 not implemented");
                Err(())
            }
        }
    }
    pub fn read_byte(&self, addr: u16) -> Result<u8, ()> {
        match map_address(addr)? {
            MappedAddress::RomBank0(addr) => {
                if self.boot_rom_lock && (addr as usize) < BOOT_ROM_SIZE {
                    Ok(self.boot_rom[addr as usize])
                } else {
                    Ok(self.cart_rom[addr as usize])
                }
            }
            MappedAddress::RomActiveBank(addr) => {
                error!("Reading from cart active bank not implemented");
                Err(())
            }
            MappedAddress::VideoRam(addr) => {
                if self.io_registers.lcd_control().display_enable() == false {
                    Ok(self.vram[addr as usize])
                } else {
                    error!("Cannot read vram whle lcd is active");
                    Err(())
                }
            }
            MappedAddress::ExternalRam(addr) => {
                error!("Reading from external ram not implemented");
                Err(())
            }
            MappedAddress::WorkRamBank0(addr) => {
                error!("Reading from work ram 0 not implemented");
                Err(())
            }
            MappedAddress::WorkRamActiveBank(addr) => {
                error!("Reading from work ram active bank not implemented");
                Err(())
            }
            MappedAddress::SpriteAttributeTable(addr) => {
                error!("Reading sprite attribute table not implemented");
                Err(())
            }
            MappedAddress::IORegisters(addr) => self.io_registers.read(addr),
            MappedAddress::HighRam(addr) => Ok(self.hram[addr as usize]),
            MappedAddress::InterruptEnableRegister => {
                error!("Reading interrupt enable register not implemented");
                Err(())
            }
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) -> Result<(), ()> {
        match map_address(addr)? {
            MappedAddress::RomBank0(addr) => {
                error!("Cannot write to rom bank 0");
                Err(())
            }
            MappedAddress::RomActiveBank(addr) => {
                error!("Cannot write to rom active bank");
                Err(())
            }
            MappedAddress::VideoRam(addr) => {
                if self.io_registers.lcd_control().display_enable() == false {
                    self.vram[addr as usize] = value;
                    Ok(())
                } else {
                    error!("Cannot write vram whle lcd is active");
                    Err(())
                }
            }
            MappedAddress::ExternalRam(addr) => {
                error!("Writing to external ram not implemented");
                Err(())
            }
            MappedAddress::WorkRamBank0(addr) => {
                error!("Writing towork ram 0 not implemented");
                Err(())
            }
            MappedAddress::WorkRamActiveBank(addr) => {
                error!("Writing to work ram active bank not implemented");
                Err(())
            }
            MappedAddress::SpriteAttributeTable(addr) => {
                error!("Writing toattribute table not implemented");
                Err(())
            }
            MappedAddress::IORegisters(addr) => {
                self.io_registers.write(addr, value);
                Ok(())
            }
            MappedAddress::HighRam(addr) => {
                self.hram[addr as usize] = value;
                Ok(())
            }
            MappedAddress::InterruptEnableRegister => {
                error!("Writing interrupt enable register not implemented");
                Err(())
            }
        }
    }

    pub fn read_word(&self, addr: u16) -> Result<u16, ()> {
        match map_address(addr)? {
            MappedAddress::RomBank0(addr) => {
                if self.boot_rom_lock {
                    Ok(LittleEndian::read_u16(&self.boot_rom[addr as usize..]))
                } else {
                    error!("Reading from cart bank 0 not implemented");
                    Err(())
                }
            }
            MappedAddress::RomActiveBank(addr) => {
                error!("Reading from cart active bank not implemented");
                Err(())
            }
            MappedAddress::VideoRam(addr) => {
                if self.io_registers.lcd_control().display_enable() == false {
                    Ok(LittleEndian::read_u16(&self.vram[addr as usize..]))
                } else {
                    error!("Cannot read vram whle lcd is active");
                    Err(())
                }
            }
            MappedAddress::ExternalRam(addr) => {
                error!("Reading from external ram not implemented");
                Err(())
            }
            MappedAddress::WorkRamBank0(addr) => {
                error!("Reading from work ram 0 not implemented");
                Err(())
            }
            MappedAddress::WorkRamActiveBank(addr) => {
                error!("Reading from work ram active bank not implemented");
                Err(())
            }
            MappedAddress::SpriteAttributeTable(addr) => {
                error!("Reading sprite attribute table not implemented");
                Err(())
            }
            MappedAddress::IORegisters(addr) => {
                error!("Reading from IO registers not implemented");
                Err(())
            }
            MappedAddress::HighRam(addr) => Ok(LittleEndian::read_u16(&self.hram[addr as usize..])),
            MappedAddress::InterruptEnableRegister => {
                error!("Reading interrupt enable register not implemented");
                Err(())
            }
        }
    }
}
