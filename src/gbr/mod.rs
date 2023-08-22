pub mod bus;
pub mod cpu;
pub mod game_boy;
pub mod instruction;
pub mod io_registers;
pub mod ppu;

mod alu;
mod memory_map;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum GbError {
    #[error("Failed to decode instruction")]
    Decode,
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
