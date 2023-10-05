use crate::gbr::GbError;

pub const BOOT_ROM_SIZE: usize = 0x100;

pub const CART_ROM_BANK0_START: u16 = 0x0000;
pub const CART_ROM_BANK0_END: u16 = 0x3FFF;
pub const CART_ROM_ACTIVE_BANK_START: u16 = 0x4000;
pub const CART_ROM_ACTIVE_BANK_END: u16 = 0x7FFF;

pub const VRAM_START: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9FFF;
pub const VRAM_SIZE: usize = (VRAM_END - VRAM_START + 1) as usize;

pub const CART_RAM_START: u16 = 0xA000;
pub const CART_RAM_END: u16 = 0xBFFF;

pub const WRAM_START: u16 = 0xC000;
const WRAM_END: u16 = 0xDFFF;
pub const WRAM_SIZE: usize = (WRAM_END - WRAM_START + 1) as usize;

const ECHO_RAM_START: u16 = 0xE000;
const ECHO_RAM_END: u16 = 0xFDFF;

pub const OBJ_ATTRIBUTE_TABLE_START: u16 = 0xFE00;
const OBJ_ATTRIBUTE_TABLE_END: u16 = 0xFE9F;
pub const OBJ_ATTRIBUTE_TABLE_SIZE: usize =
    (OBJ_ATTRIBUTE_TABLE_END - OBJ_ATTRIBUTE_TABLE_START + 1) as usize;

const NOT_USABLE_RAM_START: u16 = 0xFEA0;
const NOT_USABLE_RAM_END: u16 = 0xFEFF;

pub const IO_REGISTERS_START: u16 = 0xFF00;
const IO_REGISTERS_END: u16 = 0xFF7F;

pub const TIMER_REGISTERS_START: u16 = 0xFF04;
pub const TIMER_REGISTERS_END: u16 = 0xFF07;

pub const APU_REGISTERS_START: u16 = 0xFF10;
pub const APU_REGISTERS_END: u16 = 0xFF3F;

pub const PPU_REGISTERS_LOW_START: u16 = 0xFF40;
pub const PPU_REGISTERS_LOW_END: u16 = 0xFF45;

pub const DMA_REGISTER: u16 = 0xFF46;

pub const PPU_REGISTERS_HIGH_START: u16 = 0xFF47;
pub const PPU_REGISTERS_HIGH_END: u16 = 0xFF4B;

const BOOT_ROM_LOCK_REGISTER: u16 = 0xFF50;

const INTERRUPTS_FLAG_REGISTER: u16 = 0xFF0F;
pub const INTERRUPTS_ENABLE_REGISTER: u16 = 0xFFFF;

pub const HRAM_START: u16 = 0xFF80;
const HRAM_END: u16 = 0xFFFE;
pub const HRAM_SIZE: usize = (HRAM_END - HRAM_START + 1) as usize;

#[derive(Debug, PartialEq)]
pub enum MappedAddress {
    CartRom(u16),
    VideoRam(u16),
    CartRam(u16),
    WorkRam(u16),
    //  EchoRam(u16),
    ObjectAttributeTable(u16),
    NotUsable(u16),
    TimerRegisters(u16),
    ApuRegisters(u16),
    PpuRegisters(u16),
    DmaRegister,
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
        WRAM_START..=WRAM_END => Ok(MappedAddress::WorkRam(addr)),
        ECHO_RAM_START..=ECHO_RAM_END => Err(GbError::IllegalOp(format!(
            "access to echo RAM {:#06X}",
            addr
        ))),
        OBJ_ATTRIBUTE_TABLE_START..=OBJ_ATTRIBUTE_TABLE_END => {
            Ok(MappedAddress::ObjectAttributeTable(addr))
        }
        NOT_USABLE_RAM_START..=NOT_USABLE_RAM_END => Ok(MappedAddress::NotUsable(addr)),
        TIMER_REGISTERS_START..=TIMER_REGISTERS_END => Ok(MappedAddress::TimerRegisters(addr)),
        APU_REGISTERS_START..=APU_REGISTERS_END => Ok(MappedAddress::ApuRegisters(addr)),
        PPU_REGISTERS_LOW_START..=PPU_REGISTERS_LOW_END => Ok(MappedAddress::PpuRegisters(addr)),
        DMA_REGISTER => Ok(MappedAddress::DmaRegister),
        PPU_REGISTERS_HIGH_START..=PPU_REGISTERS_HIGH_END => Ok(MappedAddress::PpuRegisters(addr)),
        BOOT_ROM_LOCK_REGISTER => Ok(MappedAddress::BootRomLockRegister),
        INTERRUPTS_FLAG_REGISTER => Ok(MappedAddress::InterruptFlagRegister),
        IO_REGISTERS_START..=IO_REGISTERS_END => Ok(MappedAddress::IORegisters(addr)),
        HRAM_START..=HRAM_END => Ok(MappedAddress::HighRam(addr)),
        INTERRUPTS_ENABLE_REGISTER => Ok(MappedAddress::InterruptEnableRegister),
    }
}

#[cfg(test)]
mod tests {
    use crate::gbr::memory_map::MappedAddress;

    use super::map_address;

    #[test]
    fn wram_mapping() {
        let mapped = map_address(0xC000).unwrap();

        assert_eq!(mapped, MappedAddress::WorkRam(0xC000));
    }
}
