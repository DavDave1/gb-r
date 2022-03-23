use egui::{ClippedMesh, Context, TexturesDelta, TopBottomPanel};
use egui_wgpu_backend::{BackendError, RenderPass, ScreenDescriptor};
use pixels::wgpu;
use pixels::PixelsContext;
use std::sync::Arc;
use winit::{event::WindowEvent, window::Window};

use crate::gbr::cpu::CpuState;
use crate::gbr::io_registers::IORegisters;

use super::io_registers_view;
use super::{
    asm_view, cpu_view,
    debugger::{AsmState, Debugger},
};

struct UiState {
    show_asm_view: bool,
    show_cpu_view: bool,
    show_registers_view: bool,
    debugger: Arc<Debugger>,
    asm_state: AsmState,
    io_registers_state: IORegisters,
    cpu_state: CpuState,
}

impl UiState {
    fn new(debugger: Arc<Debugger>) -> Self {
        Self {
            show_asm_view: true,
            show_cpu_view: true,
            show_registers_view: true,
            debugger,
            asm_state: AsmState::default(),
            io_registers_state: IORegisters::default(),
            cpu_state: CpuState::default(),
        }
    }

    fn update_debug_data(&mut self) {
        if let Some(state) = self.debugger.asm_state() {
            self.asm_state = state;
        }
        if let Some(state) = self.debugger.io_registers_state() {
            self.io_registers_state = state;
        }
        if let Some(state) = self.debugger.cpu_state() {
            self.cpu_state = state;
        }
    }

    pub fn update(&mut self, ctx: &Context) {
        self.update_debug_data();

        TopBottomPanel::top("menubar_container").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Debug", |ui| {
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
                });
            });
        });

        egui::Window::new("Asm")
            .open(&mut self.show_asm_view)
            .show(ctx, |ui| {
                asm_view::show(&self.asm_state, ui);
            });

        egui::Window::new("CPU")
            .open(&mut self.show_cpu_view)
            .show(ctx, |ui| {
                cpu_view::show(&self.cpu_state, ui);
            });

        egui::Window::new("IO registers")
            .open(&mut self.show_registers_view)
            .show(ctx, |ui| {
                io_registers_view::show(&self.io_registers_state, ui);
            });
    }
}

pub struct Ui {
    ctx: Context,
    egui_state: egui_winit::State,
    screen_descriptor: ScreenDescriptor,
    rpass: RenderPass,
    textures: TexturesDelta,
    paint_jobs: Vec<ClippedMesh>,
    state: UiState,
}

impl Ui {
    pub fn new(
        debugger: Arc<Debugger>,
        width: u32,
        height: u32,
        scale_factor: f32,
        pixels: &pixels::Pixels,
    ) -> Self {
        let max_texure_size = pixels.device().limits().max_texture_dimension_2d as usize;

        let egui_state = egui_winit::State::from_pixels_per_point(max_texure_size, scale_factor);

        let screen_descriptor = ScreenDescriptor {
            physical_width: width,
            physical_height: height,
            scale_factor,
        };

        let rpass = RenderPass::new(pixels.device(), pixels.render_texture_format(), 1);

        Self {
            ctx: Context::default(),
            egui_state,
            screen_descriptor,
            rpass,
            paint_jobs: vec![],
            textures: TexturesDelta::default(),
            state: UiState::new(debugger),
        }
    }

    pub fn scale(&mut self, scale_factor: f32) {
        self.screen_descriptor.scale_factor = scale_factor;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.screen_descriptor.physical_width = width;
            self.screen_descriptor.physical_height = height;
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        self.egui_state.on_event(&self.ctx, event);
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
    ) -> Result<(), BackendError> {
        self.rpass
            .add_textures(&context.device, &context.queue, &self.textures)?;

        self.rpass.update_buffers(
            &context.device,
            &context.queue,
            &self.paint_jobs,
            &self.screen_descriptor,
        );

        self.rpass.execute(
            encoder,
            target,
            &self.paint_jobs,
            &self.screen_descriptor,
            None,
        )?;

        let textures = std::mem::take(&mut self.textures);
        self.rpass.remove_textures(textures)
    }
}
