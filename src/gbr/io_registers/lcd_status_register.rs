#[derive(Default, Debug, Clone, Copy)]
pub struct LcsStatusRegister {
    reg_value: u8,
}

pub enum ScreenMode {
    HBlank,
    VBlank,
    SreachingOAM,
    TransferringData,
}

impl LcsStatusRegister {
    pub fn raw(&self) -> u8 {
        // Bit 7 is always one, so mask it when
        // returning raw register
        self.reg_value | 0b10000000
    }

    pub fn lyc_check_enable(&self) -> bool {
        self.reg_value & 0b01000000 != 0
    }

    pub fn mode_2_oam_check_enable(&self) -> bool {
        self.reg_value & 0b00100000 != 0
    }

    pub fn mode_1_vblank_check_enable(&self) -> bool {
        self.reg_value & 0b00010000 != 0
    }

    pub fn mode_0_hblank_check_enable(&self) -> bool {
        self.reg_value & 0b00001000 != 0
    }

    pub fn lyc_equals_ly(&self) -> bool {
        self.reg_value & 0b00000100 != 0
    }

    pub fn screen_mode(&self) -> ScreenMode {
        match self.reg_value & 0b00000011 {
            0 => ScreenMode::HBlank,
            1 => ScreenMode::VBlank,
            2 => ScreenMode::SreachingOAM,
            _ => ScreenMode::TransferringData,
        }
    }
}

impl From<u8> for LcsStatusRegister {
    fn from(value: u8) -> Self {
        LcsStatusRegister { reg_value: value }
    }
}
