use egui::{Color32, ColorImage, TextureHandle, TextureOptions};

use crate::gbr::ppu::palette::Palette;

const PALETTE_TILE_WIDTH: usize = 16;
const PALETTE_TILE_HEIGHT: usize = 16;
const SIZE: [usize; 2] = [4 * PALETTE_TILE_WIDTH, PALETTE_TILE_HEIGHT];

pub struct PaletteView {
    texture: Option<TextureHandle>,
    img: ColorImage,
}

impl PaletteView {
    pub fn new() -> Self {
        PaletteView {
            texture: None,
            img: ColorImage::new(SIZE, Color32::default()),
        }
    }

    pub fn show(&mut self, palette: &Palette, ui: &mut egui::Ui) {
        self.update_image(palette);

        let texture = self.texture.get_or_insert_with(|| {
            ui.ctx()
                .load_texture("tiles_view", self.img.clone(), TextureOptions::default())
        });

        texture.set(self.img.clone(), TextureOptions::default());

        ui.image(texture.id(), texture.size_vec2());
    }

    fn update_image(&mut self, palette: &Palette) {
        for index in 0..SIZE[0] * SIZE[1] {
            let x = (index) % SIZE[0];

            let color_id = x / PALETTE_TILE_WIDTH;

            let rgba = &palette.rgba(color_id as u8).rgba;
            self.img.pixels[index] =
                Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3]);
        }
    }
}
