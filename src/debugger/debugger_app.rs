use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, RwLock};
use std::thread;

use cursive::views::{LinearLayout, Panel, ResizedView, SelectView};
use cursive::{traits::*, CursiveExt};
use cursive_flexi_logger_view::FlexiLoggerView;
use cursive_tabs::TabPanel;
use flexi_logger::Logger;

use crate::debugger::asm_view::AsmView;
use crate::debugger::commands::*;
use crate::debugger::cpu_view::CpuView;
use crate::debugger::debugger::Debugger;
use crate::debugger::io_registers_view::IORegistersView;

use crate::gbr::game_boy::GameBoy;
use crate::gbr::video_driver::VideoDriver;

fn backend_init() -> std::io::Result<Box<dyn cursive::backend::Backend>> {
    let backend = cursive::backends::crossterm::Backend::init()?;
    let buffered_backend = cursive_buffered_backend::BufferedBackend::new(backend);
    Ok(Box::new(buffered_backend))
}

pub struct DebuggerApp {
    debugger: Arc<Debugger>,
}

impl DebuggerApp {
    pub fn new(game_boy: Arc<RwLock<GameBoy>>) -> Self {
        DebuggerApp {
            debugger: Arc::new(Debugger::new(game_boy.clone())),
        }
    }

    pub fn run(&mut self) {
        let mut siv = cursive::Cursive::new();
        siv.set_autorefresh(true);

        let (start_sig, start_slot) = channel::<i64>();

        self.init_logger(&siv);
        self.init_ui(&mut siv, start_sig);

        let debugger = self.debugger.clone();
        thread::spawn(move || loop {
            let steps_count = start_slot.recv().unwrap();

            let mut curr_step = 0;
            while steps_count < 0 || curr_step < steps_count {
                debugger.step();
                if start_slot.try_recv().is_ok() {
                    break;
                }
                curr_step += 1;
            }
        });

        let game_boy = self.debugger.emu.clone();
        thread::spawn(|| {
            log::info!("creating video");
            let mut driver = VideoDriver::new(game_boy, 190, 144);

            driver.start().expect("Failed to start video driver");
        });

        siv.try_run_with(backend_init)
            .expect("Failed tu run cursive app");
    }

    fn init_logger(&self, siv: &cursive::Cursive) {
        Logger::try_with_env_or_str("info")
            .expect("Could not create logger")
            .log_to_file_and_writer(
                flexi_logger::FileSpec::default()
                    .directory("logs")
                    .suppress_timestamp(),
                cursive_flexi_logger_view::cursive_flexi_logger(&siv),
            )
            .format(flexi_logger::colored_with_thread)
            .start()
            .expect("failed to initialize logger!");
    }

    fn init_ui(&mut self, siv: &mut cursive::Cursive, start_sig: Sender<i64>) {
        let commands_list = SelectView::new()
            .item("Run/Stop", Command::RunStop)
            .item("Step", Command::Step)
            .item("Quit", Command::Quit)
            .on_submit(move |s, command| match *command {
                Command::RunStop => command_run_stop(s, start_sig.clone()),
                Command::Step => command_step(s, start_sig.clone()),
                Command::Quit => s.quit(),
            })
            .with_name("commands");

        let commands_panel = ResizedView::with_fixed_width(
            20,
            Panel::new(LinearLayout::vertical().child(commands_list)).title("Commands"),
        );

        let asm_view = ResizedView::with_full_height(AsmView::new(self.debugger.clone()));
        let cpu_view = ResizedView::with_full_height(CpuView::new(self.debugger.clone()));
        let regs_view = ResizedView::with_full_height(IORegistersView::new(self.debugger.clone()));

        let mut debugger_tabs = TabPanel::new()
            .with_tab(asm_view.with_name("ASM"))
            .with_tab(cpu_view.with_name("CPU"))
            .with_tab(regs_view.with_name("IO Registers"));

        debugger_tabs.set_active_tab("ASM").unwrap();

        let debugger_view = ResizedView::with_full_width(debugger_tabs);

        let log_view = ResizedView::with_fixed_height(
            5,
            Panel::new(LinearLayout::vertical().child(FlexiLoggerView::scrollable()))
                .title("Log view"),
        );

        siv.add_fullscreen_layer(
            LinearLayout::vertical()
                .child(ResizedView::with_full_height(
                    LinearLayout::horizontal()
                        .child(commands_panel)
                        .child(debugger_view),
                ))
                .child(log_view),
        );
    }
}
