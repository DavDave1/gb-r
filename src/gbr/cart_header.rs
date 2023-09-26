use std::str::from_utf8;

use super::GbError;

pub const CART_HEADER_START: usize = 0x0100;
pub const CART_HEADER_END: usize = 0x014F;
pub const CART_HEADER_SIZE: usize = CART_HEADER_END - CART_HEADER_START + 1;

const TITLE_START: usize = CART_HEADER_START + 0x0034;
const TITLE_END: usize = CART_HEADER_START + 0x0043;

const MANUFACTORER_CODE_START: usize = CART_HEADER_START + 0x003F;
const MANUFACTORER_CODE_END: usize = CART_HEADER_START + 0x0042;

const CBG_FLAG: usize = CART_HEADER_START + 0x0043;

const NEW_LICENSEE_CODES_START: usize = CART_HEADER_START + 0x0044;
const NEW_LICENSEE_CODES_END: usize = CART_HEADER_START + 0x0045;

const SGB_FLAG: usize = CART_HEADER_START + 0x0046;
const CART_TYPE: usize = CART_HEADER_START + 0x0047;
const ROM_SIZE: usize = CART_HEADER_START + 0x0048;
const RAM_SIZE: usize = CART_HEADER_START + 0x0049;
const DEST_CODE: usize = CART_HEADER_START + 0x004A;
const OLD_LICENSEE_CODE: usize = CART_HEADER_START + 0x004B;
const ROM_VERSION_NUMBER: usize = CART_HEADER_START + 0x004C;

#[derive(Default, Debug, Clone, Copy)]
pub enum MapperType {
    #[default]
    NoMbc,
    Mbc1,
    Mbc2,
    Mbc3,
    Mbc5,
    Mbc6,
    Mbc7,
    Mmm01,
    PocketCamera,
    BandaiTama5,
    HuC1,
    HuC3,
}

#[derive(Default, Debug)]
pub struct CartType {
    mapper_type: MapperType,
    with_ram: bool,
    with_battery: bool,
    with_timer: bool,
    with_rumble: bool,
    with_sensor: bool,
}

impl CartType {
    fn parse(header: u8) -> Result<Self, GbError> {
        let mut cart_type = CartType::default();

        match header {
            0x00 => cart_type.mapper_type = MapperType::NoMbc,
            0x01 => cart_type.mapper_type = MapperType::Mbc1,
            0x02 => {
                cart_type.mapper_type = MapperType::Mbc1;
                cart_type.with_ram = true;
            }
            0x03 => {
                cart_type.mapper_type = MapperType::Mbc1;
                cart_type.with_ram = true;
                cart_type.with_battery = true;
            }
            0x05 => cart_type.mapper_type = MapperType::Mbc2,
            0x06 => {
                cart_type.mapper_type = MapperType::Mbc1;
                cart_type.with_battery = true;
            }
            0x08 => cart_type.with_ram = true,
            0x0B => cart_type.mapper_type = MapperType::Mmm01,
            0x0C => {
                cart_type.mapper_type = MapperType::Mmm01;
                cart_type.with_ram = true;
            }
            0x0D => {
                cart_type.mapper_type = MapperType::Mmm01;
                cart_type.with_ram = true;
                cart_type.with_battery = true;
            }
            0x0F => {
                cart_type.mapper_type = MapperType::Mbc3;
                cart_type.with_battery = true;
                cart_type.with_timer = true;
            }
            0x10 => {
                cart_type.mapper_type = MapperType::Mbc3;
                cart_type.with_ram = true;
                cart_type.with_battery = true;
                cart_type.with_timer = true;
            }
            0x11 => cart_type.mapper_type = MapperType::Mbc3,
            0x12 => {
                cart_type.mapper_type = MapperType::Mbc3;
                cart_type.with_ram = true;
            }
            0x13 => {
                cart_type.mapper_type = MapperType::Mbc3;
                cart_type.with_ram = true;
                cart_type.with_battery = true;
            }
            0x19 => cart_type.mapper_type = MapperType::Mbc5,
            0x1A => {
                cart_type.mapper_type = MapperType::Mbc5;
                cart_type.with_ram = true;
            }
            0x1B => {
                cart_type.mapper_type = MapperType::Mbc5;
                cart_type.with_ram = true;
                cart_type.with_battery = true;
            }
            0x1C => {
                cart_type.mapper_type = MapperType::Mbc5;
                cart_type.with_rumble = true;
            }
            0x1D => {
                cart_type.mapper_type = MapperType::Mbc5;
                cart_type.with_ram = true;
                cart_type.with_rumble = true;
            }
            0x1E => {
                cart_type.mapper_type = MapperType::Mbc5;
                cart_type.with_ram = true;
                cart_type.with_battery = true;
                cart_type.with_rumble = true;
            }
            0x20 => cart_type.mapper_type = MapperType::Mbc6,
            0x27 => {
                cart_type.mapper_type = MapperType::Mbc7;
                cart_type.with_ram = true;
                cart_type.with_battery = true;
                cart_type.with_rumble = true;
                cart_type.with_sensor = true;
            }
            0xFC => cart_type.mapper_type = MapperType::PocketCamera,
            0xFD => cart_type.mapper_type = MapperType::BandaiTama5,
            0xFE => cart_type.mapper_type = MapperType::HuC3,
            0xFF => {
                cart_type.mapper_type = MapperType::HuC1;
                cart_type.with_ram = true;
                cart_type.with_battery = true;
            }
            _ => {
                return Err(GbError::HeaderParsing(format!(
                    "Invalid cart type {:#04X}",
                    header
                )))
            }
        }

        Ok(cart_type)
    }
}

#[derive(Debug)]
pub struct CartHeader {
    title: String,
    manufactorer_code: String,
    old_licensee_code: u8,
    new_licensee_code: [u8; 2],
    destination_code: u8,
    rom_version_number: u8,
    cgb_flag: bool,
    sgb_flag: bool,
    cart_type: CartType,
    rom_banks: u16,
    ram_banks: u16,
}

impl CartHeader {
    pub fn parse(header: &[u8]) -> Result<CartHeader, GbError> {
        if header.len() < CART_HEADER_SIZE {
            return Err(GbError::HeaderParsing("wrong size".into()));
        }

        Ok(CartHeader {
            title: from_utf8(&header[TITLE_START..=TITLE_END])
                .unwrap_or_default()
                .to_owned(),
            manufactorer_code: from_utf8(&header[MANUFACTORER_CODE_START..=MANUFACTORER_CODE_END])
                .unwrap_or_default()
                .to_owned(),
            old_licensee_code: header[OLD_LICENSEE_CODE],
            new_licensee_code: [
                header[NEW_LICENSEE_CODES_START],
                header[NEW_LICENSEE_CODES_END],
            ],
            destination_code: header[DEST_CODE],
            rom_version_number: header[ROM_VERSION_NUMBER],
            cgb_flag: header[CBG_FLAG] == 0xC0,
            sgb_flag: header[SGB_FLAG] == 0x03,
            cart_type: CartType::parse(header[CART_TYPE])?,
            rom_banks: 2 * (1 << header[ROM_SIZE]),
            ram_banks: CartHeader::parse_ram_banks(header[RAM_SIZE])?,
        })
    }

    pub fn mapper_type(&self) -> MapperType {
        self.cart_type.mapper_type
    }

    pub fn rom_banks(&self) -> u16 {
        self.rom_banks
    }

    pub fn ram_banks(&self) -> u16 {
        self.ram_banks
    }

    fn parse_ram_banks(header: u8) -> Result<u16, GbError> {
        match header {
            0x00 => Ok(0),
            0x02 => Ok(1),
            0x03 => Ok(4),
            0x04 => Ok(16),
            0x05 => Ok(8),
            _ => Err(GbError::HeaderParsing(format!(
                "Invalid ram size {:#04X}",
                header
            ))),
        }
    }
}
