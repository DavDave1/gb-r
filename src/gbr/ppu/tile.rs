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
    pub pixels: [[Rgba; TILE_WIDTH as usize]; TILE_HEIGHT as usize],
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            pixels: [[Rgba::white(); TILE_WIDTH as usize]; TILE_HEIGHT as usize],
        }
    }
}

impl Tile {
    pub fn from_data(data: &[u8], palette: &[Rgba]) -> Self {
        let mut tile = Self::default();

        // Tile data is represented as 2 bytes per line
        for (y, line) in data.chunks_exact(2).enumerate() {
            for x in 0..8 {
                let shift = 7 - x;

                let msb = line[1] >> shift & 0b1;
                let lsb = line[0] >> shift & 0b1;
                let color_id = (msb << 1u16) + lsb;

                tile.pixels[x][y] = palette[color_id as usize];
            }
        }

        tile
    }
}
