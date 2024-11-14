# GB-R

GB-R is an experimental Game Boy emulator and debugger written in Rust.

## Project goal

The goal of this project is to experiment and learn low level systems programming and the Rust language.

## Bulding and running

GB-R can be built with

```
cargo build
```

and run with

```
cargo run -- ./data/DMG_ROM.bin <path_to_rom>
```

## Main dependencies

- [egui](https://github.com/emilk/egui) immediate mode GUI crate for the debugguer UI
- [pixels](https://github.com/parasyte/pixels) and [winit](https://github.com/rust-windowing/winit) crates for rendering 2D graphics (debugger UI and GB-R PPU output)
- [thiserror](https://github.com/dtolnay/thiserror) for error handling

## TODOs

Emulator:

- [x] CPU interpreter
- [x] Basic Picture Processing Unit
- [x] Memory Bank Controller Type 1
- [x] DMA unit
- [x] Interrupts
- [x] Input handling
- [ ] Scan line accurare PPU
- [ ] Audio Processing Unit
- [ ] Memory Bank Controllers Types 2 - 7

Debugger:

- [x] Disassembler
- [x] Breakpoints
- [x] VRAM dump
- [x] CPU registers view
- [x] Tilemap view
- [x] LCD status and control registers view
- [x] Interrupts view
- [x] Inputs register view
- [ ] Emulator thread error handling
