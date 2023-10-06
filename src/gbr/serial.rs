use crate::gbr::interrupts::InterruptType;

use super::{interrupts::InterruptHandler, GbError};

const SERIAL_TRANSFER_DATA_REG_ADDR: u16 = 0xFF01;
const SERIAL_TRANSFER_CTRL_REG_ADDR: u16 = 0xFF02;

#[derive(Default, PartialEq)]
enum ShiftClock {
    #[default]
    External,
    Internal,
}

#[derive(Default)]
pub struct Serial {
    shift_clock: ShiftClock,
    transfer_start: bool,
    data: u8,
    out_buffer: Vec<u8>,
}

impl Serial {
    pub fn write(&mut self, addr: u16, value: u8) -> Result<(), GbError> {
        match addr {
            SERIAL_TRANSFER_DATA_REG_ADDR => self.data = value,
            SERIAL_TRANSFER_CTRL_REG_ADDR => {
                self.transfer_start = (value & 0b10000000) != 0;
                self.shift_clock = if (value & 0b0000001) != 0 {
                    ShiftClock::Internal
                } else {
                    ShiftClock::External
                }
            }
            _ => {
                return Err(GbError::IllegalOp(format!(
                    "Write to unknown serial register {:#06X}",
                    addr
                )))
            }
        }

        Ok(())
    }

    pub fn read(&self, addr: u16) -> Result<u8, GbError> {
        match addr {
            SERIAL_TRANSFER_DATA_REG_ADDR => Ok(self.data),
            SERIAL_TRANSFER_CTRL_REG_ADDR => {
                let mut val = (self.transfer_start as u8) << 7;
                if self.shift_clock == ShiftClock::Internal {
                    val += 1;
                }

                Ok(val)
            }
            _ => {
                return Err(GbError::IllegalOp(format!(
                    "Read from unknown serial register {:#06X}",
                    addr
                )))
            }
        }
    }

    pub fn step(&mut self, _cycles: u8, ir_handler: &mut InterruptHandler) {
        // TODO: implement clock for data transfer
        if self.transfer_start {
            self.out_buffer.push(self.data);
            self.transfer_start = false;
            ir_handler.set(InterruptType::Serial);
        }

        if self.out_buffer.len() > 0 && *self.out_buffer.last().unwrap() as char == '\n' {
            log::info!(
                "Serial out: {}",
                std::str::from_utf8(&self.out_buffer).unwrap()
            );

            self.out_buffer.clear();
        }
    }
}
