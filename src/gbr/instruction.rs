extern crate num;

use byteorder::{ByteOrder, LittleEndian};
use num::FromPrimitive;

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum Opcode {
    Nop = 0x00,
    IncC = 0x0C,
    LdCd8 = 0x0E,
    Stop = 0x10,
    LdDEd16 = 0x11,
    LdADE = 0x1A,
    Jrnz = 0x20,
    LdHLd16 = 0x21,
    LdSPd16 = 0x31,
    LdHLdecA = 0x32,
    LdAd8 = 0x3E,
    LdHLA = 0x77,
    AddAB = 0x80,
    SubAL = 0x95,
    XorA = 0xAF,
    Prefix = 0xCB,
    LdHa8A = 0xE0,
    LdCA = 0xE2
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum CbOpcode {
    SlaB = 0x20,
    Bit7H = 0x7C,
}
}

pub struct Instruction {
    opcode: u8,
    data: [u8; 2],
}

impl Instruction {
    pub fn new(memory: &[u8]) -> Self {
        Instruction {
            opcode: memory[0],
            data: [memory[1], memory[2]],
        }
    }

    pub fn opcode(&self) -> Opcode {
        match Opcode::from_u8(self.opcode) {
            Some(opcode) => opcode,
            None => panic!("Unknown instruction {:#04X}", self.opcode),
        }
    }

    pub fn cb_opcode(&self) -> CbOpcode {
        match CbOpcode::from_u8(self.data[0]) {
            Some(opcode) => opcode,
            None => panic!("Unknown cb instruction {:#04X}", self.data[0]),
        }
    }

    pub fn byte(&self) -> u8 {
        self.data[0]
    }

    pub fn word(&self) -> u16 {
        LittleEndian::read_u16(&self.data[0..2])
    }

    pub fn length(&self) -> u16 {
        match self.opcode() {
            Opcode::Nop => 1,
            Opcode::IncC => 1,
            Opcode::LdCd8 => 2,
            Opcode::Stop => 1,
            Opcode::LdDEd16 => 3,
            Opcode::LdADE => 1,
            Opcode::Jrnz => 2,
            Opcode::LdHLd16 => 3,
            Opcode::LdSPd16 => 3,
            Opcode::LdHLdecA => 1,
            Opcode::LdAd8 => 2,
            Opcode::LdHLA => 1,
            Opcode::AddAB => 1,
            Opcode::SubAL => 1,
            Opcode::XorA => 1,
            Opcode::Prefix => 2,
            Opcode::LdHa8A => 2,
            Opcode::LdCA => 1,
        }
    }
}
