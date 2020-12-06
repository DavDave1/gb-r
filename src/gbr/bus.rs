use std::fs;

use byteorder::{ByteOrder, LittleEndian};

use crate::gbr::instruction::Instruction;
use crate::gbr::io_registers::IORegisters;
use crate::gbr::memory_map::*;

#[derive(Default)]
struct LcdControlRegister {
    lcd_display_enable: bool,
    window_tile_map_display_select: bool,
    window_display_enable: bool,
    bg_and_window_tile_data_select: bool,
    bg_tile_map_display_select: bool,
    sprite_size_enable: bool,
    sprite_display_enable: bool,
    bg_window_display_priority: bool,
}

pub struct Bus {
    boot_rom_lock: bool,
    boot_rom: Box<[u8]>,
    cart_rom: Box<[u8]>,
    vram: Box<[u8]>,
    hram: Box<[u8]>,
    lcd_control_reg: LcdControlRegister,
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
            lcd_control_reg: LcdControlRegister::default(),
            io_registers: IORegisters::default(),
        }
    }

    pub fn fetch_instruction(&self, addr: u16) -> Instruction {
        match map_address(addr) {
            MappedAddress::RomBank0(addr) => {
                if self.boot_rom_lock {
                    Instruction::new(&self.boot_rom[addr as usize..addr as usize + 3])
                } else {
                    panic!("Fetching instruction from cart not implemented");
                }
            }
            _ => panic!("Fetching instruction ouside rom bank 0 not implemented"),
        }
    }
    pub fn read_byte(&self, addr: u16) -> u8 {
        match map_address(addr) {
            MappedAddress::RomBank0(addr) => {
                if self.boot_rom_lock && (addr as usize) < BOOT_ROM_SIZE {
                    self.boot_rom[addr as usize]
                } else {
                    self.cart_rom[addr as usize]
                }
            }
            MappedAddress::RomActiveBank(addr) => {
                panic!("Reading from cart active bank not implemented")
            }
            MappedAddress::VideoRam(addr) => {
                if self.lcd_control_reg.lcd_display_enable == false {
                    self.vram[addr as usize]
                } else {
                    panic!("Cannot read vram whle lcd is active")
                }
            }
            MappedAddress::ExternalRam(addr) => {
                panic!("Reading from external ram not implemented")
            }
            MappedAddress::WorkRamBank0(addr) => {
                panic!("Reading from work ram 0 not implemented")
            }
            MappedAddress::WorkRamActiveBank(addr) => {
                panic!("Reading from work ram active bank not implemented")
            }
            MappedAddress::SpriteAttributeTable(addr) => {
                panic!("Reading sprite attribute table not implemented")
            }
            MappedAddress::IORegisters(addr) => {
                panic!("Reading from IO registers not implemented")
            }
            MappedAddress::HighRam(addr) => self.hram[addr as usize],
            MappedAddress::InterruptEnableRegister => {
                panic!("Reading interrupt enable register not implemented")
            }
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match map_address(addr) {
            MappedAddress::RomBank0(addr) => panic!("Cannot write to rom bank 0"),
            MappedAddress::RomActiveBank(addr) => panic!("Cannot write to rom active bank"),
            MappedAddress::VideoRam(addr) => {
                if self.lcd_control_reg.lcd_display_enable == false {
                    self.vram[addr as usize] = value;
                } else {
                    panic!("Cannot write vram whle lcd is active")
                }
            }
            MappedAddress::ExternalRam(addr) => {
                panic!("Writing to external ram not implemented")
            }
            MappedAddress::WorkRamBank0(addr) => {
                panic!("Writing towork ram 0 not implemented")
            }
            MappedAddress::WorkRamActiveBank(addr) => {
                panic!("Writing to work ram active bank not implemented")
            }
            MappedAddress::SpriteAttributeTable(addr) => {
                panic!("Writing toattribute table not implemented")
            }
            MappedAddress::IORegisters(addr) => self.io_registers.write(addr, value),
            MappedAddress::HighRam(addr) => {
                self.hram[addr as usize] = value;
            }
            MappedAddress::InterruptEnableRegister => {
                panic!("Writing interrupt enable register not implemented")
            }
        }
        // self.data[addr as usize] = value;
    }

    pub fn read_word(&self, addr: u16) -> u16 {
        match map_address(addr) {
            MappedAddress::RomBank0(addr) => {
                if self.boot_rom_lock {
                    LittleEndian::read_u16(&self.boot_rom[addr as usize..])
                } else {
                    panic!("Rading from cart bank 0 not implemented");
                }
            }
            MappedAddress::RomActiveBank(addr) => {
                panic!("Reading from cart active bank not implemented")
            }
            MappedAddress::VideoRam(addr) => {
                if self.lcd_control_reg.lcd_display_enable == false {
                    LittleEndian::read_u16(&self.vram[addr as usize..])
                } else {
                    panic!("Cannot read vram whle lcd is active")
                }
            }
            MappedAddress::ExternalRam(addr) => {
                panic!("Reading from external ram not implemented")
            }
            MappedAddress::WorkRamBank0(addr) => {
                panic!("Reading from work ram 0 not implemented")
            }
            MappedAddress::WorkRamActiveBank(addr) => {
                panic!("Reading from work ram active bank not implemented")
            }
            MappedAddress::SpriteAttributeTable(addr) => {
                panic!("Reading sprite attribute table not implemented")
            }
            MappedAddress::IORegisters(addr) => {
                panic!("Reading from IO registers not implemented")
            }
            MappedAddress::HighRam(addr) => LittleEndian::read_u16(&self.hram[addr as usize..]),
            MappedAddress::InterruptEnableRegister => {
                panic!("Reading interrupt enable register not implemented")
            }
        }
    }
}
