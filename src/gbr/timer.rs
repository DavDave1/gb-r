use super::GbError;

#[derive(Default, Copy, Clone)]
pub enum ClockSelect {
    #[default]
    OneTo1024,
    OneTo16,
    OneTo64,
    OneTo256,
}

impl From<u8> for ClockSelect {
    fn from(value: u8) -> Self {
        match value & 0b00000011 {
            0 => ClockSelect::OneTo1024,
            1 => ClockSelect::OneTo16,
            2 => ClockSelect::OneTo64,
            3 => ClockSelect::OneTo256,
            _ => panic!("Cannot convert {} to ClockSelect", value),
        }
    }
}

impl From<ClockSelect> for u8 {
    fn from(value: ClockSelect) -> Self {
        match value {
            ClockSelect::OneTo1024 => 0,
            ClockSelect::OneTo16 => 1,
            ClockSelect::OneTo64 => 2,
            ClockSelect::OneTo256 => 3,
        }
    }
}

#[derive(Default)]
pub struct Timer {
    divider: u8,
    counter: u8,
    modulo: u8,
    clock_select: ClockSelect,
    enable: bool,
}

impl Timer {
    pub fn write_reg(&mut self, addr: u16, value: u8) -> Result<(), GbError> {
        match addr {
            0x0004 => self.divider = 0,
            0x0005 => self.counter = value,
            0x0006 => self.modulo = value,
            0x0007 => {
                self.enable = (value & 0b00000100) != 0;
                self.clock_select = (value & 0b00000011).into();
            }
            _ => {
                return Err(GbError::IllegalOp(format!(
                    "Write to timer reg {:#06X}",
                    addr
                )))
            }
        }

        Ok(())
    }

    pub fn read_reg(&self, addr: u16) -> Result<u8, GbError> {
        match addr {
            0x0004 => Ok(self.divider),
            0x0005 => Ok(self.counter),
            0x0006 => Ok(self.modulo),
            0x0007 => Ok((self.enable as u8) << 3 | self.clock_select as u8),
            _ => Err(GbError::IllegalOp(format!(
                "Read from timer reg {:#06X}",
                addr
            ))),
        }
    }
}
