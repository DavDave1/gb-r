use crate::gbr::{
    memory_map::{OBJ_ATTRIBUTE_TABLE_SIZE, OBJ_ATTRIBUTE_TABLE_START},
    ppu::Point,
    GbError,
};

const OBJ_ATTR_SIZE: usize = 4; // bytes
const OBJ_ATTR_COUNT: usize = OBJ_ATTRIBUTE_TABLE_SIZE / OBJ_ATTR_SIZE;

#[derive(Default, Clone)]
pub struct ObjAttribute {
    position: Point,
    tile_index: u8,
    bg_win_prio: bool,
    flip_y: bool,
    flip_x: bool,
    palette_id: u8,
}

impl ObjAttribute {
    fn write_attr(&mut self, attr_id: usize, value: u8) -> Result<(), GbError> {
        match attr_id {
            0 => self.position.y = value,
            1 => self.position.x = value,
            2 => self.tile_index = value,
            3 => {
                self.bg_win_prio = (value & 0b10000000) != 0;
                self.flip_y = (value & 0b01000000) != 0;
                self.flip_x = (value & 0b00100000) != 0;
                self.palette_id = value & 0b00010000;
            }
            _ => return Err(GbError::IllegalOp(format!("Write obj attr id {}", attr_id))),
        }

        Ok(())
    }

    pub fn position(&self) -> &Point {
        &self.position
    }
}

pub struct ObjAttributeMemory {
    attributes: Box<[ObjAttribute]>,
}

impl ObjAttributeMemory {
    pub fn new() -> Self {
        Self {
            attributes: vec![ObjAttribute::default(); OBJ_ATTR_COUNT].into_boxed_slice(),
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) -> Result<(), GbError> {
        let rel_addr = (addr - OBJ_ATTRIBUTE_TABLE_START) as usize;
        let obj_index = rel_addr / OBJ_ATTR_SIZE;
        let attr_id = rel_addr % OBJ_ATTR_SIZE;

        self.attributes[obj_index].write_attr(attr_id, value)
    }

    pub fn get_objs_at_line(&self, ly: u8) -> Vec<ObjAttribute> {
        let mut objs = vec![];
        objs.reserve(10);

        for attr in self.attributes.iter() {
            if attr.position().y < ly && attr.position().y + 8 > ly {
                objs.push(attr.clone());
            }

            if objs.len() == 10 {
                break;
            }
        }

        objs.sort_by(|l, r| l.position().x.partial_cmp(&r.position().x).unwrap());

        objs
    }
}
