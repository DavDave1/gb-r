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
