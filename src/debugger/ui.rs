use std::collections::HashSet;
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};

use egui::ClippedPrimitive;
use egui::{Context, TexturesDelta, TopBottomPanel};
use egui_wgpu::renderer::ScreenDescriptor;
use egui_wgpu::Renderer;
use flume::Receiver;
use pixels::wgpu;
use pixels::PixelsContext;
use winit::event_loop::EventLoopWindowTarget;
use winit::{event::WindowEvent, window::Window};

use crate::gbr::game_boy::GbState;

use super::debugger::{AsmState, DebuggerCommand};
use super::debugger_app::EmuState;
use super::palette_view::PaletteView;
use super::tiles_view::TilesView;
use super::{asm_view, cpu_view, mbc_view};
use super::{interrupts_view, io_registers_view};

struct UiState {
    show_asm_view: bool,
    show_cpu_view: bool,
    show_registers_view: bool,
    show_tiles: bool,
    gb_state_next: Arc<RwLock<GbState>>,
    gb_state: GbState,
    cmd_sig: Sender<DebuggerCommand>,
    tiles_view: TilesView,
    palette_view: PaletteView,
    emu_state: EmuState,
    emu_state_slot: Receiver<EmuState>,
    breakpoints: HashSet<u16>,
    asm: AsmState,
}

impl UiState {
    fn new(
        gb_state: Arc<RwLock<GbState>>,
        cmd_sig: Sender<DebuggerCommand>,
        emu_state_slot: Receiver<EmuState>,
        asm: AsmState,
    ) -> Self {
        Self {
            show_asm_view: true,
            show_cpu_view: true,
            show_registers_view: true,
            show_tiles: true,
            gb_state_next: gb_state,
            gb_state: GbState::default(),
            cmd_sig,
            tiles_view: TilesView::default(),
            palette_view: PaletteView::new(),
            emu_state: EmuState::Idle,
            emu_state_slot,
            breakpoints: HashSet::new(),
            asm,
        }
    }

    fn update_debug_data(&mut self) {
        if let Ok(state) = self.emu_state_slot.try_recv() {
            self.emu_state = state;
        }

        if let Ok(state) = self.gb_state_next.try_read() {
            self.gb_state = state.clone();
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
                                self.cmd_sig.send(DebuggerCommand::Stop).unwrap();
                            }

                            if ui.button("Pause").clicked() {
                                self.cmd_sig.send(DebuggerCommand::Pause).unwrap();
                            }
                        }
                        EmuState::Idle => {
                            if ui.button("Start").clicked() {
                                self.cmd_sig.send(DebuggerCommand::Run).unwrap();
                            }

                            if ui.button("Step").clicked() {
                                self.cmd_sig.send(DebuggerCommand::Step).unwrap();
                            }
                        }
                        EmuState::Error => {
                            if ui.button("Stop").clicked() {
                                self.cmd_sig.send(DebuggerCommand::Stop).unwrap();
                            }
                        }
                    }

                    if ui.button("Dump VRAM").clicked() {
                        self.cmd_sig.send(DebuggerCommand::DumpVram).unwrap();
                    }
                });
            });

        egui::SidePanel::new(egui::panel::Side::Left, "ASM")
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    cpu_view::show(&mut self.gb_state.cpu, ui);
                    ui.separator();
                    io_registers_view::show(&self.gb_state.io_registers, ui);
                    ui.separator();
                    interrupts_view::show(&mut self.gb_state.ir_handler, ui);
                    ui.separator();
                    mbc_view::show(&mut self.gb_state.mbc, ui);
                    ui.separator();
                    asm_view::show(
                        &self.cmd_sig,
                        &self.asm,
                        &self.gb_state.cpu,
                        &mut self.breakpoints,
                        ui,
                    );
                });
            });

        egui::SidePanel::new(egui::panel::Side::Right, "tiles")
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.heading("Tiles");
                    self.tiles_view.show(
                        &self.gb_state.ppu.tiles_list,
                        &self.gb_state.ppu.bg_palette,
                        ui,
                    );
                    ui.separator();
                    ui.heading("LCD Ctrl");
                    ui.label(format!("{}", self.gb_state.ppu.lcd_control));
                    ui.separator();
                    ui.heading("LCD Status");
                    ui.label(format!("{}", self.gb_state.ppu.lcd_status));
                    ui.heading("Viewport");
                    ui.label(format!(
                        "Viewport X, Y: ({}, {}), LY: {}",
                        self.gb_state.ppu.viewport.x,
                        self.gb_state.ppu.viewport.y,
                        self.gb_state.ppu.ly
                    ));
                    ui.label(format!(
                        "Win X, Y: ({}, {})",
                        self.gb_state.ppu.win_pos.x, self.gb_state.ppu.win_pos.y
                    ));
                    ui.separator();
                    ui.horizontal_wrapped(|ui| {
                        ui.label("BG Palette: ");
                        self.palette_view.show(&self.gb_state.ppu.bg_palette, ui);
                    });

                    ui.horizontal_wrapped(|ui| {
                        ui.label("Obj Palette 0: ");
                        self.palette_view.show(&self.gb_state.ppu.obj_palette0, ui);
                    });

                    ui.horizontal_wrapped(|ui| {
                        ui.label("Obj Palette 1: ");
                        self.palette_view.show(&self.gb_state.ppu.obj_palette1, ui);
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
        gb_state: Arc<RwLock<GbState>>,
        cmd_sig: Sender<DebuggerCommand>,
        emu_state_slot: Receiver<EmuState>,
        asm: AsmState,
        event_loop: &EventLoopWindowTarget<T>,
        width: u32,
        height: u32,
        scale_factor: f32,
        device: &wgpu::Device,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        let max_texture_size = device.limits().max_texture_dimension_2d as usize;

        let mut egui_state = egui_winit::State::new(event_loop);
        egui_state.set_max_texture_side(max_texture_size);
        egui_state.set_pixels_per_point(scale_factor);
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: scale_factor,
        };

        let renderer = Renderer::new(device, texture_format, None, 1);

        Self {
            ctx: Context::default(),
            egui_state,
            screen_descriptor,
            renderer,
            paint_jobs: vec![],
            textures: TexturesDelta::default(),
            state: UiState::new(gb_state, cmd_sig, emu_state_slot, asm),
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
