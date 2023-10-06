pub mod lcd_control_register;
pub mod lcd_status_register;
pub mod palette;
pub mod pixel_processor;
pub mod rgba;
pub mod tile;

use byteorder::{ByteOrder, LittleEndian};

use self::{
    lcd_control_register::LcdControlRegister,
    lcd_status_register::{LcsStatusRegister, ScreenMode},
    palette::Palette,
    pixel_processor::PixelProcessor,
    rgba::Rgba,
    tile::{Tile, TileData},
};
use crate::gbr::{
    memory_map::{VRAM_SIZE, VRAM_START},
    GbError,
};

use super::{
    interrupts::{InterruptHandler, InterruptType},
    oam::ObjAttributeMemory,
};

// rlative to VRAM base addr
const TILE_BLOCK0_START: u16 = 0x0000;
const TILE_BLOCK0_END: u16 = 0x07FF;
const TILE_BLOCK1_START: u16 = TILE_BLOCK0_END + 1;
const TILE_BLOCK1_END: u16 = 0x0FFF;
const TILE_BLOCK2_START: u16 = TILE_BLOCK1_END + 1;
const TILE_BLOCK2_END: u16 = 0x17FF;

const TILEMAP_BLOCK0_START: u16 = 0x9800 - VRAM_START;
const TILEMAP_BLOCK1_START: u16 = 0x9C00 - VRAM_START;

pub const SCREEN_WIDTH: u32 = 160;
pub const SCREEN_HEIGHT: u32 = 144;

pub const TILE_WIDTH: u32 = 8;
pub const TILE_HEIGHT: u32 = 8;
const TILE_DATA_SIZE: usize = 16;

pub const TILE_BLOCK_SIZE: usize = 128;

const VBLANK_LINE: u8 = 144;
const LAST_LINE: u8 = 153;

const MODE_2_DOTS: u16 = 80;
const MODE_3_DOTS_MAX: u16 = MODE_2_DOTS + 289;

const DOTS_PER_LINE: u16 = 456;

const LCD_CTRL_REG_ADDR: u16 = 0xFF40;
const LCD_STAT_REG_ADDR: u16 = 0xFF41;
const VIEWPORT_Y_REG_ADDR: u16 = 0xFF42;
const VIEWPORT_X_REG_ADDR: u16 = 0xFF43;
const LY_REG_ADDR: u16 = 0xFF44;
const LYC_REG_ADDR: u16 = 0xFF45;
const BG_PALETTE_REG_ADDR: u16 = 0xFF47;
const OBJ_PALETTE0_REG_ADDR: u16 = 0xFF48;
const OBJ_PALETTE1_REG_ADDR: u16 = 0xFF49;
const WIN_POS_Y_REG_ADDR: u16 = 0xFF4A;
const WIN_POS_X_REG_ADDR: u16 = 0xFF4B;

pub type ScreenBuffer = Vec<u8>;
pub type TileList = Vec<Tile>;

#[derive(Default, Clone)]
pub struct Point {
    pub x: u8,
    pub y: u8,
}

#[derive(Default, Clone)]
pub struct PpuState {
    pub lcd_control: LcdControlRegister,
    pub lcd_status: LcsStatusRegister,
    pub bg_palette: Palette,
    pub obj_palette0: Palette,
    pub obj_palette1: Palette,
    pub ly: u8,
    pub lyc: u8,
    pub viewport: Point,
    pub win_pos: Point,
    pub tiles_list: Vec<Tile>,
}

pub struct PPU {
    vram: Box<[u8]>,
    lcd_control: LcdControlRegister,
    lcd_status: LcsStatusRegister,
    bg_palette: Palette,
    obj_palette0: Palette,
    obj_palette1: Palette,
    ly: u8,
    lyc: u8,
    viewport: Point,
    win_pos: Point,
    tiles: TileData,
    render_ch: (flume::Sender<ScreenBuffer>, flume::Receiver<ScreenBuffer>),
    dots: u16,
    pixel_processor: PixelProcessor,
}

impl PPU {
    pub fn new() -> Self {
        Self {
            vram: vec![0; VRAM_SIZE].into_boxed_slice(),
            lcd_control: LcdControlRegister::default(),
            lcd_status: LcsStatusRegister::default(),
            bg_palette: Default::default(),
            obj_palette0: Default::default(),
            obj_palette1: Default::default(),
            ly: 0,
            lyc: 0,
            viewport: Point::default(),
            win_pos: Point::default(),
            tiles: TileData::new(),
            render_ch: flume::bounded(1),
            dots: 0,
            pixel_processor: PixelProcessor::new(),
        }
    }

    pub fn reset(&mut self) {
        self.vram.fill(0);
        self.lcd_control = LcdControlRegister::default();
        self.lcd_status = LcsStatusRegister::default();
        self.bg_palette = Default::default();
        self.ly = 0;
        self.lyc = 0;
        self.viewport = Point::default();
        self.win_pos = Point::default();
        self.tiles.clear();
        self.dots = 0;
        self.pixel_processor = PixelProcessor::new();
    }

    pub fn step(
        &mut self,
        ir_handler: &mut InterruptHandler,
        oam: &ObjAttributeMemory,
        cpu_cycles: u8,
    ) -> Result<bool, GbError> {
        if !self.lcd_control.display_enable {
            return Ok(false);
        }

        self.dots += cpu_cycles as u16;

        if self.dots > DOTS_PER_LINE {
            if self.lcd_status.mode.get() != ScreenMode::HBlank
                && self.lcd_status.mode.get() != ScreenMode::VBlank
            {
                return Err(GbError::IllegalOp(format!(
                    "unexpected mode {} during hblank",
                    self.lcd_status.mode.get()
                )));
            }
            self.ly += 1;
            self.lcd_status.lyc_equals_ly.set(self.lyc == self.ly);
            self.dots -= DOTS_PER_LINE;
        }

        let mut vblank_ev = false;
        if self.ly > LAST_LINE {
            self.render()?;
            self.ly = 0;
            vblank_ev = true;
        } else if self.ly >= VBLANK_LINE {
            self.lcd_status.mode.set(ScreenMode::VBlank);
        } else if self.dots <= MODE_2_DOTS {
            self.lcd_status.mode.set(ScreenMode::SreachingOAM);
        } else if self.lcd_status.mode.get() == ScreenMode::SreachingOAM {
            self.pixel_processor.start(
                oam,
                self.ly,
                self.dots,
                &self.viewport,
                &self.lcd_control,
                &self.vram,
                &self.tiles,
                &self.bg_palette,
            );
            self.lcd_status.mode.set(ScreenMode::TransferringData);
        } else if !self.pixel_processor.finished() {
            self.pixel_processor.process(
                self.ly,
                self.dots,
                &self.viewport,
                &self.lcd_control,
                &self.vram,
                &self.tiles,
                &self.bg_palette,
            );
            if self.dots > MODE_3_DOTS_MAX {
                log::error!("mode 3 out of bounds {}", self.dots);
            }
        } else {
            self.lcd_status.mode.set(ScreenMode::HBlank);
        }

        self.set_interrupts(ir_handler);

        Ok(vblank_ev)
    }

    pub fn read_byte(&self, addr: u16) -> Result<u8, GbError> {
        if self.lcd_control.display_enable {
            return Ok(0xFF);
        }

        Ok(self.vram[(addr - VRAM_START) as usize])
    }

    pub fn read_word(&self, addr: u16) -> Result<u16, GbError> {
        if self.lcd_control.display_enable {
            return Ok(0xFFFF);
        }

        Ok(LittleEndian::read_u16(
            &self.vram[(addr - VRAM_START) as usize..],
        ))
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) -> Result<(), GbError> {
        if self.lcd_control.display_enable {
            return Ok(());
        }

        let local_addr = (addr - VRAM_START) as usize;

        self.vram[local_addr] = value;

        if local_addr <= TILE_BLOCK2_END as usize {
            let is_lsb = local_addr % 2 == 0;

            if is_lsb {
                self.tiles
                    .write_line(local_addr, self.vram[local_addr + 1], value);
            } else {
                self.tiles
                    .write_line(local_addr, value, self.vram[local_addr - 1]);
            }
        }

        Ok(())
    }

    pub fn read_reg(&self, addr: u16) -> Result<u8, GbError> {
        match addr {
            LCD_CTRL_REG_ADDR => Ok(self.lcd_control.into()),
            LCD_STAT_REG_ADDR => Ok(self.lcd_status.read()),
            VIEWPORT_Y_REG_ADDR => Ok(self.viewport.y),
            VIEWPORT_X_REG_ADDR => Ok(self.viewport.x),
            LY_REG_ADDR => Ok(self.ly),
            LYC_REG_ADDR => Ok(self.lyc),
            BG_PALETTE_REG_ADDR => Ok(self.bg_palette.into()),
            OBJ_PALETTE0_REG_ADDR => Ok(self.obj_palette0.into()),
            OBJ_PALETTE1_REG_ADDR => Ok(self.obj_palette1.into()),
            WIN_POS_Y_REG_ADDR => Ok(self.win_pos.y),
            WIN_POS_X_REG_ADDR => Ok(self.win_pos.x),
            _ => Err(GbError::IllegalOp(format!(
                "Write to invalid PPU reg {:#06X}",
                addr
            ))),
        }
    }

    pub fn write_reg(&mut self, addr: u16, value: u8) -> Result<(), GbError> {
        match addr {
            LCD_CTRL_REG_ADDR => self.lcd_control = value.into(),
            LCD_STAT_REG_ADDR => self.lcd_status.write(value),
            VIEWPORT_Y_REG_ADDR => self.viewport.y = value,
            VIEWPORT_X_REG_ADDR => self.viewport.x = value,
            LY_REG_ADDR => return Err(GbError::IllegalOp("Cannot write to LY register".into())),
            LYC_REG_ADDR => self.lyc = value,
            BG_PALETTE_REG_ADDR => self.bg_palette = value.into(),
            OBJ_PALETTE0_REG_ADDR => self.obj_palette0 = value.into(),
            OBJ_PALETTE1_REG_ADDR => self.obj_palette1 = value.into(),
            WIN_POS_Y_REG_ADDR => self.win_pos.y = value,
            WIN_POS_X_REG_ADDR => self.win_pos.x = value,
            _ => {
                return Err(GbError::IllegalOp(format!(
                    "Write to invalid PPU reg {:#06X}",
                    addr
                )));
            }
        }

        Ok(())
    }

    pub fn render_watch(&self) -> flume::Receiver<ScreenBuffer> {
        self.render_ch.1.clone()
    }

    pub fn render(&mut self) -> Result<(), GbError> {
        self.render_ch
            .0
            .try_send(self.pixel_processor.screen_buffer.clone())
            .ok();
        self.pixel_processor.screen_buffer.fill(0);
        Ok(())
    }

    fn set_interrupts(&mut self, ir_handler: &mut InterruptHandler) {
        if self.lcd_status.mode.changed_to(ScreenMode::VBlank) {
            ir_handler.set(InterruptType::VBlank);
        }

        if self.lcd_status.is_mode_0_ir()
            || self.lcd_status.is_mode_1_ir()
            || self.lcd_status.is_mode_2_ir()
            || self.lcd_status.is_lyc_ir()
        {
            ir_handler.set(InterruptType::LcdStat)
        }
    }

    pub fn state(&self) -> PpuState {
        PpuState {
            lcd_control: self.lcd_control,
            lcd_status: self.lcd_status,
            bg_palette: self.bg_palette,
            obj_palette0: self.obj_palette0,
            obj_palette1: self.obj_palette1,
            ly: self.ly,
            lyc: self.lyc,
            viewport: self.viewport.clone(),
            win_pos: self.win_pos.clone(),
            tiles_list: self.tiles.list().clone(),
        }
    }

    pub fn vram_dump(&self) -> String {
        const BYTES_PER_LINE: usize = 32;

        let mut dump = "".to_string();
        for (addr, data) in self.vram.chunks_exact(BYTES_PER_LINE).enumerate() {
            dump.push_str(&format!(
                "{:#06X}: ",
                VRAM_START as usize + addr * BYTES_PER_LINE
            ));

            for b in 0..BYTES_PER_LINE {
                dump.push_str(&format!("{:02X} ", data[b]));
            }

            dump.pop();
            dump.push('\n');
        }

        dump
    }
}
