use crate::gbr::{
    oam::{ObjAttribute, ObjAttributeMemory},
    ppu::TILEMAP_BLOCK0_START,
};

use super::{
    lcd_control_register::LcdControlRegister,
    palette::Palette,
    tile::{TileData, TILE_COLOR_ID},
    Point, MODE_2_DOTS, SCREEN_HEIGHT, SCREEN_WIDTH, TILEMAP_BLOCK1_START,
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
    curr_tile_index: usize,
    curr_tile_line: u8,
    curr_tile_msb: u8,
    curr_tile_lsb: u8,
    scroll_x: u8,
    scroll_y: u8,
    objs: Vec<ObjAttribute>,
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
            objs: vec![],
            screen_buffer: vec![0; (SCREEN_WIDTH * SCREEN_HEIGHT * 4) as usize],
        }
    }

    pub fn start(
        &mut self,
        oam: &ObjAttributeMemory,
        ly: u8,
        dots: u16,
        viewport: &Point,
        lcd_ctrl: &LcdControlRegister,
        vram: &[u8],
        tiles: &TileData,
        bg_palette: &Palette,
    ) {
        self.scan_line_x = 0;
        self.old_dots = MODE_2_DOTS;
        self.curr_step = Step::GetTileIndex;
        // self.screen_buffer[ly as usize].fill(0);
        self.curr_tile_index = 0;
        self.curr_tile_line = 0;
        self.curr_tile_msb = 0;
        self.curr_tile_lsb = 0;
        self.scroll_x = viewport.x;
        self.scroll_y = viewport.y;
        self.objs = oam.get_objs_at_line(ly);

        self.process(ly, dots, viewport, lcd_ctrl, vram, tiles, bg_palette);
    }

    pub fn finished(&self) -> bool {
        self.scan_line_x as u32 >= SCREEN_WIDTH
    }

    pub fn process(
        &mut self,
        ly: u8,
        dots: u16,
        viewport: &Point,
        lcd_ctrl: &LcdControlRegister,
        vram: &[u8],
        tiles: &TileData,
        bg_palette: &Palette,
    ) {
        if dots < self.old_dots {
            return;
        }

        let mut delta_dots = (dots - self.old_dots) as i16;

        while delta_dots > 0 && !self.finished() {
            if self.curr_step == Step::GetTileIndex {
                self.get_tile_index(lcd_ctrl, ly, viewport, vram);
                delta_dots -= 2;
                self.curr_step = Step::GetTileData;
            }

            if delta_dots <= 0 {
                break;
            }

            if self.curr_step == Step::GetTileData {
                self.get_tile_data(tiles, lcd_ctrl);
                delta_dots -= 4;
                self.curr_step = Step::DrawTile;
            }

            if delta_dots <= 0 {
                break;
            }

            if self.curr_step == Step::DrawTile {
                self.draw_tile(ly, tiles, bg_palette);
                self.scan_line_x += 8;
                self.curr_step = Step::GetTileIndex;
                delta_dots -= 1;
            }
        }

        self.old_dots = (dots as i16 - delta_dots) as u16;
    }

    fn get_tile_index(
        &mut self,
        lcd_ctrl: &LcdControlRegister,
        ly: u8,
        viewport: &Point,
        vram: &[u8],
    ) {
        self.scroll_x |= viewport.x & 0b11111000;
        self.scroll_y = viewport.y;

        let tile_x = self.scroll_x.wrapping_add(self.scan_line_x) as u16;
        let mut tile_y = self.scroll_y as u16 + ly as u16;
        if tile_y > 255 + 8 {
            tile_y -= 255
        } else if tile_y > 255 {
            tile_y -= 8;
        }

        self.curr_tile_line = (tile_y % 8) as u8;
        let tilemap_addr = tile_y / 8 * 32 + tile_x / 8;

        let tile_block_addr = if lcd_ctrl.bg_tile_map_area_sel {
            TILEMAP_BLOCK1_START
        } else {
            TILEMAP_BLOCK0_START
        };

        self.curr_tile_index = vram[tile_block_addr as usize + tilemap_addr as usize] as usize;
    }

    fn get_tile_data(&mut self, tiles: &TileData, lcd_ctrl: &LcdControlRegister) {
        self.curr_tile_index = tiles
            .tile_index_from_bg_map(self.curr_tile_index, lcd_ctrl.bg_and_window_tile_area_sel);
    }

    fn draw_tile(&mut self, ly: u8, tiles: &TileData, bg_palette: &Palette) {
        let tile = &tiles.list()[self.curr_tile_index];

        for x in 0..8 {
            let screen_x = (self.scan_line_x + x) as usize;
            let screen_y = ly as usize;
            let screen_index = (screen_y * SCREEN_WIDTH as usize + screen_x) * 4;

            self.screen_buffer[screen_index..screen_index + 4].copy_from_slice(
                &bg_palette
                    .rgba(tile.pixels[self.curr_tile_line as usize][x as usize])
                    .rgba,
            );

            // self.screen_buffer[screen_index..screen_index + 3]
            //     .copy_from_slice(&TILE_COLOR_ID[self.curr_tile_index as usize]);
            // self.screen_buffer[screen_index + 3] = 0xFF;
        }
    }
}
