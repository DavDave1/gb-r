use std::fmt::Display;

#[derive(Default, Debug, Clone, Copy)]
pub struct LcdControlRegister {
    pub display_enable: bool,
    pub window_tile_area_sel: bool,
    pub window_enable: bool,
    pub bg_and_window_tile_area_sel: bool,
    pub bg_tile_map_area_sel: bool,
    pub obj_size_sel: bool,
    pub obj_enable: bool,
    pub bg_window_priority: bool,
}

impl From<u8> for LcdControlRegister {
    fn from(value: u8) -> Self {
        LcdControlRegister {
            display_enable: value & 0b10000000 != 0,
            window_tile_area_sel: value & 0b01000000 != 0,
            window_enable: value & 0b00100000 != 0,
            bg_and_window_tile_area_sel: value & 0b00010000 != 0,
            bg_tile_map_area_sel: value & 0b00001000 != 0,
            obj_size_sel: value & 0b00000100 != 0,
            obj_enable: value & 0b00000010 != 0,
            bg_window_priority: value & 0b00000001 != 0,
        }
    }
}

impl From<LcdControlRegister> for u8 {
    fn from(value: LcdControlRegister) -> Self {
        (value.display_enable as u8) << 7
            | (value.window_tile_area_sel as u8) << 6
            | (value.window_enable as u8) << 5
            | (value.bg_and_window_tile_area_sel as u8) << 4
            | (value.bg_tile_map_area_sel as u8) << 3
            | (value.obj_size_sel as u8) << 2
            | (value.obj_enable as u8) << 1
            | (value.bg_window_priority as u8)
    }
}

fn flag_to_str(flag: bool) -> &'static str {
    if flag {
        "T"
    } else {
        "F"
    }
}

impl Display for LcdControlRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Disp en: {}, Win tile area: {}, Win en: {}, BG/Win tile area: {}\n",
            flag_to_str(self.display_enable),
            flag_to_str(self.window_tile_area_sel),
            flag_to_str(self.window_enable),
            flag_to_str(self.bg_and_window_tile_area_sel),
        )?;
        write!(
            f,
            "BG tile area: {}, Obj size: {}, Obj en: {}, BG/Win prio: {}",
            flag_to_str(self.bg_tile_map_area_sel),
            flag_to_str(self.obj_size_sel),
            flag_to_str(self.obj_enable),
            flag_to_str(self.bg_window_priority),
        )
    }
}
