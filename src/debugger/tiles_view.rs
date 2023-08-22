use egui::ColorImage;
use image::{GenericImage, RgbaImage};

use crate::gbr::ppu::{TileList, TILE_HEIGHT, TILE_WIDTH};

const TILE_PER_ROW: usize = 32;

// TODO: replace usage if image library with handmade
// tilemap to frame generator
fn create_image(tiles: &TileList) -> ColorImage {
    let rows_count = tiles.len() / TILE_PER_ROW as usize;
    let w = TILE_PER_ROW * TILE_WIDTH as usize;
    let h = rows_count * TILE_HEIGHT as usize;

    let mut img = RgbaImage::new(w as u32, h as u32);

    for (i, tile) in tiles.iter().enumerate() {
        let x = (i as u32) % TILE_PER_ROW as u32 * TILE_WIDTH;
        let y = (i as u32) / TILE_PER_ROW as u32 * TILE_HEIGHT;

        let mut tile_region = img.sub_image(x, y, TILE_WIDTH, TILE_HEIGHT);

        for r in 0..TILE_WIDTH as usize {
            for c in 0..TILE_HEIGHT as usize {
                tile_region.put_pixel(
                    r as u32,
                    c as u32,
                    image::Rgba([
                        tile.pixels[r][c].r,
                        tile.pixels[r][c].g,
                        tile.pixels[r][c].b,
                        tile.pixels[r][c].a,
                    ]),
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
pub struct TilesView {
    texture: Option<egui::TextureHandle>,
}

impl TilesView {
    pub fn show(&mut self, tiles: &TileList, ui: &mut egui::Ui) {
        if tiles.len() > TILE_PER_ROW {
            self.texture = Some(ui.ctx().load_texture("tiles_view", create_image(tiles)));

            let tex_ref = self.texture.as_ref().unwrap();

            ui.image(tex_ref, tex_ref.size_vec2());
        }
    }
}
