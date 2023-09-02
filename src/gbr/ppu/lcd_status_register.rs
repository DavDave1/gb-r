use std::fmt::Display;

#[derive(Default, Debug, Clone, Copy)]
pub enum ScreenMode {
    HBlank,
    VBlank,
    #[default]
    SreachingOAM,
    TransferringData,
}

impl From<u8> for ScreenMode {
    fn from(value: u8) -> Self {
        match value & 0b00000011 {
            0 => Self::HBlank,
            1 => Self::VBlank,
            2 => Self::SreachingOAM,
            3 => Self::TransferringData,
            _ => panic!("Cannot convert {} to ScreenMode", value),
        }
    }
}

impl From<ScreenMode> for u8 {
    fn from(value: ScreenMode) -> Self {
        match value {
            ScreenMode::HBlank => 0,
            ScreenMode::VBlank => 1,
            ScreenMode::SreachingOAM => 2,
            ScreenMode::TransferringData => 3,
        }
    }
}

impl Display for ScreenMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HBlank => write!(f, "HBlank"),
            Self::VBlank => write!(f, "VBlank"),
            Self::SreachingOAM => write!(f, "OAM"),
            Self::TransferringData => write!(f, "Trasf"),
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct LcsStatusRegister {
    pub lyc_check_enable: bool,
    pub lyc_equals_ly: bool,
    pub mode_2_check_enable: bool,
    pub mode_1_check_enable: bool,
    pub mode_0_check_enable: bool,
    pub mode: ScreenMode,
}

impl From<u8> for LcsStatusRegister {
    fn from(value: u8) -> Self {
        LcsStatusRegister {
            lyc_check_enable: value & 0b01000000 != 0,
            lyc_equals_ly: value & 0b00100000 != 0,
            mode_2_check_enable: value & 0b00010000 != 0,
            mode_1_check_enable: value & 0b00001000 != 0,
            mode_0_check_enable: value & 0b00000100 != 0,
            mode: (value & 0b00000011).into(),
        }
    }
}

impl From<LcsStatusRegister> for u8 {
    fn from(value: LcsStatusRegister) -> Self {
        (value.lyc_check_enable as u8) << 6
            | (value.lyc_equals_ly as u8) << 5
            | (value.mode_2_check_enable as u8) << 4
            | (value.mode_1_check_enable as u8) << 3
            | (value.mode_0_check_enable as u8) << 2
            | value.mode as u8
    }
}

fn flag_to_str(flag: bool) -> &'static str {
    if flag {
        "T"
    } else {
        "F"
    }
}

impl Display for LcsStatusRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mode: {}\n", self.mode)?;
        write!(
            f,
            "LYC: enabled {} equals {}\n",
            flag_to_str(self.lyc_check_enable),
            flag_to_str(self.lyc_equals_ly)
        )?;
        write!(
            f,
            "Interrupts: mode2 {} mode1 {} mode0 {}",
            flag_to_str(self.mode_2_check_enable),
            flag_to_str(self.mode_2_check_enable),
            flag_to_str(self.mode_1_check_enable)
        )
    }
}
