#[macro_use]
extern crate enum_primitive;

#[macro_use]
extern crate lazy_static;

use std::path::PathBuf;
use std::sync::{Arc, RwLock};

mod debugger;
mod gbr;

use debugger::debugger_app::DebuggerApp;
use gbr::game_boy::GameBoy;

fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::try_init().ok();

    let boot_rom_filename = std::env::args().nth(1).and_then(|p| Some(PathBuf::from(p)));
    let cart_rom_filename = std::env::args().nth(2).and_then(|p| Some(PathBuf::from(p)));

    let gb_emu = Arc::new(RwLock::new(GameBoy::new(
        boot_rom_filename,
        cart_rom_filename,
    )));

    let app = DebuggerApp {};

    app.run(gb_emu).unwrap();
}
