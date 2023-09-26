use crate::gbr::GbError;

pub const BOOT_ROM_SIZE: usize = 0x100;

pub const CART_ROM_BANK0_START: u16 = 0x0000;
pub const CART_ROM_BANK0_END: u16 = 0x3FFF;
pub const CART_ROM_ACTIVE_BANK_START: u16 = 0x4000;
pub const CART_ROM_ACTIVE_BANK_END: u16 = 0x7FFF;

pub const VIDEO_RAM_START: u16 = 0x8000;
const VIDEO_RAM_END: u16 = 0x9FFF;
pub const VIDEO_RAM_SIZE: usize = (VIDEO_RAM_END - VIDEO_RAM_START + 1) as usize;

pub const CART_RAM_START: u16 = 0xA000;
pub const CART_RAM_END: u16 = 0xBFFF;

const WORK_RAM_BANK0_START: u16 = 0xC000;
const WORK_RAM_BANK0_END: u16 = 0xCFFF;
pub const WORK_RAM_BANK0_SIZE: usize = (WORK_RAM_BANK0_END - WORK_RAM_BANK0_START + 1) as usize;

const WORK_RAM_ACTIVE_BANK_START: u16 = 0xD000;
const WORK_RAM_ACTIVE_BANK_END: u16 = 0xDFFF;
pub const WORK_RAM_ACTIVE_BANK_SIZE: usize =
    (WORK_RAM_ACTIVE_BANK_END - WORK_RAM_ACTIVE_BANK_START + 1) as usize;

const ECHO_RAM_START: u16 = 0xE000;
const ECHO_RAM_END: u16 = 0xFDFF;
const SPRITE_ATTRIBUTE_TABLE_START: u16 = 0xFE00;
const SPRITE_ATTRIBUTE_TABLE_END: u16 = 0xFE9F;
const NOT_USABLE_RAM_START: u16 = 0xFEA0;
const NOT_USABLE_RAM_END: u16 = 0xFEFF;

pub const IO_REGISTERS_START: u16 = 0xFF00;
const IO_REGISTERS_END: u16 = 0xFF7F;

pub const TIMER_REGISTERS_START: u16 = 0xFF04;
pub const TIMER_REGISTERS_END: u16 = 0xFF07;

pub const APU_REGISTERS_START: u16 = 0xFF10;
pub const APU_REGISTERS_END: u16 = 0xFF3F;

pub const PPU_REGISTERS_START: u16 = 0xFF40;
pub const PPU_REGISTERS_END: u16 = 0xFF4B;

const BOOT_ROM_LOCK_REGISTER: u16 = 0xFF50;

const INTERRUPTS_FLAG_REGISTER: u16 = 0xFF0F;
const INTERRUPTS_ENABLE_REGISTER: u16 = 0xFFFF;

const HIGH_RAM_START: u16 = 0xFF80;
const HIGH_RAM_END: u16 = 0xFFFE;
pub const HIGH_RAM_SIZE: usize = (HIGH_RAM_END - HIGH_RAM_START + 1) as usize;

pub enum MappedAddress {
    CartRom(u16),
    VideoRam(u16),
    CartRam(u16),
    WorkRamBank0(u16),
    WorkRamActiveBank(u16),
    //  EchoRam(u16),
    SpriteAttributeTable(u16),
    NotUsable(u16),
    TimerRegisters(u16),
    ApuRegisters(u16),
    PpuRegisters(u16),
    BootRomLockRegister,
    IORegisters(u16),
    HighRam(u16),
    InterruptFlagRegister,
    InterruptEnableRegister,
}

pub fn map_address(addr: u16) -> Result<MappedAddress, GbError> {
    match addr {
        CART_ROM_BANK0_START..=CART_ROM_ACTIVE_BANK_END => Ok(MappedAddress::CartRom(addr)),
        VIDEO_RAM_START..=VIDEO_RAM_END => Ok(MappedAddress::VideoRam(addr - VIDEO_RAM_START)),
        CART_RAM_START..=CART_RAM_END => Ok(MappedAddress::CartRam(addr)),
        WORK_RAM_BANK0_START..=WORK_RAM_BANK0_END => {
            Ok(MappedAddress::WorkRamBank0(addr - WORK_RAM_BANK0_START))
        }
        WORK_RAM_ACTIVE_BANK_START..=WORK_RAM_ACTIVE_BANK_END => Ok(
            MappedAddress::WorkRamActiveBank(addr - WORK_RAM_ACTIVE_BANK_START),
        ),
        ECHO_RAM_START..=ECHO_RAM_END => Err(GbError::IllegalOp(format!(
            "access to echo RAM {:#06X}",
            addr
        ))),
        SPRITE_ATTRIBUTE_TABLE_START..=SPRITE_ATTRIBUTE_TABLE_END => Ok(
            MappedAddress::SpriteAttributeTable(addr - SPRITE_ATTRIBUTE_TABLE_START),
        ),
        NOT_USABLE_RAM_START..=NOT_USABLE_RAM_END => Ok(MappedAddress::NotUsable(addr)),
        TIMER_REGISTERS_START..=TIMER_REGISTERS_END => {
            Ok(MappedAddress::TimerRegisters(addr - IO_REGISTERS_START))
        }
        APU_REGISTERS_START..=APU_REGISTERS_END => {
            Ok(MappedAddress::ApuRegisters(addr - IO_REGISTERS_START))
        }
        PPU_REGISTERS_START..=PPU_REGISTERS_END => {
            Ok(MappedAddress::PpuRegisters(addr - IO_REGISTERS_START))
        }
        BOOT_ROM_LOCK_REGISTER => Ok(MappedAddress::BootRomLockRegister),
        INTERRUPTS_FLAG_REGISTER => Ok(MappedAddress::InterruptFlagRegister),
        IO_REGISTERS_START..=IO_REGISTERS_END => {
            Ok(MappedAddress::IORegisters(addr - IO_REGISTERS_START))
        }
        HIGH_RAM_START..=HIGH_RAM_END => Ok(MappedAddress::HighRam(addr - HIGH_RAM_START)),
        INTERRUPTS_ENABLE_REGISTER => Ok(MappedAddress::InterruptEnableRegister),
    }
}
