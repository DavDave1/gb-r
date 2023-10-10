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
    PushBg,
    PushObjs,
    PopPixels,
}

struct Pixel {
    color_id: u8,
    palette_id: usize,
    tile_index: usize,
    is_bg: bool,
}

impl Pixel {
    pub fn new(color_id: u8, palette_id: usize, tile_index: usize) -> Self {
        Self {
            color_id,
            palette_id,
            tile_index,
            is_bg: true,
        }
    }

    pub fn push_obj(&mut self, color_id: u8, palette_id: usize, tile_index: usize, bg_prio: bool) {
        if self.is_bg && color_id != 0 && (!bg_prio || (bg_prio && self.color_id == 0)) {
            self.color_id = color_id;
            self.palette_id = palette_id;
            self.tile_index = tile_index;
            self.is_bg = false;
        }
    }
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
    pixel_fifo: Vec<Pixel>,
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
            pixel_fifo: vec![],
            screen_buffer: vec![0; (SCREEN_WIDTH * SCREEN_HEIGHT * 4) as usize],
        }
    }

    pub fn start(&mut self, oam: &ObjAttributeMemory, ly: u8, viewport: &Point) {
        self.scan_line_x = 0;
        self.old_dots = MODE_2_DOTS;
        self.curr_step = Step::GetTileIndex;
        self.curr_tile_index = 0;
        self.curr_tile_line = 0;
        self.curr_tile_msb = 0;
        self.curr_tile_lsb = 0;
        self.scroll_x = viewport.x;
        self.scroll_y = viewport.y;
        self.pixel_fifo.clear();

        self.objs = oam.get_objs_at_line(ly);
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
        obj_palettes: &[Palette],
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
                self.curr_step = Step::PushBg;
            }

            if delta_dots <= 0 {
                break;
            }

            if self.curr_step == Step::PushBg {
                self.push_bg(tiles);
                self.curr_step = Step::PushObjs;
                delta_dots -= 1;
            }

            if self.curr_step == Step::PushObjs {
                self.curr_step = Step::PopPixels;
                self.push_objs(ly, tiles, lcd_ctrl);
                delta_dots -= 1;
            }

            if self.curr_step == Step::PopPixels {
                self.curr_step = Step::GetTileIndex;
                self.pop_pixels(ly, bg_palette, obj_palettes);
                self.scan_line_x += 8;
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

    fn push_bg(&mut self, tiles: &TileData) {
        let tile = &tiles.list()[self.curr_tile_index];

        for x in 0..8 {
            self.pixel_fifo.push(Pixel::new(
                tile.pixels[self.curr_tile_line as usize][x as usize],
                0,
                self.curr_tile_index,
            ));
        }
    }

    fn push_objs(&mut self, ly: u8, tiles: &TileData, lcd_ctrl: &LcdControlRegister) {
        if lcd_ctrl.obj_enable == false {
            return;
        }

        // Remove objects before scanline, since they will not be drawn anymore
        self.objs.retain(|o| o.right() > self.scan_line_x as i16);

        for obj in self.objs.iter() {
            // Entire object is after scanline, don't process any other objects
            // sice they will all be after scanline because they are x ordered
            if obj.left() > self.scan_line_x as i16 + 8 {
                break;
            }

            let tile = &tiles.list()[obj.tile_index() as usize];
            let line = tile.pixels[(ly as i16 - obj.top()) as usize];

            let x_start = 0.max(obj.left() - self.scan_line_x as i16);

            let x_end = 8.min(obj.right() - self.scan_line_x as i16);

            let tile_start = if x_start > 0 {
                0
            } else {
                (self.scan_line_x as i16 - obj.left()) as usize
            };

            for (tile_x, fifo_x) in (x_start as usize..x_end as usize).enumerate() {
                self.pixel_fifo[fifo_x].push_obj(
                    line[tile_start + tile_x],
                    obj.palette_id() as usize,
                    obj.tile_index() as usize,
                    obj.bg_win_prio(),
                );
            }
        }
    }

    fn pop_pixels(&mut self, ly: u8, bg_palette: &Palette, obj_palettes: &[Palette]) {
        for (x, pixel) in self.pixel_fifo.iter().enumerate() {
            let screen_x = self.scan_line_x as usize + x;
            let screen_y = ly as usize;
            let screen_index = (screen_y * SCREEN_WIDTH as usize + screen_x) * 4;

            let palette = if pixel.is_bg {
                &bg_palette
            } else {
                &obj_palettes[pixel.palette_id]
            };

            self.screen_buffer[screen_index..screen_index + 4]
                .copy_from_slice(&palette.rgba(pixel.color_id).rgba);

            // if pixel.is_bg {
            //     self.screen_buffer[screen_index..screen_index + 4]
            //         .copy_from_slice(&palette.rgba(pixel.color_id).rgba);
            // } else {
            //     self.screen_buffer[screen_index..screen_index + 3]
            //         .copy_from_slice(&TILE_COLOR_ID[pixel.tile_index]);
            //     self.screen_buffer[screen_index + 3] = 0xFF;
            // }
        }

        self.pixel_fifo.clear();
    }
}
