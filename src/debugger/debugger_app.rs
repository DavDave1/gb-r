use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};

use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

use super::{debugger::Debugger, ui::Ui};
use crate::gbr::game_boy::{self, GbrEvent, GenericInput, InputType};
use crate::gbr::joypad::{Buttons, Directions};
use crate::gbr::{game_boy::GameBoy, ppu};

#[derive(Clone)]
pub struct Settings {
    keymap: HashMap<VirtualKeyCode, GenericInput>,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            keymap: Settings::default_keymap(),
        }
    }

    fn keymap(&self) -> &HashMap<VirtualKeyCode, GenericInput> {
        &self.keymap
    }

    fn default_keymap() -> HashMap<VirtualKeyCode, GenericInput> {
        let mut keymap = HashMap::new();

        keymap.insert(VirtualKeyCode::A, GenericInput::Button(Buttons::A));
        keymap.insert(VirtualKeyCode::Z, GenericInput::Button(Buttons::B));
        keymap.insert(VirtualKeyCode::Space, GenericInput::Button(Buttons::Select));
        keymap.insert(VirtualKeyCode::Return, GenericInput::Button(Buttons::Start));
        keymap.insert(
            VirtualKeyCode::Right,
            GenericInput::Direction(Directions::Right),
        );
        keymap.insert(
            VirtualKeyCode::Left,
            GenericInput::Direction(Directions::Left),
        );
        keymap.insert(VirtualKeyCode::Up, GenericInput::Direction(Directions::Up));
        keymap.insert(
            VirtualKeyCode::Down,
            GenericInput::Direction(Directions::Down),
        );

        keymap
    }
}

pub struct DebuggerApp {
    settings: Settings,
}

impl DebuggerApp {
    pub fn new() -> Self {
        env_logger::init();

        Self {
            settings: Settings::new(),
        }
    }

    pub fn run(&self, gb: Arc<RwLock<GameBoy>>) -> Result<(), Box<dyn std::error::Error>> {
        let debugger = Debugger::new();

        let gb_state = debugger.gb_state_recv();
        let asm_state = debugger.asm_state_recv();
        let render_slot = gb.read().unwrap().ppu().render_watch();

        let (ev_sender, emu_state_slot) = game_boy::start_gb_thread(gb, debugger);

        log::debug!("create window");
        let event_loop = EventLoop::new();

        let width = 800;
        let height = 600;

        let window = WindowBuilder::new()
            .with_title("gb-r")
            .with_maximized(true)
            .with_min_inner_size(LogicalSize::new(width as f64, height as f64))
            .build(&event_loop)?;

        log::debug!("create render surface");
        let scale_factor = window.scale_factor() as f32;

        let surface_texture = SurfaceTexture::new(ppu::SCREEN_WIDTH, ppu::SCREEN_HEIGHT, &window);
        let mut pixels = Pixels::new(ppu::SCREEN_WIDTH, ppu::SCREEN_HEIGHT, surface_texture)?;

        pixels.clear_color(pixels::wgpu::Color {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 1.0,
        });

        let mut ui = Ui::new(
            gb_state,
            asm_state,
            ev_sender.clone(),
            emu_state_slot,
            &event_loop,
            width,
            height,
            scale_factor,
            pixels.device(),
            pixels.render_texture_format(),
        );

        log::debug!("Starting video loop");

        let mut input = WinitInputHelper::new();

        let settings = self.settings.clone();

        event_loop.run(move |event, _, control_flow| {
            if input.update(&event) {
                if input.key_pressed(VirtualKeyCode::Escape)
                    || input.close_requested()
                    || input.destroyed()
                {
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                DebuggerApp::send_key(&input, &ev_sender, &settings);

                if let Some(scale_factor) = input.scale_factor() {
                    if scale_factor > 0.0 {
                        ui.scale(scale_factor as f32);
                    }
                }

                if let Some(size) = input.window_resized() {
                    if size.width > 0 && size.height > 0 {
                        pixels.resize_surface(size.width, size.height).unwrap();
                        ui.resize(size.width, size.height);
                    }
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
                        .map_err(|e| log::error!("Render failed {}", e))
                        .is_err()
                    {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                _ => (),
            }
        });
    }

    fn send_key(input: &WinitInputHelper, ev_sender: &Sender<GbrEvent>, settings: &Settings) {
        for (key, button) in settings.keymap() {
            if input.key_pressed(*key) {
                ev_sender
                    .send(GbrEvent::Input(InputType::Pressed(button.clone())))
                    .ok();
            }
            if input.key_released(*key) {
                ev_sender
                    .send(GbrEvent::Input(InputType::Released(button.clone())))
                    .ok();
            }
        }
    }
}
