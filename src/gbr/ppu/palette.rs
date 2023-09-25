use super::Rgba;

#[derive(Copy, Clone)]
pub enum GrayShade {
    White,
    LightGray,
    DarkGray,
    Black,
}

impl GrayShade {
    pub fn to_rgba(&self) -> Rgba {
        match self {
            GrayShade::Black => Rgba::black(),
            GrayShade::DarkGray => Rgba::dark(),
            GrayShade::LightGray => Rgba::light(),
            GrayShade::White => Rgba::white(),
        }
    }
}

impl From<u8> for GrayShade {
    fn from(value: u8) -> Self {
        match value & 0b00000011 {
            0 => GrayShade::White,
            1 => GrayShade::LightGray,
            2 => GrayShade::DarkGray,
            3 => GrayShade::Black,
            _ => panic!("Cannot convert {} to GrayShade", value),
        }
    }
}

impl From<GrayShade> for u8 {
    fn from(value: GrayShade) -> Self {
        match value {
            GrayShade::White => 0,
            GrayShade::LightGray => 1,
            GrayShade::DarkGray => 2,
            GrayShade::Black => 3,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Palette {
    indexed: [GrayShade; 4],
    rgba: [Rgba; 4],
}

impl Palette {
    pub fn new(color0: GrayShade, color1: GrayShade, color2: GrayShade, color3: GrayShade) -> Self {
        Self {
            indexed: [color0, color1, color2, color3],
            rgba: [
                color0.to_rgba(),
                color1.to_rgba(),
                color2.to_rgba(),
                color3.to_rgba(),
            ],
        }
    }
    pub fn indexed(&self, color_id: u8) -> GrayShade {
        self.indexed[color_id as usize]
    }

    pub fn rgba(&self, color_id: u8) -> &Rgba {
        &self.rgba[color_id as usize]
    }
}

impl Default for Palette {
    fn default() -> Self {
        Palette {
            indexed: [GrayShade::White; 4],
            rgba: [Rgba::default(); 4],
        }
    }
}

impl From<u8> for Palette {
    fn from(value: u8) -> Self {
        let mask: u8 = 0b0000011;
        Palette::new(
            GrayShade::from(value & mask),
            GrayShade::from(value >> 2 & mask),
            GrayShade::from(value >> 4 & mask),
            GrayShade::from(value >> 6 & mask),
        )
    }
}

impl From<Palette> for u8 {
    fn from(value: Palette) -> Self {
        value.indexed[0] as u8
            | (value.indexed[1] as u8) << 2
            | (value.indexed[2] as u8) << 4
            | (value.indexed[3] as u8) << 6
    }
}
