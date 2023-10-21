use random_color::RandomColor;

use super::{
    TILE_BLOCK_SIZE, TILE_DATA_SIZE, TILE_HEIGHT, TILE_MAP_DATA_COLS, TILE_MAP_DATA_ROWS,
    TILE_WIDTH,
};

lazy_static! {
    pub static ref TILE_COLOR_ID: Vec<[u8; 3]> = {
        let mut v = vec![[0; 3]; 3 * TILE_BLOCK_SIZE];

        for i in 0..v.len() {
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
    pub pixels: [[u8; TILE_WIDTH as usize]; TILE_HEIGHT as usize],
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            pixels: [[0; TILE_WIDTH as usize]; TILE_HEIGHT as usize],
        }
    }
}

impl Tile {
    #[allow(dead_code)]
    pub fn from_data(data: &[u8]) -> Self {
        let mut tile = Self::default();

        // Tile data is represented as 2 bytes per line
        let mut index = 0;
        for line in data.chunks_exact(2) {
            tile.decode_line(index, line[1], line[0]);

            index += 1;
        }

        tile
    }

    pub fn decode_line(&mut self, line_index: usize, msb: u8, lsb: u8) {
        for x in 0..8 {
            let shift = 7 - x;

            let msb_t = ((msb >> shift) & 0b1) << 1;
            let lsb_t = (lsb >> shift) & 0b1;
            let color_id = msb_t + lsb_t;

            self.pixels[line_index][x] = color_id;
        }
    }

    pub fn line(&self, line_index: usize, flip_y: bool, flip_x: bool) -> [u8; TILE_WIDTH as usize] {
        let index = if flip_y {
            TILE_HEIGHT as usize - line_index
        } else {
            line_index
        };

        let mut line = self.pixels[index].clone();

        if flip_x {
            line.reverse();
        }

        line
    }
}

#[derive(Default, Clone)]
pub struct TileData {
    tiles: Vec<Tile>,
}

impl TileData {
    pub fn new() -> Self {
        Self {
            tiles: vec![Tile::default(); 3 * TILE_BLOCK_SIZE],
        }
    }

    pub fn clear(&mut self) {
        self.tiles.fill(Tile::default());
    }

    pub fn list(&self) -> &Vec<Tile> {
        &self.tiles
    }

    pub fn tile_index_from_bg_map(index: usize, tile_area_select: bool) -> usize {
        if tile_area_select {
            return index;
        } else if index < 128 {
            return 2 * TILE_BLOCK_SIZE + index;
        } else {
            return TILE_BLOCK_SIZE + index - 128;
        }
    }

    pub fn write_line(&mut self, addr: usize, msb: u8, lsb: u8) {
        let index = addr as usize / TILE_DATA_SIZE;
        let line_index = (addr as usize % TILE_DATA_SIZE) / 2;

        self.tiles[index].decode_line(line_index, msb, lsb);
    }
}

#[derive(Default, Clone)]
pub struct TileMap {
    data: [[u8; TILE_MAP_DATA_COLS]; TILE_MAP_DATA_ROWS],
}

impl TileMap {
    pub fn at(&self, r: usize, c: usize) -> u8 {
        self.data[r][c]
    }

    pub fn set(&mut self, addr: u16, value: u8) {
        let r = addr as usize / TILE_MAP_DATA_ROWS;
        let c = addr as usize % TILE_MAP_DATA_COLS;

        self.data[r][c] = value;
    }

    pub fn line(&self, line_index: usize, tiles: &TileData, tile_area_select: bool) -> [u8; 256] {
        let mut line = [0; 256];

        let row = line_index / TILE_HEIGHT as usize;
        let tile_line = line_index % TILE_HEIGHT as usize;

        for col in 0..TILE_MAP_DATA_COLS {
            let tile_index =
                TileData::tile_index_from_bg_map(self.data[row][col] as usize, tile_area_select);
            let tile = &tiles.list()[tile_index];

            line[col * TILE_WIDTH as usize..8 + col * TILE_WIDTH as usize]
                .copy_from_slice(&tile.line(tile_line, false, false));
        }

        line
    }
}
