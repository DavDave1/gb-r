use crate::gbr::bus::Bus;

use super::GbError;

pub const SCREEN_WIDTH: u32 = 190;
pub const SCREEN_HEIGHT: u32 = 144;
pub const NUM_CHANNELS: u32 = 4; // rgba
pub const SCREEN_SIZE: usize = (SCREEN_WIDTH * SCREEN_HEIGHT * NUM_CHANNELS) as usize;

struct Screen {
    data: [u8; SCREEN_SIZE],
}

impl Screen {
    fn new() -> Self {
        Self {
            data: [127; SCREEN_SIZE],
        }
    }
}

pub struct PPU {
    screen: Screen,
}

impl PPU {
    pub fn new() -> Self {
        Self {
            screen: Screen::new(),
        }
    }

    pub fn render(&mut self, bus: &Bus) -> Result<(), GbError> {
        Ok(())
    }

    pub fn buffer(&self) -> &[u8] {
        &self.screen.data
    }
}
