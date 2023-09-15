use crate::gbr::ppu::TILEMAP_BLOCK0_START;

use super::{
    background_palette::BackgroundPalette, lcd_control_register::LcdControlRegister,
    tile::TILE_COLOR_ID, MODE_2_DOTS, SCREEN_HEIGHT, SCREEN_WIDTH, TILE_BLOCK1_START,
    TILE_BLOCK2_START, TILE_DATA_SIZE,
};

#[derive(PartialEq)]
enum Step {
    GetTileIndex,
    GetTileData,
    DrawTile,
}

pub struct PixelProcessor {
    scan_line_x: u8,
    old_dots: u16,
    curr_step: Step,
    curr_tile_index: u8,
    curr_tile_line: u8,
    curr_tile_msb: u8,
    curr_tile_lsb: u8,
    scroll_x: u8,
    scroll_y: u8,
    pub screen_buffer: Vec<u8>,
}

impl PixelProcessor {
    pub fn new() -> Self {
        Self {
            scan_line_x: 0,
            old_dots: MODE_2_DOTS,
            curr_step: Step::GetTileIndex,
            curr_tile_index: 0,
            curr_tile_line: 0,
            curr_tile_msb: 0,
            curr_tile_lsb: 0,
            scroll_x: 0,
            scroll_y: 0,
            screen_buffer: vec![0; (SCREEN_WIDTH * SCREEN_HEIGHT * 4) as usize],
        }
    }

    pub fn start(
        &mut self,
        ly: u8,
        dots: u16,
        viewport: &(u8, u8),
        lcd_ctrl: &LcdControlRegister,
        vram: &[u8],
        bg_palette: &BackgroundPalette,
    ) {
        self.scan_line_x = 0;
        self.old_dots = MODE_2_DOTS;
        self.curr_step = Step::GetTileIndex;
        // self.screen_buffer[ly as usize].fill(0);
        self.curr_tile_index = 0;
        self.curr_tile_line = 0;
        self.curr_tile_msb = 0;
        self.curr_tile_lsb = 0;
        self.scroll_x = viewport.0;
        self.scroll_y = viewport.1;

        self.process(ly, dots, viewport, lcd_ctrl, vram, bg_palette);
    }

    pub fn finished(&self) -> bool {
        self.scan_line_x as u32 >= SCREEN_WIDTH
    }

    pub fn process(
        &mut self,
        ly: u8,
        dots: u16,
        viewport: &(u8, u8),
        lcd_ctrl: &LcdControlRegister,
        vram: &[u8],
        bg_palette: &BackgroundPalette,
    ) {
        if dots < self.old_dots {
            return;
        }

        let mut delta_dots = (dots - self.old_dots) as i16;

        while delta_dots > 0 && !self.finished() {
            if self.curr_step == Step::GetTileIndex {
                self.get_tile_index(ly, viewport, vram);
                delta_dots -= 2;
                self.curr_step = Step::GetTileData;
            }

            if delta_dots <= 0 {
                break;
            }

            if self.curr_step == Step::GetTileData {
                self.get_tile_data(lcd_ctrl, vram);
                delta_dots -= 4;
                self.curr_step = Step::DrawTile;
            }

            if delta_dots <= 0 {
                break;
            }

            if self.curr_step == Step::DrawTile {
                self.draw_tile(ly, bg_palette);
                self.scan_line_x += 8;
                self.curr_step = Step::GetTileIndex;
                delta_dots -= 1;
            }
        }

        self.old_dots = (dots as i16 - delta_dots) as u16;
    }

    fn get_tile_index(&mut self, ly: u8, viewport: &(u8, u8), vram: &[u8]) {
        self.scroll_x |= viewport.0 & 0b11111000;
        self.scroll_y = viewport.1;

        let tile_x = self.scroll_x.wrapping_add(self.scan_line_x) as u16;
        let mut tile_y = self.scroll_y as u16 + ly as u16;
        if tile_y > 255 + 8 {
            tile_y -= 255
        } else if tile_y > 255 {
            tile_y -= 8;
        }

        // let tile_y = viewport.1.wrapping_add(ly) as u16;

        self.curr_tile_line = (tile_y % 8) as u8;
        let tilemap_addr = tile_y / 8 * 32 + tile_x / 8;

        self.curr_tile_index = vram[TILEMAP_BLOCK0_START as usize + tilemap_addr as usize];
    }

    fn get_tile_data(&mut self, lcd_ctrl: &LcdControlRegister, vram: &[u8]) {
        let tile_block_addr = if lcd_ctrl.bg_and_window_tile_area_sel {
            0
        } else if self.curr_tile_index < 128 {
            TILE_BLOCK1_START as usize
        } else {
            TILE_BLOCK2_START as usize
        };

        let tile_addr = tile_block_addr
            + self.curr_tile_index as usize * TILE_DATA_SIZE
            + 2 * self.curr_tile_line as usize;

        self.curr_tile_lsb = vram[tile_addr];
        self.curr_tile_msb = vram[tile_addr + 1];
    }

    fn draw_tile(&mut self, ly: u8, bg_palette: &BackgroundPalette) {
        for x in 0..8 {
            let shift = 7 - x;

            let msb = self.curr_tile_msb >> shift & 0b1;
            let lsb = self.curr_tile_lsb >> shift & 0b1;
            let color_id = (msb << 1u16) + lsb;

            let screen_x = (self.scan_line_x + x) as usize;
            let screen_y = ly as usize;
            let screen_index = (screen_y * SCREEN_WIDTH as usize + screen_x) * 4;

            self.screen_buffer[screen_index..screen_index + 4]
                .copy_from_slice(&bg_palette.to_rgba(color_id).rgba);

            // self.screen_buffer[screen_index..screen_index + 3]
            //     .copy_from_slice(&TILE_COLOR_ID[self.curr_tile_index as usize]);
            // self.screen_buffer[screen_index + 3] = 0xFF;
        }
    }
}
