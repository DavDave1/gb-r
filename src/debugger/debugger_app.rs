use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use flume::Receiver;

use egui_tracing::EventCollector;
use pixels::{Pixels, SurfaceTexture};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

use super::{
    debugger::{Debugger, DebuggerCommand},
    ui::Ui,
};
use crate::gbr::{game_boy::GameBoy, ppu};

pub enum EmuState {
    Idle,
    Running,
    Error,
}

pub struct DebuggerApp {
    collector: EventCollector,
}

impl DebuggerApp {
    pub fn new() -> Self {
        let collector = egui_tracing::EventCollector::default();
        tracing_subscriber::registry()
            .with(EnvFilter::from_default_env())
            .with(collector.clone())
            .init();

        Self { collector }
    }

    pub fn run(&self, game_boy: Arc<RwLock<GameBoy>>) -> Result<(), Box<dyn std::error::Error>> {
        let debugger = Debugger::new();

        let gb_state = debugger.gb_state.clone();
        let asm_state = debugger.asm_state.clone();

        let (cmd_sig, emu_state_slot) = DebuggerApp::start_emu_thread(game_boy.clone(), debugger);

        let render_slot = game_boy.read().unwrap().ppu().render_watch();

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
            self.collector.clone(),
            gb_state,
            asm_state,
            cmd_sig,
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

    fn start_emu_thread(
        emu: Arc<RwLock<GameBoy>>,
        mut debugger: Debugger,
    ) -> (Sender<DebuggerCommand>, Receiver<EmuState>) {
        let (cmd_sig, cmd_slot) = channel::<DebuggerCommand>();
        let (emu_state_sig, emu_state_slot) = flume::bounded(1);

        let frame_time = Duration::from_secs_f64(1.0 / 59.7);

        std::thread::spawn(move || {
            let mut emu = emu.write().unwrap();

            let mut running = false;
            let mut stepping = false;

            let mut now = SystemTime::now();
            loop {
                {
                    let mut state = debugger.gb_state.write().unwrap();
                    *state = emu.collect_state();

                    let mut asm = debugger.asm_state.write().unwrap();
                    *asm = Debugger::disassemble(&emu);
                }

                let cmd = if !running {
                    cmd_slot.recv().ok()
                } else {
                    cmd_slot.try_recv().ok()
                };

                match cmd {
                    Some(DebuggerCommand::Run) => {
                        emu_state_sig.try_send(EmuState::Running).ok();
                        running = true;
                    }
                    Some(DebuggerCommand::Stop) => {
                        emu_state_sig.try_send(EmuState::Idle).ok();
                        emu.reset();
                        running = false;
                    }
                    Some(DebuggerCommand::Pause) => {
                        emu_state_sig.try_send(EmuState::Idle).ok();
                        running = false;
                    }
                    Some(DebuggerCommand::SetBreakpoint(pc)) => {
                        debugger.add_breakpoint(pc);
                    }
                    Some(DebuggerCommand::UnsetBreakpoint(pc)) => {
                        debugger.remove_breakpoint(pc);
                    }
                    Some(DebuggerCommand::Step) => stepping = true,
                    Some(DebuggerCommand::DumpVram) => log::info!("\n{}", emu.ppu().vram_dump()),
                    None => (),
                }

                let mut vblank_ev = false;
                if running || stepping {
                    stepping = false;

                    vblank_ev = match emu.step() {
                        Err(e) => {
                            log::error!("emu error: {}", e);
                            emu_state_sig.try_send(EmuState::Error).ok();
                            running = false;
                            false
                        }
                        Ok(ev) => ev,
                    };
                }

                if debugger.should_break(emu.cpu().read_pc()) {
                    running = false;
                }

                if running && vblank_ev {
                    let elapsed = SystemTime::now().duration_since(now).unwrap();
                    now = SystemTime::now();

                    if elapsed < frame_time {
                        std::thread::sleep(frame_time - elapsed);
                    }
                }
            }
        });

        (cmd_sig, emu_state_slot)
    }
}
