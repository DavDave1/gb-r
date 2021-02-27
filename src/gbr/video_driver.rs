#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use beryllium::*;
use pixels::{Pixels, SurfaceTexture};

use crate::gbr::game_boy::GameBoy;

pub struct VideoDriver {
    sdl: beryllium::SDL,
    window: beryllium::RawWindow,
    pixels: Pixels<beryllium::RawWindow>,
}

impl VideoDriver {
    pub fn new(width: u32, height: u32) -> Self {
        let sdl = SDL::init(InitFlags::default()).expect("Failed to initialize SDL");
        let window = sdl
            .create_raw_window("gb-r", WindowPosition::Centered, width, height, 0)
            .expect("Failed to create SDL raw window");
        let pixels = {
            // TODO: Beryllium does not expose the SDL2 `GetDrawableSize` APIs, so choosing the correct
            // surface texture size is not possible.
            let surface_texture = SurfaceTexture::new(width, height, &window);
            Pixels::new(width, height, surface_texture).unwrap()
        };

        VideoDriver {
            sdl: sdl,
            window: window,
            pixels: pixels,
        }
    }

    pub fn draw(&mut self, game_boy: &GameBoy) {
        self.pixels.render();
    }
}
