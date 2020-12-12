use crate::gbr::memory_map::*;

#[derive(Copy, Clone)]
pub enum GrayShade {
    White,
    LightGray,
    DarkGray,
    Black,
}

impl GrayShade {
    pub fn as_ascii(&self) -> char {
        match self {
            GrayShade::White => 176u8 as char,
            GrayShade::LightGray => 177u8 as char,
            GrayShade::DarkGray => 178u8 as char,
            GrayShade::Black => 219u8 as char,
        }
    }
}

impl From<u8> for GrayShade {
    fn from(value: u8) -> Self {
        match value {
            0 => GrayShade::White,
            1 => GrayShade::LightGray,
            2 => GrayShade::DarkGray,
            3 => GrayShade::Black,
            _ => panic!("Cannot convert {:#04X} into GrayShade", value),
        }
    }
}

pub struct BackgroundPalette {
    color_0: GrayShade,
    color_1: GrayShade,
    color_2: GrayShade,
    color_3: GrayShade,
}

impl BackgroundPalette {
    pub fn color_0(&self) -> GrayShade {
        self.color_0
    }

    pub fn color_1(&self) -> GrayShade {
        self.color_1
    }

    pub fn color_2(&self) -> GrayShade {
        self.color_2
    }

    pub fn color_3(&self) -> GrayShade {
        self.color_3
    }
}

impl Default for BackgroundPalette {
    fn default() -> Self {
        BackgroundPalette {
            color_0: GrayShade::White,
            color_1: GrayShade::White,
            color_2: GrayShade::White,
            color_3: GrayShade::White,
        }
    }
}

impl From<u8> for BackgroundPalette {
    fn from(value: u8) -> Self {
        let mask: u8 = 0b0000011;
        BackgroundPalette {
            color_0: GrayShade::from(value & mask),
            color_1: GrayShade::from(value >> 2 & mask),
            color_2: GrayShade::from(value >> 4 & mask),
            color_3: GrayShade::from(value >> 6 & mask),
        }
    }
}

#[derive(Default)]
pub struct IORegisters {
    reg_port_p1: u8,
    reg_serial_data: u8,
    reg_serial_control: u8,
    reg_sound_enable: u8,
    reg_sound_channel_1_wave_pattern_length: u8,
    reg_sound_channel_1_volume_envelope: u8,
    reg_sound_output_terminal_selection: u8,
    reg_sound_channel_volume_control: u8,
    reg_bg_palette_data: BackgroundPalette,
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

    pub fn bg_palette(&self) -> &BackgroundPalette {
        &self.reg_bg_palette_data
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000 => self.reg_port_p1 = value,
            0x0001 => self.reg_serial_data = value,
            0x0002 => self.reg_serial_control = value,
            0x0011 => self.reg_sound_channel_1_wave_pattern_length = value,
            0x0012 => self.reg_sound_channel_1_volume_envelope = value,
            0x0024 => self.reg_sound_channel_volume_control = value,
            0x0025 => self.reg_sound_output_terminal_selection = value,
            0x0026 => {
                if value & 0x7F != 0 {
                    panic!("Can only write to sound enable register (NR52) bit 1. Attempting to write {:#04X}", value)
                } else {
                    self.reg_sound_enable = value
                }
            }
            0x0047 => self.reg_bg_palette_data = value.into(),
            _ => panic!(
                "Attempting to wirte to unimplemented io register {:#06X}",
                addr + IO_REGISTERS_START
            ),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000 => self.reg_port_p1,
            0x0001 => self.reg_serial_data,
            0x0002 => self.reg_serial_control,
            _ => panic!(
                "Attempting to read from unimplemented io register {:#06X}",
                addr + IO_REGISTERS_START
            ),
        }
    }
}
