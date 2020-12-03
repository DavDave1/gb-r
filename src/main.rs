#[macro_use]
extern crate enum_primitive;

use std::path::Path;

mod gbr;

use gbr::game_boy::GameBoy;

fn main() {
    let boot_rom_filename = std::env::args().nth(1).unwrap();

    let mut gb_emu = GameBoy::new();

    gb_emu
        .load_boot_rom(Path::new(&boot_rom_filename))
        .expect("Failed to laod boot rom");

    gb_emu.run();
}
