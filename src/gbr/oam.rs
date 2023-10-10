use crate::gbr::{
    memory_map::{OBJ_ATTRIBUTE_TABLE_SIZE, OBJ_ATTRIBUTE_TABLE_START},
    GbError,
};

const OBJ_ATTR_SIZE: usize = 4; // bytes
const OBJ_ATTR_COUNT: usize = OBJ_ATTRIBUTE_TABLE_SIZE / OBJ_ATTR_SIZE;

#[derive(Default, Clone, Copy, Debug)]
pub struct ObjAttribute {
    top: i16,
    left: i16,
    tile_index: u8,
    bg_win_prio: bool,
    flip_y: bool,
    flip_x: bool,
    palette_id: u8,
}

impl ObjAttribute {
    fn write_attr(&mut self, attr_id: usize, value: u8) -> Result<(), GbError> {
        match attr_id {
            0 => self.top = value as i16 - 16,
            1 => self.left = value as i16 - 8,
            2 => self.tile_index = value,
            3 => {
                self.bg_win_prio = (value & 0b10000000) != 0;
                self.flip_y = (value & 0b01000000) != 0;
                self.flip_x = (value & 0b00100000) != 0;
                self.palette_id = (value & 0b00010000) >> 4;
            }
            _ => return Err(GbError::IllegalOp(format!("Write obj attr id {}", attr_id))),
        }

        Ok(())
    }

    pub fn top(&self) -> i16 {
        self.top
    }

    pub fn bottom(&self) -> i16 {
        &self.top + 8
    }

    pub fn left(&self) -> i16 {
        self.left
    }

    pub fn right(&self) -> i16 {
        self.left + 8
    }

    pub fn tile_index(&self) -> u8 {
        self.tile_index
    }

    pub fn palette_id(&self) -> u8 {
        self.palette_id
    }

    pub fn flip_x(&self) -> bool {
        self.flip_x
    }

    pub fn flip_y(&self) -> bool {
        self.flip_y
    }

    pub fn bg_win_prio(&self) -> bool {
        self.bg_win_prio
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
            if attr.bottom() > ly as i16 && attr.top <= ly as i16 {
                objs.push(attr.clone());
            }

            if objs.len() == 10 {
                break;
            }
        }

        objs.sort_by(|l, r| l.left.partial_cmp(&r.left).unwrap());

        objs
    }

    pub fn state(&self) -> Vec<ObjAttribute> {
        let mut state = vec![ObjAttribute::default(); OBJ_ATTR_COUNT];
        state.copy_from_slice(&self.attributes);

        state
    }
}
