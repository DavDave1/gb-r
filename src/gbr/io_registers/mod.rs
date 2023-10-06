use super::GbError;

#[derive(Default, Clone, Copy)]
pub struct IORegisters {
    port_p1: u8,
}

impl IORegisters {
    pub fn port_p1(&self) -> u8 {
        self.port_p1
    }

    pub fn write(&mut self, addr: u16, value: u8) -> Result<(), GbError> {
        match addr {
            0xFF00 => Ok(self.port_p1 = value),
            _ => {
                log::warn!("Write {:#04X} to unknown IO register {:#06X}", value, addr);
                Ok(())
            }
        }
    }

    pub fn read(&self, addr: u16) -> Result<u8, GbError> {
        match addr {
            0xFF00 => Ok(self.port_p1),
            _ => {
                log::warn!("Read from unknown IO register {:#06X}", addr);
                Ok(0xFF)
            }
        }
    }
}
