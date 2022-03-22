use log::error;
use pixels::{Pixels, SurfaceTexture};
use std::sync::{Arc, RwLock, RwLockReadGuard};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

use crate::gbr::{game_boy::GameBoy, ppu};

use crate::debugger::ui::Ui;

use super::debugger::Debugger;

pub struct VideoDriver {
    debugger: Arc<Debugger>,
    width: u32,
    height: u32,
}

impl VideoDriver {
    pub fn new(debugger: Arc<Debugger>, width: u32, height: u32) -> Self {
        VideoDriver {
            debugger,
            width,
            height,
        }
    }

    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::debug!("window initialization");

        let event_loop = EventLoop::new();
        let mut input = WinitInputHelper::new();

        let win_size = LogicalSize::new(self.width as f64, self.height as f64);
        let window = WindowBuilder::new()
            .with_title("gb-r")
            .with_inner_size(win_size)
            .with_min_inner_size(win_size)
            .build(&event_loop)?;

        let scale_factor = window.scale_factor() as f32;

        let surface_texture = SurfaceTexture::new(ppu::SCREEN_WIDTH, ppu::SCREEN_HEIGHT, &window);
        let mut pixels = Pixels::new(self.width, self.height, surface_texture)?;
        let mut ui = Ui::new(
            self.debugger.clone(),
            self.width,
            self.height,
            scale_factor,
            &pixels,
        );

        pixels.set_clear_color(pixels::wgpu::Color {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 1.0,
        });

        log::debug!("Starting video loop");
        let emu = self.debugger.emu.clone();
        event_loop.run(move |event, _, control_flow| {
            if input.update(&event) {
                if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                if let Some(scale_factor) = input.scale_factor() {
                    ui.scale(scale_factor as f32);
                }

                if let Some(size) = input.window_resized() {
                    pixels.resize_surface(size.width, size.height);
                    ui.resize(size.width, size.height);
                }

                window.request_redraw();
            }

            match event {
                Event::WindowEvent { event, .. } => ui.handle_event(&event),
                Event::RedrawRequested(_) => {
                    {
                        if let Ok(gb) = emu.try_read() {
                            VideoDriver::draw(&gb, pixels.get_frame());
                        }
                    }

                    ui.prepare(&window);

                    let render_result = pixels.render_with(|encoder, render_target, context| {
                        context.scaling_renderer.render(encoder, render_target);

                        ui.render(encoder, render_target, context)?;

                        Ok(())
                    });

                    if render_result
                        .map_err(|e| error!("Render failed {}", e))
                        .is_err()
                    {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                _ => (),
            }
        });
    }

    fn draw(emu: &RwLockReadGuard<GameBoy>, frame: &mut [u8]) {
        // frame.copy_from_slice(emu.ppu().buffer());
    }

    fn draw_test(line_idx: usize, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let _x = i % ppu::SCREEN_WIDTH as usize;
            let y = i / ppu::SCREEN_WIDTH as usize;

            let rgba = if y == line_idx {
                [0x00, 0xFA, 0xFF, 0xFF]
            } else {
                [0x00, 0x00, 0x00, 0x00]
            };

            pixel.copy_from_slice(&rgba);
        }
    }
}
