#[macro_use]
extern crate enum_primitive;

use std::path::Path;
use std::sync::{Arc, RwLock};

mod debugger;
mod gbr;

use debugger::debugger_app::DebuggerApp;
use gbr::game_boy::GameBoy;

fn main() {
    let boot_rom_filename = std::env::args().nth(1).unwrap();
    let cart_rom_filename = std::env::args().nth(2).unwrap();

    let gb_emu = Arc::new(RwLock::new(GameBoy::new(
        Path::new(&boot_rom_filename),
        Path::new(&cart_rom_filename),
    )));
    let mut app = DebuggerApp::new(gb_emu);
    app.run();
}
