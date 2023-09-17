use random_color::RandomColor;

use super::rgba::Rgba;
use super::{TILE_HEIGHT, TILE_WIDTH};

lazy_static! {
    pub static ref TILE_COLOR_ID: Vec<[u8; 3]> = {
        let mut v = vec![[0; 3]; 256];

        for i in 0..256 {
            let c = RandomColor::new().to_rgb_array();
            v[i][0] = c[0];
            v[i][1] = c[1];
            v[i][2] = c[2];
        }

        v
    };
}

#[derive(Clone, Copy)]
pub struct Tile {
    pub pixels: [u8; TILE_WIDTH as usize * TILE_HEIGHT as usize],
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            pixels: [0; TILE_WIDTH as usize * TILE_HEIGHT as usize],
        }
    }
}

impl Tile {
    pub fn from_data(data: &[u8]) -> Self {
        let mut tile = Self::default();

        // Tile data is represented as 2 bytes per line
        let mut index = 0;
        for line in data.chunks_exact(2) {
            Self::decode_line(
                line[1],
                line[0],
                &mut tile.pixels[index..index + TILE_WIDTH as usize],
            );

            index += TILE_WIDTH as usize;
        }

        tile
    }

    pub fn decode_line(msb_in: u8, lsb_in: u8, dst: &mut [u8]) {
        for x in 0..8 {
            let shift = 7 - x;

            let msb = msb_in >> shift & 0b1;
            let lsb = lsb_in >> shift & 0b1;
            let color_id = (msb << 1u16) + lsb;

            dst[x] = color_id;
        }
    }
}
