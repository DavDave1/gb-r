pub mod background_palette;
pub mod lcd_control_register;
pub mod lcd_status_register;

use crate::gbr::memory_map::*;

use super::GbError;

#[derive(Default, Clone, Copy)]
pub struct IORegisters {
    port_p1: u8,
    serial_data: u8,
    serial_control: u8,
    sound_enable: u8,
    sound_channel_1_wave_pattern_length: u8,
    sound_channel_1_volume_envelope: u8,
    sound_output_terminal_selection: u8,
    sound_channel_volume_control: u8,
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

    pub fn sound_enable(&self) -> u8 {
        self.sound_enable
    }

    pub fn sound_ch1_wave_pattern_length(&self) -> u8 {
        self.sound_channel_1_wave_pattern_length
    }

    pub fn sound_ch1_volume_envelope(&self) -> u8 {
        self.sound_channel_1_volume_envelope
    }

    pub fn sound_output_terminal_selection(&self) -> u8 {
        self.sound_output_terminal_selection
    }

    pub fn sound_channel_volume_control(&self) -> u8 {
        self.sound_channel_volume_control
    }

    pub fn write(&mut self, addr: u16, value: u8) -> Result<(), GbError> {
        match addr {
            0x0000 => Ok(self.port_p1 = value),
            0x0001 => Ok(self.serial_data = value),
            0x0002 => Ok(self.serial_control = value),
            0x0011 => Ok(self.sound_channel_1_wave_pattern_length = value),
            0x0012 => Ok(self.sound_channel_1_volume_envelope = value),
            0x0024 => Ok(self.sound_channel_volume_control = value),
            0x0025 => Ok(self.sound_output_terminal_selection = value),
            0x0026 => {
                if value & 0x7F != 0 {
                    Err(GbError::IllegalOp(format!(
                        "attempting to write {:#04X} to sound enable register (NR52)",
                        value
                    )))
                } else {
                    Ok(self.sound_enable = value)
                }
            }
            _ => Err(GbError::Unimplemented(format!(
                "write to io register {:#06X}",
                addr + IO_REGISTERS_START
            ))),
        }
    }

    pub fn read(&self, addr: u16) -> Result<u8, GbError> {
        match addr {
            0x0000 => Ok(self.port_p1),
            0x0001 => Ok(self.serial_data),
            0x0002 => Ok(self.serial_control),
            _ => Err(GbError::Unimplemented(format!(
                "read from io register {:#06X}",
                addr + IO_REGISTERS_START
            ))),
        }
    }
}
