use log::error;

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

    pub fn cb_opcode(&self) -> Option<CbOpcode> {
        CbOpcode::from_u8(self.data[0])
    }

    pub fn byte(&self) -> u8 {
        self.data[0]
    }

    pub fn word(&self) -> u16 {
        LittleEndian::read_u16(&self.data[0..2])
    }

    pub fn length(&self) -> Result<u16, ()> {
        match self.opcode() {
            Some(Opcode::Nop) => Ok(1),
            Some(Opcode::DecB) => Ok(1),
            Some(Opcode::IncC) => Ok(1),
            Some(Opcode::LdBd8) => Ok(2),
            Some(Opcode::LdCd8) => Ok(2),
            Some(Opcode::Stop) => Ok(1),
            Some(Opcode::LdDEd16) => Ok(3),
            Some(Opcode::IncDE) => Ok(1),
            Some(Opcode::RlA) => Ok(1),
            Some(Opcode::LdADE) => Ok(1),
            Some(Opcode::Jrnz) => Ok(2),
            Some(Opcode::LdHLd16) => Ok(3),
            Some(Opcode::LdHLincA) => Ok(1),
            Some(Opcode::IncHL) => Ok(1),
            Some(Opcode::LdSPd16) => Ok(3),
            Some(Opcode::LdHLdecA) => Ok(1),
            Some(Opcode::LdAd8) => Ok(2),
            Some(Opcode::LdCA) => Ok(1),
            Some(Opcode::LdHLA) => Ok(1),
            Some(Opcode::LdAE) => Ok(1),
            Some(Opcode::AddAB) => Ok(1),
            Some(Opcode::SubAL) => Ok(1),
            Some(Opcode::XorA) => Ok(1),
            Some(Opcode::PopBC) => Ok(1),
            Some(Opcode::PushCB) => Ok(1),
            Some(Opcode::Ret) => Ok(1),
            Some(Opcode::Prefix) => Ok(2),
            Some(Opcode::Calla16) => Ok(3),
            Some(Opcode::Ldha8A) => Ok(2),
            Some(Opcode::LdhCA) => Ok(1),
            Some(Opcode::Cpd8) => Ok(2),
            None => {
                error!("Cannot retrieve length of unknown instruction");
                Err(())
            }
        }
    }
}
