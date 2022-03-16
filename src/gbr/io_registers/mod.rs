pub mod background_palette;
pub mod lcd_control_register;

use log::error;

use crate::gbr::memory_map::*;

#[derive(Default, Clone, Copy)]
pub struct IORegisters {
    reg_port_p1: u8,
    reg_serial_data: u8,
    reg_serial_control: u8,
    reg_sound_enable: u8,
    reg_sound_channel_1_wave_pattern_length: u8,
    reg_sound_channel_1_volume_envelope: u8,
    reg_sound_output_terminal_selection: u8,
    reg_sound_channel_volume_control: u8,
    reg_bg_palette_data: background_palette::BackgroundPalette,
    reg_lcd_control: lcd_control_register::LcdControlRegister,
    reg_scroll_y: u8,
    reg_y_coordinate: u8,
}

impl IORegisters {
    pub fn port_p1(&self) -> u8 {
        self.reg_port_p1
    }

    pub fn serial_data(&self) -> u8 {
        self.reg_serial_data
    }

    pub fn serial_control(&self) -> u8 {
        self.reg_serial_control
    }

    pub fn sound_enable(&self) -> u8 {
        self.reg_sound_enable
    }

    pub fn sound_ch1_wave_pattern_length(&self) -> u8 {
        self.reg_sound_channel_1_wave_pattern_length
    }

    pub fn sound_ch1_volume_envelope(&self) -> u8 {
        self.reg_sound_channel_1_volume_envelope
    }

    pub fn sound_output_terminal_selection(&self) -> u8 {
        self.reg_sound_output_terminal_selection
    }

    pub fn sound_channel_volume_control(&self) -> u8 {
        self.reg_sound_channel_volume_control
    }

    pub fn bg_palette(&self) -> &background_palette::BackgroundPalette {
        &self.reg_bg_palette_data
    }

    pub fn lcd_control(&self) -> &lcd_control_register::LcdControlRegister {
        &self.reg_lcd_control
    }

    pub fn write(&mut self, addr: u16, value: u8) -> Result<(), ()> {
        match addr {
            0x0000 => Ok(self.reg_port_p1 = value),
            0x0001 => Ok(self.reg_serial_data = value),
            0x0002 => Ok(self.reg_serial_control = value),
            0x0011 => Ok(self.reg_sound_channel_1_wave_pattern_length = value),
            0x0012 => Ok(self.reg_sound_channel_1_volume_envelope = value),
            0x0024 => Ok(self.reg_sound_channel_volume_control = value),
            0x0025 => Ok(self.reg_sound_output_terminal_selection = value),
            0x0026 => {
                if value & 0x7F != 0 {
                    error!("Can only write to sound enable register (NR52) bit 1. Attempting to write {:#04X}", value);
                    Err(())
                } else {
                    Ok(self.reg_sound_enable = value)
                }
            }
            0x0040 => Ok(self.reg_lcd_control = value.into()),
            0x0042 => Ok(self.reg_scroll_y = value),
            0x0047 => Ok(self.reg_bg_palette_data = value.into()),
            _ => {
                error!(
                    "Attempting to write to unimplemented io register {:#06X}",
                    addr + IO_REGISTERS_START
                );
                Err(())
            }
        }
    }

    pub fn read(&self, addr: u16) -> Result<u8, ()> {
        match addr {
            0x0000 => Ok(self.reg_port_p1),
            0x0001 => Ok(self.reg_serial_data),
            0x0002 => Ok(self.reg_serial_control),
            0x0044 => Ok(self.reg_y_coordinate),
            _ => {
                error!(
                    "Attempting to read from unimplemented io register {:#06X}",
                    addr + IO_REGISTERS_START
                );
                Err(())
            }
        }
    }
}
