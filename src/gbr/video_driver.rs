#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use beryllium::{
    event::Event,
    init::{InitFlags, Sdl},
    window::WindowFlags,
};
use fermium::keycode;
use pixels::{Pixels, SurfaceTexture};
use std::sync::{Arc, RwLock, RwLockReadGuard};
use zstring::zstr;

use crate::gbr::{
    game_boy::GameBoy,
    ppu::{self, PPU},
};

pub struct VideoDriver {
    emu: Arc<RwLock<GameBoy>>,
    width: u32,
    height: u32,
}

impl VideoDriver {
    pub fn new(emu: Arc<RwLock<GameBoy>>, width: u32, height: u32) -> Self {
        VideoDriver { emu, width, height }
    }

    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let sdl = Sdl::init(InitFlags::EVERYTHING)?;
        let window = sdl.create_vk_window(
            zstr!("gb-r"),
            None,
            (self.width as i32, self.height as i32),
            WindowFlags::ALLOW_HIGHDPI,
        )?;

        let mut pixels = {
            let surface_texture =
                SurfaceTexture::new(ppu::SCREEN_WIDTH, ppu::SCREEN_HEIGHT, &*window);
            Pixels::new(self.width, self.height, surface_texture)?
        };

        'game_loop: loop {
            while let Some(event) = sdl.poll_event() {
                match event {
                    Event::Quit { .. } => break 'game_loop,
                    Event::Keyboard { keycode: key, .. } if key == keycode::SDLK_ESCAPE => {
                        break 'game_loop
                    }
                    // Event::WindowResized { width, height, .. } => {
                    //     pixels.resize_surface(width, height)
                    // }
                    _ => (),
                }
            }

            {
                let gb = self.emu.read().unwrap();
                if gb.bus().io_registers().lcd_control().display_enable() {
                    VideoDriver::draw(&gb, pixels.get_frame());
                    pixels.render()?;
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(166));
        }

        Ok(())
    }

    fn draw(emu: &RwLockReadGuard<GameBoy>, frame: &mut [u8]) {
        frame.copy_from_slice(emu.ppu().buffer());
    }
}
