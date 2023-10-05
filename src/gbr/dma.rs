use super::{
    mbc::MBC,
    memory_map::{
        CART_RAM_END, CART_RAM_START, CART_ROM_ACTIVE_BANK_END, CART_ROM_BANK0_START,
        OBJ_ATTRIBUTE_TABLE_SIZE, OBJ_ATTRIBUTE_TABLE_START, VRAM_END, VRAM_START,
    },
    oam::ObjAttributeMemory,
    ppu::PPU,
    GbError,
};

enum SourceType {
    Cart,
    Vram,
}

pub struct DMA {
    source_addr: u16,
    curr_index: u16,
    source_type: SourceType,
    started: bool,
}

impl DMA {
    pub fn new() -> Self {
        Self {
            source_addr: 0,
            curr_index: 0,
            source_type: SourceType::Cart,
            started: false,
        }
    }

    pub fn step(
        &mut self,
        ppu: &PPU,
        mbc: &MBC,
        oam: &mut ObjAttributeMemory,
        cycles: u8,
    ) -> Result<(), GbError> {
        if self.started {
            for _ in 0..cycles {
                let data = match self.source_type {
                    SourceType::Vram => ppu.read_byte(self.source_addr + self.curr_index),
                    SourceType::Cart => mbc.read_byte(self.source_addr + self.curr_index),
                }?;

                oam.write_byte(OBJ_ATTRIBUTE_TABLE_START + self.curr_index, data)?;

                self.curr_index += 1;

                if self.curr_index as usize == OBJ_ATTRIBUTE_TABLE_SIZE {
                    self.started = false;
                    break;
                }
            }
        }
        Ok(())
    }

    pub fn write_reg(&mut self, src: u8) {
        self.source_addr = (src as u16) << 7;
        self.started = true;
        self.curr_index = 0;

        match self.source_addr {
            CART_ROM_BANK0_START..=CART_ROM_ACTIVE_BANK_END => self.source_type = SourceType::Cart,
            VRAM_START..=VRAM_END => self.source_type = SourceType::Vram,
            CART_RAM_START..=CART_RAM_END => self.source_type = SourceType::Cart,
            _ => {
                log::warn!("Starting DMA from invalid addres {:#06X}", self.source_addr);
                self.started = false;
            }
        }
    }

    pub fn read_reg(&self) -> u8 {
        (self.source_addr >> 8) as u8
    }
}
