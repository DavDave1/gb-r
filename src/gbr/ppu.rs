use byteorder::{ByteOrder, LittleEndian};

use crate::gbr::io_registers::background_palette::GrayShade;

use super::{
    io_registers::{
        background_palette,
        lcd_control_register::LcdControlRegister,
        lcd_status_register::{LcsStatusRegister, ScreenMode},
    },
    memory_map::VIDEO_RAM_SIZE,
    GbError,
};

pub const TILE_DATA_END: u16 = 0x17FF;

pub const SCREEN_WIDTH: u32 = 190;
pub const SCREEN_HEIGHT: u32 = 144;
pub const NUM_CHANNELS: u32 = 4; // rgba
pub const SCREEN_SIZE: usize = (SCREEN_WIDTH * SCREEN_HEIGHT * NUM_CHANNELS) as usize;

const RENDER_FRAME_WIDTH: u32 = 256;
const RENDER_FRAME_HEIGHT: u32 = 256;
const RENDER_FRAME_SIZE: usize = (RENDER_FRAME_WIDTH * RENDER_FRAME_HEIGHT * NUM_CHANNELS) as usize;

pub const TILE_WIDTH: u32 = 8;
pub const TILE_HEIGHT: u32 = 8;
const TILE_RENDER_SIZE: usize = (TILE_WIDTH * TILE_HEIGHT * NUM_CHANNELS) as usize;
const TILE_DATA_SIZE: usize = 16;

pub const TILE_BLOCK_SIZE: usize = 128;

const VBLANK_LINE: u8 = 145;
const LAST_LINE: u8 = 153;

const MODE_2_DOTS: u16 = 80;
const MODE_3_DOTS: u16 = 172;
const MODE_0_DOTS: u16 = 204;

const DOTS_PER_LINE: u16 = 456;

pub type ScreenBuffer = Vec<u8>;
pub type TileList = Vec<Tile>;

#[derive(Clone, Copy)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub fn black() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }
    }

    pub fn dark() -> Self {
        Self {
            r: 84,
            g: 84,
            b: 84,
            a: 255,
        }
    }

    pub fn light() -> Self {
        Self {
            r: 168,
            g: 168,
            b: 168,
            a: 255,
        }
    }

    pub fn white() -> Self {
        Self {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        }
    }
}

impl Default for Rgba {
    fn default() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Tile {
    pub pixels: [[Rgba; TILE_WIDTH as usize]; TILE_HEIGHT as usize],
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            pixels: [[Rgba::white(); TILE_WIDTH as usize]; TILE_HEIGHT as usize],
        }
    }
}

impl Tile {
    fn from_data(data: &[u8], palette: &background_palette::BackgroundPalette) -> Self {
        let mut tile = Self::default();

        // Tile data is represented as 2 bytes per line
        for (x, line) in data.chunks_exact(2).enumerate() {
            for y in 0..8 {
                let shift = 7 - y;

                let msb = line[0] >> shift & 0b1;
                let lsb = line[1] >> shift & 0b1;
                let color_id = (msb << 1u16) + lsb;

                let shade = match color_id {
                    0 => palette.color_0(),
                    1 => palette.color_1(),
                    2 => palette.color_2(),
                    3 => palette.color_3(),
                    _ => panic!("Tile color id out of range"),
                };

                tile.pixels[y][x] = shade_to_rgba(shade);
            }
        }

        fn shade_to_rgba(shade: GrayShade) -> Rgba {
            match shade {
                GrayShade::Black => Rgba::black(),
                GrayShade::DarkGray => Rgba::dark(),
                GrayShade::LightGray => Rgba::light(),
                GrayShade::White => Rgba::white(),
            }
        }

        tile
    }
}

#[derive(Default)]
pub struct PpuState {
    pub lcd_control: LcdControlRegister,
    pub lcd_status: LcsStatusRegister,
    pub bg_palette: background_palette::BackgroundPalette,
    pub ly: u8,
    pub lyc: u8,
    pub viewport: (u8, u8),
    pub tile_list: Vec<Tile>,
}

pub struct PPU {
    vram: Box<[u8]>,
    lcd_control: LcdControlRegister,
    lcd_status: LcsStatusRegister,
    bg_palette: background_palette::BackgroundPalette,
    ly: u8,
    lyc: u8,
    viewport: (u8, u8),
    screen_buffer: Vec<u8>,
    render_buffer: Vec<u8>,
    tile_list: Vec<Tile>,
    tile_list_updated: bool,
    render_ch: (flume::Sender<ScreenBuffer>, flume::Receiver<ScreenBuffer>),
    dots: u16,
}

impl PPU {
    pub fn new() -> Self {
        Self {
            vram: vec![0; VIDEO_RAM_SIZE].into_boxed_slice(),
            lcd_control: LcdControlRegister::default(),
            lcd_status: LcsStatusRegister::default(),
            bg_palette: Default::default(),
            ly: 0,
            lyc: 0,
            viewport: (0, 0),
            screen_buffer: vec![0; SCREEN_SIZE],
            render_buffer: vec![0; RENDER_FRAME_SIZE],
            tile_list: vec![Tile::default(); 3 * TILE_BLOCK_SIZE],
            tile_list_updated: false,
            render_ch: flume::bounded(1),
            dots: 0,
        }
    }

    pub fn step(&mut self, cpu_cycles: u8) -> Result<(), GbError> {
        if !self.lcd_control.display_enable {
            return Ok(());
        }

        self.dots += cpu_cycles as u16;

        if self.dots > DOTS_PER_LINE {
            self.ly += 1;
            self.lcd_status.lyc_equals_ly = self.lcd_status.lyc_check_enable && self.lyc == self.ly;
            self.dots -= DOTS_PER_LINE;
        }

        if self.ly > LAST_LINE {
            self.render()?;
            self.ly = 0;
        } else if self.ly >= VBLANK_LINE {
            self.lcd_status.mode = ScreenMode::VBlank;
        } else if self.dots <= MODE_0_DOTS {
            self.lcd_status.mode = ScreenMode::SreachingOAM;
            self.update_tile_list()?;
        } else if self.dots <= MODE_3_DOTS {
            self.lcd_status.mode = ScreenMode::TransferringData;
        } else {
            self.lcd_status.mode = ScreenMode::HBlank;
        }

        Ok(())
    }

    pub fn read_byte(&self, addr: u16) -> Result<u8, GbError> {
        if addr as usize >= VIDEO_RAM_SIZE {
            return Err(GbError::AddrOutOfBounds(addr));
        }

        if self.lcd_control.display_enable {
            return Ok(0xFF);
        }

        Ok(self.vram[addr as usize])
    }

    pub fn read_word(&self, addr: u16) -> Result<u16, GbError> {
        if self.lcd_control.display_enable {
            return Ok(0xFFFF);
        }

        if addr as usize >= VIDEO_RAM_SIZE - 1 {
            return Err(GbError::AddrOutOfBounds(addr));
        }

        Ok(LittleEndian::read_u16(&self.vram[addr as usize..]))
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) -> Result<(), GbError> {
        if addr as usize >= VIDEO_RAM_SIZE {
            return Err(GbError::AddrOutOfBounds(addr));
        }

        if !self.lcd_control.display_enable {
            self.vram[addr as usize] = value;
        }

        Ok(())
    }

    pub fn read_reg(&self, addr: u16) -> Result<u8, GbError> {
        match addr {
            0x0040 => Ok(self.lcd_control.into()),
            0x0041 => Ok(self.lcd_status.into()),
            0x0042 => Ok(self.viewport.0),
            0x0043 => Ok(self.viewport.1),
            0x0044 => Ok(self.ly),
            0x0045 => Ok(self.lyc),
            0x0047 => Ok(self.bg_palette.into()),
            _ => Err(GbError::IllegalOp(format!(
                "Write to invalid PPU reg {:#06X}",
                addr
            ))),
        }
    }

    pub fn write_reg(&mut self, addr: u16, value: u8) -> Result<(), GbError> {
        match addr {
            0x0040 => Ok(self.lcd_control = value.into()),
            0x0041 => Ok(self.lcd_status = value.into()),
            0x0042 => Ok(self.viewport.0 = value),
            0x0043 => Ok(self.viewport.1 = value),
            0x0044 => Err(GbError::IllegalOp("Cannot write to LY register".into())),
            0x0045 => Ok(self.lyc = value),
            0x0047 => Ok(self.bg_palette = value.into()),
            _ => Err(GbError::IllegalOp(format!(
                "Write to invalid PPU reg {:#06X}",
                addr
            ))),
        }
    }

    pub fn render_watch(&self) -> flume::Receiver<ScreenBuffer> {
        self.render_ch.1.clone()
    }

    pub fn render(&mut self) -> Result<(), GbError> {
        self.render_ch.0.try_send(self.screen_buffer.clone()).ok();
        self.tile_list_updated = false;
        Ok(())
    }

    fn update_tile_list(&mut self) -> Result<(), GbError> {
        let mut tile_addr = 0;

        let mut tile_index = 0usize;
        while tile_addr < TILE_DATA_END {
            let mut tile_data = [0u8; TILE_DATA_SIZE];

            for chunk in tile_data.chunks_exact_mut(2) {
                chunk.copy_from_slice(&self.read_word(tile_addr as u16)?.to_be_bytes());
                tile_addr += 2;
            }

            self.tile_list[tile_index] = Tile::from_data(&tile_data, &self.bg_palette);
            tile_index += 1;
        }

        self.tile_list_updated = true;

        Ok(())
    }

    pub fn state(&self) -> PpuState {
        PpuState {
            lcd_control: self.lcd_control,
            lcd_status: self.lcd_status,
            bg_palette: self.bg_palette,
            ly: self.ly,
            lyc: self.lyc,
            viewport: self.viewport,
            tile_list: self.tile_list.clone(),
        }
    }
}
