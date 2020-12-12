use std::sync::{Arc, Mutex};

use cursive::traits::*;
use cursive::views::{LinearLayout, Panel, ResizedView, SelectView};
use cursive_flexi_logger_view::FlexiLoggerView;
use cursive_tabs::TabPanel;
use flexi_logger::{LogTarget, Logger};

use crate::debugger::asm_view::AsmView;
use crate::debugger::commands::*;
use crate::debugger::cpu_view::CpuView;
use crate::debugger::debugger::Debugger;
use crate::debugger::io_registers_view::IORegistersView;

use crate::gbr::game_boy::GameBoy;

pub struct DebuggerApp {
    debugger: Arc<Mutex<Debugger>>,
    siv: cursive::Cursive,
}

impl DebuggerApp {
    pub fn new(game_boy: GameBoy) -> Self {
        DebuggerApp {
            debugger: Arc::new(Mutex::new(Debugger::new(game_boy))),
            siv: cursive::default(),
        }
    }

    pub fn run(&mut self) {
        self.init_logger();
        self.init_ui();

        self.siv.set_fps(60);

        while self.siv.is_running() {
            self.siv.step();

            let mut d = self.debugger.lock().unwrap();

            if d.is_running() {
                d.step();
            }
        }
    }

    fn init_logger(&self) {
        Logger::with_env_or_str("info")
            .log_target(LogTarget::FileAndWriter(
                cursive_flexi_logger_view::cursive_flexi_logger(&self.siv),
            ))
            .directory("logs")
            .suppress_timestamp()
            .format(flexi_logger::colored_with_thread)
            .start()
            .expect("failed to initialize logger!");
    }

    fn init_ui(&mut self) {
        let debugger = self.debugger.clone();

        let commands_list = SelectView::new()
            .item("Run/Stop", Command::RunStop)
            .item("Run detached", Command::RunDetached)
            .item("Step", Command::Step)
            .item("Quit", Command::Quit)
            .on_submit(move |s, command| {
                let mut d = debugger.lock().unwrap();
                match *command {
                    Command::RunStop => command_run_stop(s, &mut d),
                    Command::RunDetached => command_run_detached(s, &mut d),
                    Command::Step => command_step(s, &mut d),
                    Command::Quit => s.quit(),
                }
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
            .with_tab("ASM", asm_view)
            .with_tab("CPU", cpu_view)
            .with_tab("IO Registers", regs_view);

        debugger_tabs.set_active_tab("ASM").unwrap();

        let debugger_view = ResizedView::with_full_width(debugger_tabs);

        let log_view = ResizedView::with_fixed_height(
            5,
            Panel::new(LinearLayout::vertical().child(FlexiLoggerView::scrollable()))
                .title("Log view"),
        );

        self.siv.add_fullscreen_layer(
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
