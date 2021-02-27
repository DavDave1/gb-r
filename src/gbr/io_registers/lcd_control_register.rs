#[derive(Default, Debug)]
pub struct LcdControlRegister {
    reg_value: u8,
}

impl LcdControlRegister {
    pub fn display_enable(&self) -> bool {
        self.reg_value & 0b10000000 != 0
    }

    pub fn window_tile_map_display_select(&self) -> bool {
        self.reg_value & 0b01000000 != 0
    }

    pub fn window_display_enable(&self) -> bool {
        self.reg_value & 0b00100000 != 0
    }

    pub fn bg_and_window_tile_data_select(&self) -> bool {
        self.reg_value & 0b00010000 != 0
    }

    pub fn bg_tile_map_display_select(&self) -> bool {
        self.reg_value & 0b00001000 != 0
    }

    pub fn sprite_size_enable(&self) -> bool {
        self.reg_value & 0b00000100 != 0
    }

    pub fn sprite_display_enable(&self) -> bool {
        self.reg_value & 0b00000010 != 0
    }

    pub fn bg_window_display_priority(&self) -> bool {
        self.reg_value & 0b00000010 != 0
    }

    pub fn raw(&self) -> u8 {
        self.reg_value
    }
}

impl From<u8> for LcdControlRegister {
    fn from(value: u8) -> Self {
        LcdControlRegister { reg_value: value }
    }
}
