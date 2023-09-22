mod sound_channel;

use sound_channel::{Channel1, Channel2, Channel3, Channel4};

use super::GbError;

pub struct APU {
    sound_enable: u8,
    sound_output_terminal_selection: u8,
    sound_channel_volume_control: u8,
    ch1: Channel1,
    ch2: Channel2,
    ch3: Channel3,
    ch4: Channel4,
}

impl APU {
    pub fn new() -> Self {
        Self {
            sound_enable: 0,
            sound_output_terminal_selection: 0,
            sound_channel_volume_control: 0,
            ch1: Default::default(),
            ch2: Default::default(),
            ch3: Default::default(),
            ch4: Default::default(),
        }
    }

    pub fn step(&mut self, _cpu_cycles: u8) -> Result<(), GbError> {
        // TODO
        Ok(())
    }

    pub fn read_reg(&self, addr: u16) -> Result<u8, GbError> {
        match addr {
            _ => Err(GbError::IllegalOp(format!(
                "Write to invalid APU reg {:#06X}",
                addr
            ))),
        }
    }

    pub fn write_reg(&mut self, addr: u16, value: u8) -> Result<(), GbError> {
        match addr {
            0x0011 => self.ch1.write_wave_and_timer(value),
            0x0012 => self.ch1.write_envelope(value),
            0x0013 => self.ch1.write_period_low(value),
            0x0014 => self.ch1.write_period_high_and_ctrl(value),
            0x0024 => self.sound_channel_volume_control = value,
            0x0025 => self.sound_output_terminal_selection = value,
            0x0026 => {
                if value & 0x7F != 0 {
                    return Err(GbError::IllegalOp(format!(
                        "attempting to write {:#04X} to sound enable register (NR52)",
                        value
                    )));
                } else {
                    self.sound_enable = value
                }
            }
            _ => {
                return Err(GbError::IllegalOp(format!(
                    "Write to invalid APU reg {:#06X}",
                    addr
                )))
            }
        }

        Ok(())
    }
}
