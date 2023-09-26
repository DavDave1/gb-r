use std::path::Path;

use byteorder::{ByteOrder, LittleEndian};

use super::{
    cart_header::{CartHeader, MapperType},
    memory_map::{
        CART_RAM_END, CART_RAM_START, CART_ROM_ACTIVE_BANK_END, CART_ROM_ACTIVE_BANK_START,
        CART_ROM_BANK0_END, CART_ROM_BANK0_START,
    },
    GbError,
};

const ROM_BANK_SIZE: usize = 0x4000;
const RAM_BANK_SIZE: usize = 0x2000;

const RAM_ENABLE_REG_START: u16 = 0x0000;
const RAM_ENABLE_REG_END: u16 = 0x1FFF;

const BANK_REG1_START: u16 = 0x2000;
const BANK_REG1_END: u16 = 0x3FFF;

const BANK_REG2_START: u16 = 0x4000;
const BANK_REG2_END: u16 = 0x5FFF;

const BANK_MODE_START: u16 = 0x6000;
const BANK_MODE_END: u16 = 0x7FFF;

const RAM_ENABLE_NUMBER: u8 = 0x0A;

#[derive(Default, Clone, Copy)]
pub struct MbcState {
    pub mbc_type: MapperType,
    pub active_rom_bank: u16,
    pub active_ram_bank: u16,
    pub rom_banks_count: u16,
    pub ram_banks_count: u16,
    pub ram_enable: bool,
    pub banking_mode: BankingMode,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum BankingMode {
    #[default]
    Simple,
    Advanced,
}

#[derive(Default)]
pub struct MBC {
    mbc_type: MapperType,
    rom: Box<[u8]>,
    ram: Box<[u8]>,
    rom_banks_count: u16,
    ram_banks_count: u16,
    active_rom_bank: u16,
    active_ram_bank: u16,
    ram_enable: bool,
    banking_mode: BankingMode,
}

impl MBC {
    pub fn new(rom_path: &Path) -> Result<MBC, GbError> {
        let rom = std::fs::read(rom_path)
            .map_err(|e| GbError::HeaderParsing(format!("failed to parse rom {}", e)))?;

        let header = CartHeader::parse(&rom)?;

        log::info!("{:#?} ", header);

        let header_rom_size = ROM_BANK_SIZE * header.rom_banks() as usize;

        if header_rom_size < rom.len() {
            return Err(GbError::HeaderParsing(format!("invalid rom size")));
        }

        let mut mbc_rom = vec![0; ROM_BANK_SIZE * header.rom_banks() as usize].into_boxed_slice();
        mbc_rom.copy_from_slice(&rom);

        Ok(Self {
            mbc_type: header.mapper_type(),
            rom: mbc_rom,
            ram: vec![0; RAM_BANK_SIZE * header.ram_banks() as usize].into_boxed_slice(),
            rom_banks_count: header.rom_banks(),
            ram_banks_count: header.ram_banks(),
            active_rom_bank: 1,
            active_ram_bank: 0,
            ram_enable: false,
            banking_mode: BankingMode::Simple,
        })
    }

    pub fn read_byte(&self, addr: u16) -> Result<u8, GbError> {
        match addr {
            CART_ROM_BANK0_START..=CART_ROM_BANK0_END => Ok(self.rom[addr as usize]),
            CART_ROM_ACTIVE_BANK_START..=CART_ROM_ACTIVE_BANK_END => {
                Ok(self.rom[self.rom_relative_addr(addr)])
            }
            CART_RAM_START..=CART_RAM_END => {
                let val = if self.ram_enable {
                    self.ram[self.ram_relative_addr(addr)]
                } else {
                    0xFF
                };
                Ok(val)
            }
            _ => Err(GbError::AddrOutOfBounds(addr)),
        }
    }

    pub fn write_byte(&mut self, addr: u16, byte: u8) -> Result<(), GbError> {
        match addr {
            RAM_ENABLE_REG_START..=RAM_ENABLE_REG_END => {
                log::debug!("Writing {} to ram enable reg", byte);
                self.ram_enable = (byte & 0b1111) == RAM_ENABLE_NUMBER;
            }
            BANK_REG1_START..=BANK_REG1_END => {
                let mut v = byte & 0b11111;
                if v == 0 {
                    v = 1;
                }

                self.active_rom_bank = self.active_rom_bank & 0b0000000011100000 + v as u16;
            }
            BANK_REG2_START..=BANK_REG2_END => {
                self.active_rom_bank =
                    self.active_rom_bank & 0b0000000000011111 + (byte & 0b01100000) as u16;
                if self.ram_banks_count > 1 {
                    self.active_ram_bank = (byte & 0b01100000 >> 5) as u16;
                }
            }
            BANK_MODE_START..=BANK_MODE_END => {
                if byte & 0b1 == 0 {
                    self.banking_mode = BankingMode::Simple;
                } else {
                    self.banking_mode = BankingMode::Advanced
                }
            }
            CART_RAM_START..=CART_RAM_END => {
                if self.ram_enable {
                    self.ram[self.ram_relative_addr(addr)] = byte;
                }
            }
            _ => return Err(GbError::AddrOutOfBounds(addr)),
        }

        Ok(())
    }

    pub fn read_word(&self, addr: u16) -> Result<u16, GbError> {
        match addr {
            CART_ROM_BANK0_START..=CART_ROM_BANK0_END => {
                Ok(LittleEndian::read_u16(&self.rom[addr as usize..]))
            }
            CART_ROM_ACTIVE_BANK_START..=CART_ROM_ACTIVE_BANK_END => Ok(LittleEndian::read_u16(
                &self.rom[self.rom_relative_addr(addr)..],
            )),
            CART_RAM_START..=CART_RAM_END => {
                let val = if self.ram_enable {
                    LittleEndian::read_u16(&self.ram[self.ram_relative_addr(addr)..])
                } else {
                    0xFFFF
                };
                Ok(val)
            }
            _ => Err(GbError::AddrOutOfBounds(addr)),
        }
    }

    fn ram_relative_addr(&self, abs_addr: u16) -> usize {
        (abs_addr - CART_RAM_START) as usize + self.active_ram_bank as usize * RAM_BANK_SIZE
    }

    fn rom_relative_addr(&self, abs_addr: u16) -> usize {
        (abs_addr - CART_ROM_BANK0_START) as usize + self.active_rom_bank as usize * ROM_BANK_SIZE
    }

    pub fn state(&self) -> MbcState {
        MbcState {
            mbc_type: self.mbc_type,
            active_rom_bank: self.active_rom_bank,
            active_ram_bank: self.active_ram_bank,
            rom_banks_count: self.rom_banks_count,
            ram_banks_count: self.ram_banks_count,
            ram_enable: self.ram_enable,
            banking_mode: self.banking_mode,
        }
    }
}
