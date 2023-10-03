#[derive(Default, Copy, Clone)]
pub struct Interrupt {
    pub enabled: bool,
    pub set: bool,
}

pub enum InterruptType {
    VBlank,
    LcdStat,
    Timer,
    Serial,
    Joypad,
}

#[derive(Default, Clone)]
pub struct InterruptHandlerState {
    pub ime: bool,
    pub vblank: Interrupt,
    pub lcd_stat: Interrupt,
    pub timer: Interrupt,
    pub serial: Interrupt,
    pub joypad: Interrupt,
}

#[derive(Default, Clone)]
pub struct InterruptHandler {
    ime: bool,
    vblank: Interrupt,
    lcd_stat: Interrupt,
    timer: Interrupt,
    serial: Interrupt,
    joypad: Interrupt,
}

impl InterruptHandler {
    pub fn set_ime(&mut self, enabled: bool) {
        self.ime = enabled;
    }

    pub fn ime(&self) -> bool {
        self.ime
    }

    pub fn write_if(&mut self, value: u8) {
        self.vblank.set = value & 0b00000001 != 0;
        self.lcd_stat.set = value & 0b00000010 != 0;
        self.timer.set = value & 0b00000100 != 0;
        self.serial.set = value & 0b00001000 != 0;
        self.joypad.set = value & 0b00010000 != 0;
    }

    pub fn write_ie(&mut self, value: u8) {
        self.vblank.enabled = value & 0b00000001 != 0;
        self.lcd_stat.enabled = value & 0b00000010 != 0;
        self.timer.enabled = value & 0b00000100 != 0;
        self.serial.enabled = value & 0b00001000 != 0;
        self.joypad.enabled = value & 0b00010000 != 0;
    }

    pub fn enable(&mut self, ir: InterruptType) {
        match ir {
            InterruptType::VBlank => self.vblank.enabled = true,
            InterruptType::LcdStat => self.lcd_stat.enabled = true,
            InterruptType::Timer => self.timer.enabled = true,
            InterruptType::Serial => self.serial.enabled = true,
            InterruptType::Joypad => self.joypad.enabled = true,
        }
    }

    pub fn disable(&mut self, ir: InterruptType) {
        match ir {
            InterruptType::VBlank => self.vblank.enabled = false,
            InterruptType::LcdStat => self.lcd_stat.enabled = false,
            InterruptType::Timer => self.timer.enabled = false,
            InterruptType::Serial => self.serial.enabled = false,
            InterruptType::Joypad => self.joypad.enabled = false,
        }
    }

    pub fn set(&mut self, ir: InterruptType) {
        match ir {
            InterruptType::VBlank => self.vblank.set = true,
            InterruptType::LcdStat => self.lcd_stat.set = true,
            InterruptType::Timer => self.timer.set = true,
            InterruptType::Serial => self.serial.set = true,
            InterruptType::Joypad => self.joypad.set = true,
        }
    }

    pub fn clear(&mut self, ir: InterruptType) {
        match ir {
            InterruptType::VBlank => self.vblank.set = false,
            InterruptType::LcdStat => self.lcd_stat.set = false,
            InterruptType::Timer => self.timer.set = false,
            InterruptType::Serial => self.serial.set = false,
            InterruptType::Joypad => self.joypad.set = false,
        }
    }

    pub fn test(&self, ir: InterruptType) -> bool {
        match ir {
            InterruptType::VBlank => self.vblank.enabled && self.vblank.set,
            InterruptType::LcdStat => self.lcd_stat.enabled && self.lcd_stat.set,
            InterruptType::Timer => self.timer.enabled && self.timer.set,
            InterruptType::Serial => self.serial.enabled && self.serial.set,
            InterruptType::Joypad => self.joypad.enabled && self.joypad.set,
        }
    }

    pub fn state(&self) -> InterruptHandlerState {
        InterruptHandlerState {
            ime: self.ime,
            vblank: self.vblank,
            lcd_stat: self.lcd_stat,
            timer: self.timer,
            serial: self.serial,
            joypad: self.joypad,
        }
    }
}
