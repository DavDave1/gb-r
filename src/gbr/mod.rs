pub mod apu;
pub mod bus;
pub mod cpu;
pub mod game_boy;
pub mod instruction;
pub mod interrupts;
pub mod io_registers;
pub mod memory_map;
pub mod ppu;
pub mod timer;

mod alu;

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
}
