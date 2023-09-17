use std::collections::HashSet;

use egui::ClippedPrimitive;
use egui::{Context, TexturesDelta, TopBottomPanel};
use egui_wgpu::renderer::ScreenDescriptor;
use egui_wgpu::Renderer;
use pixels::wgpu;
use pixels::PixelsContext;
use winit::event_loop::EventLoopWindowTarget;
use winit::{event::WindowEvent, window::Window};

use super::debugger::DebuggerCommand;
use super::debugger::EmuState;
use super::io_registers_view;
use super::palette_view::PaletteView;
use super::tiles_view::TilesView;
use super::{asm_view, cpu_view, debugger::Debugger};

struct UiState {
    show_asm_view: bool,
    show_cpu_view: bool,
    show_registers_view: bool,
    show_tiles: bool,
    debugger: Debugger,
    tiles_view: TilesView,
    palette_view: PaletteView,
    emu_state: EmuState,
    breakpoints: HashSet<u16>,
}

impl UiState {
    fn new(debugger: Debugger) -> Self {
        Self {
            show_asm_view: true,
            show_cpu_view: true,
            show_registers_view: true,
            show_tiles: true,
            debugger,
            tiles_view: TilesView::default(),
            palette_view: PaletteView::new(),
            emu_state: EmuState::Idle,
            breakpoints: HashSet::new(),
        }
    }

    fn update_debug_data(&mut self) {
        if let Some(state) = self.debugger.emu_state() {
            self.emu_state = state;
        }
    }

    pub fn update(&mut self, ctx: &Context) {
        self.update_debug_data();

        TopBottomPanel::top("menubar_container").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Window", |ui| {
                    if ui.button("Asm view...").clicked() {
                        self.show_asm_view = true;
                        ui.close_menu();
                    }

                    if ui.button("CPU view...").clicked() {
                        self.show_cpu_view = true;
                        ui.close_menu();
                    }

                    if ui.button("IO registers view...").clicked() {
                        self.show_registers_view = true;
                        ui.close_menu();
                    }

                    if ui.button("Tiles view...").clicked() {
                        self.show_tiles = true;
                        ui.close_menu();
                    }
                });
            });
        });

        egui::TopBottomPanel::top("toolbar")
            .max_height(60.0)
            .show(ctx, |ui| {
                ui.horizontal_top(|ui| {
                    match self.emu_state {
                        EmuState::Running => {
                            if ui.button("Stop").clicked() {
                                self.debugger.send_cmd(DebuggerCommand::Stop).unwrap();
                            }

                            if ui.button("Pause").clicked() {
                                self.debugger.send_cmd(DebuggerCommand::Pause).unwrap();
                            }
                        }
                        EmuState::Idle => {
                            if ui.button("Start").clicked() {
                                self.debugger.send_cmd(DebuggerCommand::Run).unwrap();
                            }

                            if ui.button("Step").clicked() {
                                self.debugger.send_cmd(DebuggerCommand::Step).unwrap();
                            }
                        }
                        EmuState::Error => {
                            if ui.button("Stop").clicked() {
                                self.debugger.send_cmd(DebuggerCommand::Stop).unwrap();
                            }
                        }
                    }

                    if ui.button("Dump VRAM").clicked() {
                        self.debugger.send_cmd(DebuggerCommand::DumpVram).unwrap();
                    }
                });
            });

        let mut gb_state = self.debugger.gb_state.read().unwrap().clone();

        egui::SidePanel::new(egui::panel::Side::Left, "ASM")
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    cpu_view::show(&mut gb_state.cpu, ui);
                    ui.separator();
                    io_registers_view::show(&gb_state.io_registers, ui);
                    ui.separator();
                    asm_view::show(&self.debugger, &gb_state.cpu, &mut self.breakpoints, ui);
                });
            });

        egui::SidePanel::new(egui::panel::Side::Right, "tiles")
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.heading("Tiles");
                    self.tiles_view
                        .show(&gb_state.ppu.tiles_list, &gb_state.ppu.bg_palette, ui);
                    ui.separator();
                    ui.heading("LCD Ctrl");
                    ui.label(format!("{}", gb_state.ppu.lcd_control));
                    ui.separator();
                    ui.heading("LCD Status");
                    ui.label(format!("{}", gb_state.ppu.lcd_status));
                    ui.heading("Viewport");
                    ui.label(format!(
                        "X: {}, Y: {}, LY: {}",
                        gb_state.ppu.viewport.0, gb_state.ppu.viewport.1, gb_state.ppu.ly
                    ));
                    ui.separator();
                    ui.horizontal_wrapped(|ui| {
                        ui.label("Palette: ");
                        self.palette_view.show(&gb_state.ppu.bg_palette, ui);
                    });
                });
            });
    }
}

pub struct Ui {
    ctx: Context,
    egui_state: egui_winit::State,
    screen_descriptor: ScreenDescriptor,
    renderer: Renderer,
    textures: TexturesDelta,
    paint_jobs: Vec<ClippedPrimitive>,
    state: UiState,
}

impl Ui {
    pub fn new<T>(
        debugger: Debugger,
        event_loop: &EventLoopWindowTarget<T>,
        width: u32,
        height: u32,
        scale_factor: f32,
        pixels: &pixels::Pixels,
    ) -> Self {
        let max_texture_size = pixels.device().limits().max_texture_dimension_2d as usize;

        let mut egui_state = egui_winit::State::new(event_loop);
        egui_state.set_max_texture_side(max_texture_size);
        egui_state.set_pixels_per_point(scale_factor);
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: scale_factor,
        };

        let renderer = Renderer::new(pixels.device(), pixels.render_texture_format(), None, 1);

        Self {
            ctx: Context::default(),
            egui_state,
            screen_descriptor,
            renderer,
            paint_jobs: vec![],
            textures: TexturesDelta::default(),
            state: UiState::new(debugger),
        }
    }

    pub fn scale(&mut self, scale_factor: f32) {
        self.screen_descriptor.pixels_per_point = scale_factor;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.screen_descriptor.size_in_pixels = [width, height];
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        let _ = self.egui_state.on_event(&self.ctx, event);
    }

    pub fn prepare(&mut self, window: &Window) {
        let raw_input = self.egui_state.take_egui_input(window);

        let output = self.ctx.run(raw_input, |ctx| {
            self.state.update(ctx);
        });

        self.textures.append(output.textures_delta);
        self.egui_state
            .handle_platform_output(window, &self.ctx, output.platform_output);
        self.paint_jobs = self.ctx.tessellate(output.shapes);
    }

    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        context: &PixelsContext,
    ) {
        for (id, image_delta) in &self.textures.set {
            self.renderer
                .update_texture(&context.device, &context.queue, *id, image_delta);
        }
        self.renderer.update_buffers(
            &context.device,
            &context.queue,
            encoder,
            &self.paint_jobs,
            &self.screen_descriptor,
        );

        // Render egui with WGPU
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            self.renderer
                .render(&mut rpass, &self.paint_jobs, &self.screen_descriptor);
        }

        // Cleanup
        let textures = std::mem::take(&mut self.textures);
        for id in &textures.free {
            self.renderer.free_texture(id);
        }
    }
}
