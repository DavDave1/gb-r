use std::fmt::Display;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
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

#[derive(Default, Clone, Copy, Debug)]
pub struct Signal<Type: PartialEq + Default + Copy> {
    val: Type,
    old_val: Type,
}

impl<Type: PartialEq + Default + Copy> Signal<Type> {
    pub fn set(&mut self, v: Type) {
        self.old_val = self.val;
        self.val = v;
    }

    pub fn get(&self) -> Type {
        self.val
    }

    pub fn changed(&self) -> bool {
        self.val != self.old_val
    }

    pub fn changed_to(&self, to: Type) -> bool {
        self.changed() && self.val == to
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct LcsStatusRegister {
    pub lyc_ir_enable: bool,
    pub mode_2_ir_enable: bool,
    pub mode_1_ir_enable: bool,
    pub mode_0_ir_enable: bool,
    pub lyc_equals_ly: Signal<bool>,
    pub mode: Signal<ScreenMode>,
}

impl LcsStatusRegister {
    pub fn read(&self) -> u8 {
        (self.lyc_ir_enable as u8) << 6
            | (self.mode_2_ir_enable as u8) << 5
            | (self.mode_1_ir_enable as u8) << 4
            | (self.mode_0_ir_enable as u8) << 3
            | (self.lyc_equals_ly.get() as u8) << 2
            | self.mode.get() as u8
    }

    pub fn write(&mut self, v: u8) {
        self.lyc_ir_enable = v & 0b01000000 != 0;
        self.mode_2_ir_enable = v & 0b00100000 != 0;
        self.mode_1_ir_enable = v & 0b00010000 != 0;
        self.mode_0_ir_enable = v & 0b00001000 != 0;
        self.lyc_equals_ly.set(v & 0b00000100 != 0);
        self.mode.set((v & 0b00000011).into());
    }

    pub fn is_mode_0_ir(&self) -> bool {
        self.mode_0_ir_enable && self.mode.changed_to(ScreenMode::SreachingOAM)
    }

    pub fn is_mode_1_ir(&self) -> bool {
        self.mode_1_ir_enable && self.mode.changed_to(ScreenMode::TransferringData)
    }

    pub fn is_mode_2_ir(&self) -> bool {
        self.mode_2_ir_enable && self.mode.changed_to(ScreenMode::HBlank)
    }

    pub fn is_lyc_ir(&self) -> bool {
        self.lyc_ir_enable && self.lyc_equals_ly.changed()
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
        write!(f, "Mode: {}\n", self.mode.get())?;
        write!(
            f,
            "LYC: enabled {} equals {}\n",
            flag_to_str(self.lyc_ir_enable),
            flag_to_str(self.lyc_equals_ly.get())
        )?;
        write!(
            f,
            "Interrupts: mode2 {} mode1 {} mode0 {}",
            flag_to_str(self.mode_2_ir_enable),
            flag_to_str(self.mode_1_ir_enable),
            flag_to_str(self.mode_0_ir_enable)
        )
    }
}
