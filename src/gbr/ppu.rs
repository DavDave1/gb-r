use rand::prelude::*;

use crate::gbr::bus::Bus;

use super::{memory_map, GbError};

pub const SCREEN_WIDTH: u32 = 190;
pub const SCREEN_HEIGHT: u32 = 144;
pub const NUM_CHANNELS: u32 = 4; // rgba
pub const SCREEN_SIZE: usize = (SCREEN_WIDTH * SCREEN_HEIGHT * NUM_CHANNELS) as usize;

const RENDER_FRAME_WIDTH: u32 = 256;
const RENDER_FRAME_HEIGHT: u32 = 256;
const RENDER_FRAME_SIZE: usize = (RENDER_FRAME_WIDTH * RENDER_FRAME_HEIGHT * NUM_CHANNELS) as usize;

pub const TILE_WIDTH: u32 = 8;
pub const TILE_HEIGHT: u32 = 8;
const TILE_RENDER_SIZE: usize = (TILE_WIDTH * TILE_HEIGHT * NUM_CHANNELS) as usize;
const TILE_DATA_SIZE: usize = 16;

pub const TILE_BLOCK_SIZE: usize = 128;

pub type ScreenBuffer = Vec<u8>;
pub type TileList = Vec<Tile>;

#[derive(Clone, Copy)]
pub struct Tile {
    pub data: [u8; TILE_RENDER_SIZE],
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            data: [0; TILE_RENDER_SIZE],
        }
    }
}

impl Tile {
    fn from_data(data: &[u8], palette: &[u32]) -> Self {
        let mut tile = Self::default();

        let r: u8 = rand::thread_rng().gen();
        let g: u8 = rand::thread_rng().gen();
        let b: u8 = rand::thread_rng().gen();
        for pixel in tile.data.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[r, g, b, 0xFF]);
        }

        tile
    }
}

pub struct PPU {
    screen_buffer: Vec<u8>,
    render_buffer: Vec<u8>,
    tile_list: Vec<Tile>,
    palette: Box<[u32; 4]>,
    render_ch: (flume::Sender<ScreenBuffer>, flume::Receiver<ScreenBuffer>),
}

impl PPU {
    pub fn new() -> Self {
        Self {
            screen_buffer: vec![0; SCREEN_SIZE],
            render_buffer: vec![0; RENDER_FRAME_SIZE],
            tile_list: vec![Tile::default(); 3 * TILE_BLOCK_SIZE],
            palette: Box::new([0; 4]),
            render_ch: flume::bounded(1),
        }
    }

    pub fn tile_list(&self) -> &[Tile] {
        &self.tile_list
    }

    pub fn render_watch(&self) -> flume::Receiver<ScreenBuffer> {
        self.render_ch.1.clone()
    }

    pub fn render(&mut self, bus: &Bus) -> Result<(), GbError> {
        self.update_tile_list(bus)?;
        self.draw_test_frame();

        self.render_ch.0.try_send(self.screen_buffer.clone()).ok();
        Ok(())
    }

    fn update_tile_list(&mut self, bus: &Bus) -> Result<(), GbError> {
        let mut tile_addr = memory_map::TILE_DATA_START;

        let mut tile_index = 0usize;
        while tile_addr < memory_map::TILE_DATA_END {
            let mut tile_data = [0u8; TILE_DATA_SIZE];

            for chunk in tile_data.chunks_exact_mut(2) {
                chunk.copy_from_slice(&bus.ppu_read_word(tile_addr)?.to_be_bytes());
                tile_addr += 2;
            }

            self.tile_list[tile_index] = Tile::from_data(&tile_data, &(*self.palette));
            tile_index += 1;
        }

        Ok(())
    }

    fn draw_test_frame(&mut self) {
        for pixel in self.screen_buffer.chunks_exact_mut(4) {
            let r: u8 = rand::thread_rng().gen();
            let g: u8 = rand::thread_rng().gen();
            let b: u8 = rand::thread_rng().gen();
            pixel.copy_from_slice(&[r, g, b, 0xFF]);
        }
    }
}
