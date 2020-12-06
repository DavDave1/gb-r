extern crate num;

use byteorder::{ByteOrder, LittleEndian};
use num::FromPrimitive;

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum Opcode {
    Nop = 0x00,
    DecB = 0x05,
    IncC = 0x0C,
    LdBd8 = 0x06,
    LdCd8 = 0x0E,
    Stop = 0x10,
    LdDEd16 = 0x11,
    IncDE = 0x13,
    RlA = 0x17,
    LdADE = 0x1A,
    Jrnz = 0x20,
    LdHLd16 = 0x21,
    LdHLincA = 0x22,
    IncHL = 0x23,
    LdSPd16 = 0x31,
    LdHLdecA = 0x32,
    LdAd8 = 0x3E,
    LdCA = 0x4F,
    LdHLA = 0x77,
    LdAE = 0x7B,
    AddAB = 0x80,
    SubAL = 0x95,
    XorA = 0xAF,
    PopBC = 0xC1,
    PushCB = 0xC5,
    Ret = 0xC9,
    Prefix = 0xCB,
    Calla16 = 0xCD,
    Ldha8A = 0xE0,
    LdhCA = 0xE2,
    Cpd8 = 0xFE,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum CbOpcode {
    RlcC = 0x11,
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

    pub fn opcode(&self) -> Option<Opcode> {
        Opcode::from_u8(self.opcode)
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
        match self
            .opcode()
            .expect("Cannot retreive length of unknown instruction")
        {
            Opcode::Nop => 1,
            Opcode::DecB => 1,
            Opcode::IncC => 1,
            Opcode::LdBd8 => 2,
            Opcode::LdCd8 => 2,
            Opcode::Stop => 1,
            Opcode::LdDEd16 => 3,
            Opcode::IncDE => 1,
            Opcode::RlA => 1,
            Opcode::LdADE => 1,
            Opcode::Jrnz => 2,
            Opcode::LdHLd16 => 3,
            Opcode::LdHLincA => 1,
            Opcode::IncHL => 1,
            Opcode::LdSPd16 => 3,
            Opcode::LdHLdecA => 1,
            Opcode::LdAd8 => 2,
            Opcode::LdCA => 1,
            Opcode::LdHLA => 1,
            Opcode::LdAE => 1,
            Opcode::AddAB => 1,
            Opcode::SubAL => 1,
            Opcode::XorA => 1,
            Opcode::PopBC => 1,
            Opcode::PushCB => 1,
            Opcode::Ret => 1,
            Opcode::Prefix => 2,
            Opcode::Calla16 => 3,
            Opcode::Ldha8A => 2,
            Opcode::LdhCA => 1,
            Opcode::Cpd8 => 2,
        }
    }
}
