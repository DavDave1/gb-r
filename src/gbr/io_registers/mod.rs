use crate::gbr::memory_map::*;

use super::GbError;

#[derive(Default, Clone, Copy)]
pub struct IORegisters {
    port_p1: u8,
    serial_data: u8,
    serial_control: u8,
}

impl IORegisters {
    pub fn port_p1(&self) -> u8 {
        self.port_p1
    }

    pub fn serial_data(&self) -> u8 {
        self.serial_data
    }

    pub fn serial_control(&self) -> u8 {
        self.serial_control
    }

    pub fn write(&mut self, addr: u16, value: u8) -> Result<(), GbError> {
        match addr {
            0xFF00 => Ok(self.port_p1 = value),
            0xFF01 => Ok(self.serial_data = value),
            0xFF02 => Ok(self.serial_control = value),
            0xFF7F => {
                log::warn!("Write to unknown register 0xFF7F");
                Ok(())
            }
            _ => Err(GbError::Unimplemented(format!(
                "write to io register {:#06X}",
                addr
            ))),
        }
    }

    pub fn read(&self, addr: u16) -> Result<u8, GbError> {
        match addr {
            0xFF00 => Ok(self.port_p1),
            0xFF01 => Ok(self.serial_data),
            0xFF02 => Ok(self.serial_control),
            _ => Err(GbError::Unimplemented(format!(
                "read from io register {:#06X}",
                addr
            ))),
        }
    }
}
