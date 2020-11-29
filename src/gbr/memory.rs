use std::fs;
use std::io::{Error, ErrorKind};

use byteorder::{ByteOrder, LittleEndian};

use crate::gbr::memory_map::*;

const BOOT_ROM_SIZE: usize = 256;

pub struct Memory {
    data: [u8; MEMORY_SIZE],
}

impl Memory {
    pub fn load_boot_rom(&mut self, rom_filepath: &std::path::Path) -> Result<(), Error> {
        let boot_rom_data = fs::read(rom_filepath)?;

        if boot_rom_data.len() != BOOT_ROM_SIZE {
            return Err(Error::new(ErrorKind::Other, "wrong boot rom size"));
        }

        self.data[..BOOT_ROM_SIZE].copy_from_slice(&boot_rom_data[..]);
        Ok(())
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        self.data[addr as usize] = value;
    }

    pub fn read_word(&self, addr: u16) -> u16 {
        LittleEndian::read_u16(&self.data[addr as usize..])
    }
}

impl Default for Memory {
    fn default() -> Self {
        Memory {
            data: [0; MEMORY_SIZE],
        }
    }
}
