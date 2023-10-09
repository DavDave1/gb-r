use egui::{ColorImage, TextureOptions};
use image::{GenericImage, RgbaImage};

use crate::gbr::ppu::{
    palette::{GrayShade, Palette},
    tile::TileData,
    TILE_HEIGHT, TILE_WIDTH,
};

const TILE_PER_ROW: usize = 32;
const ROWS: usize = 32;

fn create_image(
    tilemap: &[u8],
    tiles: &TileData,
    palette: &Palette,
    bg_tile_area: bool,
) -> ColorImage {
    let pal = Palette::new(
        GrayShade::White,
        GrayShade::LightGray,
        GrayShade::DarkGray,
        GrayShade::Black,
    );

    let w = TILE_PER_ROW * TILE_WIDTH as usize;
    let h = ROWS * TILE_HEIGHT as usize;

    let mut img = RgbaImage::new(w as u32, h as u32);

    for (i, tile_id) in tilemap.iter().enumerate() {
        let x = (i as u32) % TILE_PER_ROW as u32 * TILE_WIDTH;
        let y = (i as u32) / TILE_PER_ROW as u32 * TILE_HEIGHT;

        let mut tile_region = img.sub_image(x, y, TILE_WIDTH, TILE_HEIGHT);

        let tile = tiles.list()[tiles.tile_index_from_bg_map(*tile_id as usize, bg_tile_area)];

        for r in 0..TILE_HEIGHT as usize {
            for c in 0..TILE_WIDTH as usize {
                tile_region.put_pixel(
                    c as u32,
                    r as u32,
                    image::Rgba(pal.rgba(tile.pixels[r][c]).rgba),
                );
            }
        }
    }

    let resized = image::imageops::resize(
        &img,
        img.width() * 2,
        img.height() * 2,
        image::imageops::FilterType::Nearest,
    );

    let size = [resized.width() as _, resized.height() as _];
    let pixels = resized.as_flat_samples();

    ColorImage::from_rgba_unmultiplied(size, pixels.as_slice())
}

#[derive(Default)]
pub struct TilemapView {
    texture: Option<egui::TextureHandle>,
}

impl TilemapView {
    pub fn show(
        &mut self,
        tilemap: &[u8],
        tiles: &TileData,
        palette: &Palette,
        bg_tile_area: bool,
        ui: &mut egui::Ui,
    ) {
        match self.texture.as_mut() {
            None => {
                self.texture = Some(ui.ctx().load_texture(
                    "tiles_view",
                    create_image(tilemap, tiles, palette, bg_tile_area),
                    egui::TextureOptions::default(),
                ));
            }
            Some(tex_ref) => tex_ref.set(
                create_image(tilemap, tiles, palette, bg_tile_area),
                TextureOptions::default(),
            ),
        }

        let tex_ref = self.texture.as_ref().unwrap();

        ui.image(tex_ref, tex_ref.size_vec2());
    }
}
