use std::sync::{Arc, Mutex};

use cursive::traits::*;
use cursive::views::{
    Button, Dialog, DummyView, LinearLayout, Panel, ResizedView, ScrollView, SelectView,
    TextContent, TextView,
};

use crate::debugger::commands::*;
use crate::debugger::debugger::Debugger;
use crate::debugger::debugger_view::DebuggerView;

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
        self.init_ui();

        self.siv.run();
    }

    fn init_ui(&mut self) {
        let debugger = self.debugger.clone();

        let commands_list = SelectView::new()
            .item("Step", Command::Step)
            .item("Quit", Command::Quit)
            .on_submit(move |s, command| {
                let mut d = debugger.lock().unwrap();
                match *command {
                    Command::Step => command_step(s, &mut d),
                    Command::Quit => s.quit(),
                }
            })
            .with_name("commands");

        let commands_panel = ResizedView::with_fixed_width(
            20,
            Panel::new(LinearLayout::vertical().child(commands_list)).title("Commands"),
        );

        let debugger_view = ResizedView::with_full_width(
            Panel::new(
                LinearLayout::vertical().child(ResizedView::with_full_height(DebuggerView::new(
                    self.debugger.clone(),
                ))),
            )
            .title("Debugger View"),
        );

        self.siv.add_fullscreen_layer(
            LinearLayout::vertical().child(ResizedView::with_full_height(
                LinearLayout::horizontal()
                    .child(commands_panel)
                    .child(debugger_view),
            )),
        );
    }
}
