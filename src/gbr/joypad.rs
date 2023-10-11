use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq )]
    pub struct Buttons: u8 {
        const Start = 1 << 3;
        const Select = 1 << 2;
        const B = 1 << 1;
        const A = 1 << 0;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq )]
    pub struct Directions: u8 {
        const Down = 1 << 3;
        const Up = 1 << 2;
        const Left = 1 << 1;
        const Right = 1 << 0;
    }
}

#[derive(Clone)]
pub struct Joypad {
    select_buttons: bool,
    select_directions: bool,
    buttons: Buttons,
    directions: Directions,
}

impl Default for Joypad {
    fn default() -> Self {
        Self {
            select_buttons: true,
            select_directions: true,
            buttons: Buttons::empty(),
            directions: Directions::empty(),
        }
    }
}

impl Joypad {
    pub fn write(&mut self, value: u8) {
        self.select_buttons = (value & 0b00100000) == 0;
        self.select_directions = (value & 0b00010000) == 0;
    }

    pub fn read(&self) -> u8 {
        // Need to invert bits because in register logic 0 means selected and
        // 1 not selected
        if self.select_buttons {
            0x10 | self.buttons.complement().bits()
        } else if self.select_directions {
            0x20 | self.directions.complement().bits()
        } else {
            0x3F
        }
    }

    pub fn press_button(&mut self, button: Buttons) {
        self.buttons.set(button, true);
    }

    pub fn press_direction(&mut self, direction: Directions) {
        self.directions.set(direction, true);
    }

    pub fn release_button(&mut self, button: Buttons) {
        self.buttons.set(button, false);
    }

    pub fn release_direction(&mut self, direction: Directions) {
        self.directions.set(direction, false);
    }
}
