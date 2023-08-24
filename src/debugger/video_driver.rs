use log::error;
use pixels::{Pixels, SurfaceTexture};
use std::sync::Arc;
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

use crate::debugger::{debugger::Debugger, ui::Ui};
use crate::gbr::ppu;

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
        let mut pixels = Pixels::new(ppu::SCREEN_WIDTH, ppu::SCREEN_HEIGHT, surface_texture)?;
        let mut ui = Ui::new(
            self.debugger.clone(),
            &event_loop,
            self.width,
            self.height,
            scale_factor,
            &pixels,
        );

        pixels.clear_color(pixels::wgpu::Color {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 1.0,
        });

        log::debug!("Starting video loop");
        let render_slot = self.debugger.emu.read().unwrap().ppu().render_watch();
        event_loop.run(move |event, _, control_flow| {
            if input.update(&event) {
                if input.key_pressed(VirtualKeyCode::Escape)
                    || input.close_requested()
                    || input.destroyed()
                {
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                if let Some(scale_factor) = input.scale_factor() {
                    ui.scale(scale_factor as f32);
                }

                if let Some(size) = input.window_resized() {
                    pixels.resize_surface(size.width, size.height).unwrap();
                    ui.resize(size.width, size.height);
                }

                window.request_redraw();
            }

            match event {
                Event::WindowEvent { event, .. } => ui.handle_event(&event),
                Event::RedrawRequested(_) => {
                    {
                        if let Ok(frame) = render_slot.try_recv() {
                            pixels.frame_mut().copy_from_slice(&frame);
                        }
                    }

                    ui.prepare(&window);

                    let render_result = pixels.render_with(|encoder, render_target, context| {
                        context.scaling_renderer.render(encoder, render_target);

                        ui.render(encoder, render_target, context);

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
}
