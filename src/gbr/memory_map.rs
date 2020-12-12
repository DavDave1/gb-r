use log::error;

pub const MEMORY_SIZE: usize = 0x10000;
pub const BOOT_ROM_SIZE: usize = 0x100;

const ROM_BANK0_START: u16 = 0x0000;
const ROM_BANK0_END: u16 = 0x3FFF;
const ROM_ACTIVE_BANK_START: u16 = 0x4000;
const ROM_ACTIVE_BANK_END: u16 = 0x7FFF;

const VIDEO_RAM_START: u16 = 0x8000;
const VIDEO_RAM_END: u16 = 0x9FFF;
pub const VIDEO_RAM_SIZE: usize = (VIDEO_RAM_END - VIDEO_RAM_START + 1) as usize;

const EXTERNAL_RAM_START: u16 = 0xA000;
const EXTERNAL_RAM_END: u16 = 0xBFFF;
const WORK_RAM_BANK0_START: u16 = 0xC000;
const WORK_RAM_BANK0_END: u16 = 0xCFFF;
const WORK_RAM_ACTIVE_BANK_START: u16 = 0xD000;
const WORK_RAM_ACTIVE_BANK_END: u16 = 0xDFFF;
const ECHO_RAM_START: u16 = 0xE000;
const ECHO_RAM_END: u16 = 0xFDFF;
const SPRITE_ATTRIBUTE_TABLE_START: u16 = 0xFE00;
const SPRITE_ATTRIBUTE_TABLE_END: u16 = 0xFE9F;
const NOT_USABLE_RAM_START: u16 = 0xFEA0;
const NOT_USABLE_RAM_END: u16 = 0xFEFF;
pub const IO_REGISTERS_START: u16 = 0xFF00;
const IO_REGISTERS_END: u16 = 0xFF7F;

const HIGH_RAM_START: u16 = 0xFF80;
const HIGH_RAM_END: u16 = 0xFFFE;
pub const HIGH_RAM_SIZE: usize = (HIGH_RAM_END - HIGH_RAM_START + 1) as usize;

const INTERRUPTS_ENABLE_REGISTER: u16 = 0xFFFF;

pub enum MappedAddress {
    RomBank0(u16),
    RomActiveBank(u16),
    VideoRam(u16),
    ExternalRam(u16),
    WorkRamBank0(u16),
    WorkRamActiveBank(u16),
    //  EchoRam(u16),
    SpriteAttributeTable(u16),
    //  NotUsable(u16),
    IORegisters(u16),
    HighRam(u16),
    InterruptEnableRegister,
}

pub fn map_address(addr: u16) -> Result<MappedAddress, ()> {
    match addr {
        ROM_BANK0_START..=ROM_BANK0_END => Ok(MappedAddress::RomBank0(addr - ROM_BANK0_START)),
        ROM_ACTIVE_BANK_START..=ROM_ACTIVE_BANK_END => {
            Ok(MappedAddress::RomActiveBank(addr - ROM_ACTIVE_BANK_START))
        }
        VIDEO_RAM_START..=VIDEO_RAM_END => Ok(MappedAddress::VideoRam(addr - VIDEO_RAM_START)),
        EXTERNAL_RAM_START..=EXTERNAL_RAM_END => {
            Ok(MappedAddress::ExternalRam(addr - EXTERNAL_RAM_START))
        }
        WORK_RAM_BANK0_START..=WORK_RAM_BANK0_END => {
            Ok(MappedAddress::WorkRamBank0(addr - WORK_RAM_BANK0_END))
        }
        WORK_RAM_ACTIVE_BANK_START..=WORK_RAM_ACTIVE_BANK_END => Ok(
            MappedAddress::WorkRamActiveBank(addr - WORK_RAM_ACTIVE_BANK_START),
        ),
        ECHO_RAM_START..=ECHO_RAM_END => {
            error!("Attempted to access echo RAM {:#06X}", addr);
            Err(())
        }
        SPRITE_ATTRIBUTE_TABLE_START..=SPRITE_ATTRIBUTE_TABLE_END => Ok(
            MappedAddress::SpriteAttributeTable(addr - SPRITE_ATTRIBUTE_TABLE_START),
        ),
        NOT_USABLE_RAM_START..=NOT_USABLE_RAM_END => {
            error!("Attempted to access not usable RAM {:#06X}", addr);
            Err(())
        }
        IO_REGISTERS_START..=IO_REGISTERS_END => {
            Ok(MappedAddress::IORegisters(addr - IO_REGISTERS_START))
        }
        HIGH_RAM_START..=HIGH_RAM_END => Ok(MappedAddress::HighRam(addr - HIGH_RAM_START)),
        INTERRUPTS_ENABLE_REGISTER => Ok(MappedAddress::InterruptEnableRegister),
    }
}
