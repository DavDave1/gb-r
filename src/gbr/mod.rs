pub mod apu;
pub mod bus;
pub mod cart_header;
pub mod cpu;
pub mod dma;
pub mod game_boy;
pub mod instruction;
pub mod interrupts;
pub mod joypad;
pub mod mbc;
pub mod memory_map;
pub mod oam;
pub mod ppu;
pub mod timer;

mod alu;
mod serial;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum GbError {
    #[error("Unknown instruction {0:#04X}")]
    UnknownInstruction(u8),
    #[error("Unknown cb instruction {0:#04X}")]
    UnknownCbInstruction(u8),
    #[error("Unimplemented feature: {0}")]
    Unimplemented(String),
    #[error("Illegal operation: {0}")]
    IllegalOp(String),
    #[error("Address {0:#06X} out of bounds")]
    AddrOutOfBounds(u16),
    #[error("MBC Address {0:#06X} out of bounds")]
    MbcAddrOutOfBounds(u16),
    #[error("Header parsing: {0}")]
    HeaderParsing(String),
}
