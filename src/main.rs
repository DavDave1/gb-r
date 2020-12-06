#[macro_use]
extern crate enum_primitive;

use std::path::Path;

mod gbr;

use gbr::game_boy::GameBoy;

fn main() {
    let boot_rom_filename = std::env::args().nth(1).unwrap();
    let cart_rom_filename = std::env::args().nth(2).unwrap();

    let mut gb_emu = GameBoy::new(Path::new(&boot_rom_filename), Path::new(&cart_rom_filename));

    gb_emu.run();
}
