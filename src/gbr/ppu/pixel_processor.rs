use super::{
    background_palette::BackgroundPalette, lcd_control_register::LcdControlRegister, MODE_2_DOTS,
    SCREEN_HEIGHT, SCREEN_WIDTH, TILE_BLOCK1_START, TILE_BLOCK2_START,
};

#[derive(PartialEq)]
enum Step {
    GetTileIndex,
    GetTileData,
    DrawTile,
}

pub struct PixelProcessor {
    scan_line_x: u8,
    dots: i16,
    curr_step: Step,
    curr_tile_index: u8,
    curr_tile_line: u8,
    curr_tile_msb: u8,
    curr_tile_lsb: u8,
    pub screen_buffer: Vec<u8>,
}

impl PixelProcessor {
    pub fn new() -> Self {
        Self {
            scan_line_x: 0,
            dots: 0,
            curr_step: Step::GetTileIndex,
            curr_tile_index: 0,
            curr_tile_line: 0,
            curr_tile_msb: 0,
            curr_tile_lsb: 0,
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
        self.dots = 0;
        self.curr_step = Step::GetTileIndex;
        // self.screen_buffer[ly as usize].fill(0);
        self.curr_tile_index = 0;
        self.curr_tile_line = 0;
        self.curr_tile_msb = 0;
        self.curr_tile_lsb = 0;

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
        self.dots += (dots - MODE_2_DOTS) as i16;

        while self.dots > 0 && !self.finished() {
            if self.curr_step == Step::GetTileIndex {
                self.get_tile_index(ly, viewport);
                self.dots -= 2;
                self.curr_step = Step::GetTileData;
            }

            if self.dots <= 0 {
                break;
            }

            if self.curr_step == Step::GetTileData {
                self.get_tile_data(lcd_ctrl, vram);
                self.dots -= 4;
                self.curr_step = Step::DrawTile;
            }

            if self.dots <= 0 {
                break;
            }

            if self.curr_step == Step::DrawTile {
                self.draw_tile(ly, bg_palette);
                self.scan_line_x += 8;
            }
        }
    }

    fn get_tile_index(&mut self, ly: u8, viewport: &(u8, u8)) {
        let tile_x = (viewport.0 / 8 + self.scan_line_x / 8) & 0x1F;
        let tile_y = (ly + viewport.1) & 0xFF;

        self.curr_tile_line = tile_y % 8;
        self.curr_tile_index = tile_y / 8 * 32 + tile_x;
    }

    fn get_tile_data(&mut self, lcd_ctrl: &LcdControlRegister, vram: &[u8]) {
        if lcd_ctrl.bg_and_window_tile_area_sel {
            self.curr_tile_msb = vram[self.curr_tile_index as usize];
            self.curr_tile_lsb = vram[self.curr_tile_index as usize + 1];
        } else if self.curr_tile_index < 128 {
            self.curr_tile_msb = vram[TILE_BLOCK1_START as usize + self.curr_tile_index as usize];
            self.curr_tile_lsb =
                vram[TILE_BLOCK1_START as usize + self.curr_tile_index as usize + 1];
        } else {
            self.curr_tile_msb = vram[TILE_BLOCK2_START as usize + self.curr_tile_index as usize];
            self.curr_tile_lsb =
                vram[TILE_BLOCK2_START as usize + self.curr_tile_index as usize + 1];
        }
    }

    fn draw_tile(&mut self, ly: u8, bg_palette: &BackgroundPalette) {
        for x in 0..8 {
            let shift = 7 - x;

            let msb = self.curr_tile_msb >> shift & 0b1;
            let lsb = self.curr_tile_lsb >> shift & 0b1;
            let color_id = (msb << 1u16) + lsb;

            let screen_x = (self.scan_line_x + x) as usize;
            let screen_y = ly as usize;
            let screen_index = screen_y * SCREEN_WIDTH as usize * 4 + screen_x * 4;

            self.screen_buffer[screen_index..screen_index + 4]
                .copy_from_slice(&bg_palette.to_rgba(color_id).rgba);
        }
    }
}
