use crate::gbr::GbError;

pub const BOOT_ROM_SIZE: usize = 0x100;

pub const CART_ROM_BANK0_START: u16 = 0x0000;
pub const CART_ROM_BANK0_END: u16 = 0x3FFF;
pub const CART_ROM_ACTIVE_BANK_START: u16 = 0x4000;
pub const CART_ROM_ACTIVE_BANK_END: u16 = 0x7FFF;

pub const VRAM_START: u16 = 0x8000;
const VRAM_END: u16 = 0x9FFF;
pub const VRAM_SIZE: usize = (VRAM_END - VRAM_START + 1) as usize;

pub const CART_RAM_START: u16 = 0xA000;
pub const CART_RAM_END: u16 = 0xBFFF;

pub const WRAM_BANK0_START: u16 = 0xC000;
const WRAM_BANK0_END: u16 = 0xCFFF;
pub const WRAM_BANK0_SIZE: usize = (WRAM_BANK0_END - WRAM_BANK0_START + 1) as usize;

pub const WRAM_ACTIVE_BANK_START: u16 = 0xD000;
const WRAM_ACTIVE_BANK_END: u16 = 0xDFFF;
pub const WRAM_ACTIVE_BANK_SIZE: usize =
    (WRAM_ACTIVE_BANK_END - WRAM_ACTIVE_BANK_START + 1) as usize;

const ECHO_RAM_START: u16 = 0xE000;
const ECHO_RAM_END: u16 = 0xFDFF;
const OBJ_ATTRIBUTE_TABLE_START: u16 = 0xFE00;
const OBJ_ATTRIBUTE_TABLE_END: u16 = 0xFE9F;
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

pub const HRAM_START: u16 = 0xFF80;
const HRAM_END: u16 = 0xFFFE;
pub const HRAM_SIZE: usize = (HRAM_END - HRAM_START + 1) as usize;

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
        VRAM_START..=VRAM_END => Ok(MappedAddress::VideoRam(addr)),
        CART_RAM_START..=CART_RAM_END => Ok(MappedAddress::CartRam(addr)),
        WRAM_BANK0_START..=WRAM_BANK0_END => Ok(MappedAddress::WorkRamBank0(addr)),
        WRAM_ACTIVE_BANK_START..=WRAM_ACTIVE_BANK_END => Ok(MappedAddress::WorkRamActiveBank(addr)),
        ECHO_RAM_START..=ECHO_RAM_END => Err(GbError::IllegalOp(format!(
            "access to echo RAM {:#06X}",
            addr
        ))),
        OBJ_ATTRIBUTE_TABLE_START..=OBJ_ATTRIBUTE_TABLE_END => {
            Ok(MappedAddress::SpriteAttributeTable(addr))
        }
        NOT_USABLE_RAM_START..=NOT_USABLE_RAM_END => Ok(MappedAddress::NotUsable(addr)),
        TIMER_REGISTERS_START..=TIMER_REGISTERS_END => Ok(MappedAddress::TimerRegisters(addr)),
        APU_REGISTERS_START..=APU_REGISTERS_END => Ok(MappedAddress::ApuRegisters(addr)),
        PPU_REGISTERS_START..=PPU_REGISTERS_END => Ok(MappedAddress::PpuRegisters(addr)),
        BOOT_ROM_LOCK_REGISTER => Ok(MappedAddress::BootRomLockRegister),
        INTERRUPTS_FLAG_REGISTER => Ok(MappedAddress::InterruptFlagRegister),
        IO_REGISTERS_START..=IO_REGISTERS_END => Ok(MappedAddress::IORegisters(addr)),
        HRAM_START..=HRAM_END => Ok(MappedAddress::HighRam(addr)),
        INTERRUPTS_ENABLE_REGISTER => Ok(MappedAddress::InterruptEnableRegister),
    }
}
