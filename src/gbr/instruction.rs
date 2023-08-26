use std::fmt::Display;

use byteorder::{ByteOrder, LittleEndian};
use num::FromPrimitive;

use super::GbError;

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum Opcode {
    Nop = 0x00,
    IncB = 0x04,
    DecB = 0x05,
    IncC = 0x0C,
    DecC = 0x0D,
    LdBd8 = 0x06,
    LdCd8 = 0x0E,
    Stop = 0x10,
    LdDEd16 = 0x11,
    IncDE = 0x13,
    RlA = 0x17,
    LdADE = 0x1A,
    Jr = 0x18,
    LdEd8 = 0x1E,
    Jrnz = 0x20,
    Jrz = 0x28,
    LdHLd16 = 0x21,
    LdHLincA = 0x22,
    IncHL = 0x23,
    LdLd8 = 0x2E,
    LdSPd16 = 0x31,
    LdHLdecA = 0x32,
    DecA = 0x3D,
    LdAd8 = 0x3E,
    LdCA = 0x4F,
    LdDA = 0x57,
    LdHA = 0x67,
    LdHLA = 0x77,
    LdAE = 0x7B,
    AddAB = 0x80,
    SubAL = 0x95,
    XorA = 0xAF,
    PopBC = 0xC1,
    PushBC = 0xC5,
    Ret = 0xC9,
    Prefix = 0xCB,
    Calla16 = 0xCD,
    Ldha8A = 0xE0,
    Lda16A = 0xEA,
    LdhCA = 0xE2,
    LdhAa8 = 0xF0,
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

pub enum InstructionType {
    Nop,
    Stop,
    Arithmetic(ArithmeticType),
    JumpRelative(JumpCondition, i8),
    Load8(SingleRegType, SourceType),
    Load16(DoubleRegType, u16),
    Store(DestType, SingleRegType, PostStore), // target, source, post store
    Push(DoubleRegType),
    Pop(DoubleRegType),
    Call(CallMode),
    Ret,
}

pub enum ArithmeticType {
    Inc8(SingleRegType),
    Inc16(DoubleRegType),
    Dec(SingleRegType),
    Add(SingleRegType, SingleRegType), // target, source
    Sub(SingleRegType, SingleRegType), // target, source
    Cmp(SingleRegType, CompareType),
    Rl(SingleRegType),
    Rlc(SingleRegType),
    Sla(SingleRegType),
    Xor(SingleRegType, SingleRegType), // target, source
    TestBit(SingleRegType, u8),
}

impl Display for ArithmeticType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inc8(reg) => write!(f, "{}++", reg),
            Self::Inc16(reg) => write!(f, "{}++", reg),
            Self::Dec(reg) => write!(f, "{}--", reg),
            Self::Add(dest, src) => write!(f, "{} += {}", dest, src),
            Self::Sub(dest, src) => write!(f, "{} -= {}", dest, src),
            Self::Cmp(dest, src) => write!(f, "{} == {}", dest, src),
            Self::Rl(reg) => write!(f, "Rl({})", reg),
            Self::Rlc(reg) => write!(f, "Rlc({})", reg),
            Self::Sla(reg) => write!(f, "Sla({})", reg),
            Self::Xor(dest, src) => write!(f, "{} = {} ^ {}", dest, src, dest),
            Self::TestBit(reg, bit) => write!(f, "TestBit({}, {})", reg, bit),
        }
    }
}

pub enum CompareType {
    Imm(u8),
    Reg(SingleRegType),
}

impl Display for CompareType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Imm(val) => write!(f, "{:#04X}", val),
            Self::Reg(reg) => write!(f, "{}", reg),
        }
    }
}

pub enum SourceType {
    RegImm(SingleRegType),
    RegAddr(DoubleRegType),
    Imm8(u8),
    Imm16(u16),
    Addr(u16),
    IoPortImm(u8),
    IoPortReg(SingleRegType),
}

impl Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RegImm(reg) => write!(f, "{}", reg),
            Self::RegAddr(reg) => write!(f, "Addr({})", reg),
            Self::Imm8(val) => write!(f, "{:#04X}", val),
            Self::Imm16(val) => write!(f, "{:#06X}", val),
            Self::Addr(addr) => write!(f, "Addr({:#06X})", addr),
            Self::IoPortImm(val) => write!(f, "IO({:#04X})", val),
            Self::IoPortReg(reg) => write!(f, "IO({})", reg),
        }
    }
}

pub enum DestType {
    Addr(u16),
    RegAddr(DoubleRegType),
    IoPort(u8),
    IoPortReg(SingleRegType),
}

impl Display for DestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Addr(addr) => write!(f, "{:#06X}", addr),
            Self::RegAddr(reg) => write!(f, "Addr({})", reg),
            Self::IoPort(val) => write!(f, "IO({:#04X})", val),
            Self::IoPortReg(reg) => write!(f, "IO({})", reg),
        }
    }
}

pub enum SingleRegType {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
}

impl Display for SingleRegType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::A => write!(f, "A"),
            Self::B => write!(f, "B"),
            Self::C => write!(f, "C"),
            Self::D => write!(f, "D"),
            Self::E => write!(f, "E"),
            Self::F => write!(f, "F"),
            Self::H => write!(f, "H"),
            Self::L => write!(f, "L"),
        }
    }
}

pub enum DoubleRegType {
    BC,
    DE,
    HL,
    SP,
}

impl Display for DoubleRegType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BC => write!(f, "BC"),
            Self::DE => write!(f, "DE"),
            Self::HL => write!(f, "HL"),
            Self::SP => write!(f, "SP"),
        }
    }
}

pub enum JumpCondition {
    Always,
    Zero,
    NotZero,
    Carry,
    NotCarry,
}

impl Display for JumpCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Always => write!(f, ""),
            Self::Zero => write!(f, "Z"),
            Self::NotZero => write!(f, "NZ"),
            Self::Carry => write!(f, "C"),
            Self::NotCarry => write!(f, "NC"),
        }
    }
}

pub enum PostStore {
    Inc,
    Dec,
    None,
}

impl Display for PostStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inc => write!(f, "HL+"),
            Self::Dec => write!(f, "HL-"),
            Self::None => write!(f, ""),
        }
    }
}

pub enum CallMode {
    Absolute(u16),
}

impl Display for CallMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Absolute(addr) => write!(f, "Abs({:#06X})", addr),
        }
    }
}

pub struct Instruction {
    instr: InstructionType,
    length: u16,
    cycles: u8,
}

impl Instruction {
    pub fn decode(memory: &[u8]) -> Result<Self, GbError> {
        use ArithmeticType::*;
        use DoubleRegType::*;
        use InstructionType::*;
        use JumpCondition::*;
        use SingleRegType::*;
        use SourceType::*;

        let byte = memory[1];
        let word = LittleEndian::read_u16(&memory[1..3]);

        let opcode = Opcode::from_u8(memory[0]).ok_or(GbError::UnknownInstruction(memory[0]))?;
        let instr = match opcode {
            Opcode::Nop => Nop,
            Opcode::Stop => Stop,
            Opcode::IncB => Arithmetic(Inc8(B)),
            Opcode::IncC => Arithmetic(Inc8(C)),
            Opcode::IncDE => Arithmetic(Inc16(DE)),
            Opcode::IncHL => Arithmetic(Inc16(HL)),
            Opcode::DecA => Arithmetic(Dec(A)),
            Opcode::DecB => Arithmetic(Dec(B)),
            Opcode::DecC => Arithmetic(Dec(C)),
            Opcode::AddAB => Arithmetic(Add(A, B)),
            Opcode::SubAL => Arithmetic(Sub(A, L)),
            Opcode::RlA => Arithmetic(Rl(A)),
            Opcode::XorA => Arithmetic(Xor(A, A)),
            Opcode::Cpd8 => Arithmetic(Cmp(A, CompareType::Imm(byte))),
            Opcode::Jr => JumpRelative(Always, byte as i8),
            Opcode::Jrz => JumpRelative(Zero, byte as i8),
            Opcode::Jrnz => JumpRelative(NotZero, byte as i8),
            Opcode::LdAd8 => Load8(A, Imm8(byte)),
            Opcode::LdBd8 => Load8(B, Imm8(byte)),
            Opcode::LdCd8 => Load8(C, Imm8(byte)),
            Opcode::LdEd8 => Load8(E, Imm8(byte)),
            Opcode::LdLd8 => Load8(L, Imm8(byte)),
            Opcode::LdSPd16 => Load16(SP, word),
            Opcode::LdDEd16 => Load16(DE, word),
            Opcode::LdADE => Load8(A, RegAddr(DE)),
            Opcode::LdHLd16 => Load16(HL, word),
            Opcode::LdHLincA => Store(DestType::RegAddr(HL), A, PostStore::Inc),
            Opcode::LdHLdecA => Store(DestType::RegAddr(HL), A, PostStore::Dec),
            Opcode::LdHLA => Store(DestType::RegAddr(HL), A, PostStore::None),
            Opcode::LdCA => Load8(C, RegImm(A)),
            Opcode::LdDA => Load8(D, RegImm(A)),
            Opcode::LdHA => Load8(H, RegImm(A)),
            Opcode::LdAE => Load8(A, RegImm(E)),
            Opcode::Lda16A => Store(DestType::Addr(word), A, PostStore::None),
            Opcode::Ldha8A => Store(DestType::IoPort(byte), A, PostStore::None),
            Opcode::LdhAa8 => Load8(A, SourceType::IoPortImm(byte)),
            Opcode::LdhCA => Store(DestType::IoPortReg(C), A, PostStore::None),
            Opcode::PushBC => Push(BC),
            Opcode::PopBC => Pop(BC),
            Opcode::Calla16 => Call(CallMode::Absolute(word)),
            Opcode::Ret => Ret,
            Opcode::Prefix => {
                match CbOpcode::from_u8(byte).ok_or(GbError::UnknownCbInstruction(byte))? {
                    CbOpcode::RlcC => Arithmetic(Rlc(C)),
                    CbOpcode::SlaB => Arithmetic(Sla(B)),
                    CbOpcode::Bit7H => Arithmetic(TestBit(H, 7)),
                }
            }
        };

        Ok(Self {
            instr,
            length: Self::length(opcode),
            cycles: 0,
        })
    }

    pub fn len(&self) -> u16 {
        self.length
    }

    pub fn instr_type(&self) -> &InstructionType {
        &self.instr
    }

    pub fn cycles(&self) -> u8 {
        self.cycles
    }

    fn length(opcode: Opcode) -> u16 {
        match opcode {
            Opcode::Nop => 1,
            Opcode::DecB => 1,
            Opcode::IncB => 1,
            Opcode::IncC => 1,
            Opcode::DecC => 1,
            Opcode::LdBd8 => 2,
            Opcode::LdCd8 => 2,
            Opcode::LdEd8 => 2,
            Opcode::Stop => 1,
            Opcode::LdDEd16 => 3,
            Opcode::IncDE => 1,
            Opcode::RlA => 1,
            Opcode::LdADE => 1,
            Opcode::Jr => 2,
            Opcode::Jrnz => 2,
            Opcode::Jrz => 2,
            Opcode::LdHLd16 => 3,
            Opcode::LdHLincA => 1,
            Opcode::IncHL => 1,
            Opcode::LdLd8 => 2,
            Opcode::LdSPd16 => 3,
            Opcode::LdHLdecA => 1,
            Opcode::DecA => 1,
            Opcode::LdAd8 => 2,
            Opcode::LdCA => 1,
            Opcode::LdDA => 1,
            Opcode::LdHA => 1,
            Opcode::LdHLA => 1,
            Opcode::LdAE => 1,
            Opcode::AddAB => 1,
            Opcode::SubAL => 1,
            Opcode::XorA => 1,
            Opcode::PopBC => 1,
            Opcode::PushBC => 1,
            Opcode::Ret => 1,
            Opcode::Prefix => 2,
            Opcode::Calla16 => 3,
            Opcode::Ldha8A => 2,
            Opcode::Lda16A => 3,
            Opcode::LdhCA => 1,
            Opcode::LdhAa8 => 2,
            Opcode::Cpd8 => 2,
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use InstructionType::*;
        match &self.instr {
            Nop => write!(f, "Nop"),
            Stop => write!(f, "Stop"),
            Arithmetic(ar_type) => write!(f, "{}", ar_type),
            JumpRelative(cond, offset) => write!(f, "Jr{}, {:#04X}", cond, offset),
            Load8(reg, source) => write!(f, "Load {} {}", reg, source),
            Load16(reg, source) => write!(f, "Load {} {:#06X}", reg, source),
            Store(dest, reg, post_store) => write!(f, "Store {} {} {}", dest, reg, post_store),
            Push(reg) => write!(f, "Push {}", reg),
            Pop(reg) => write!(f, "Pop {}", reg),
            Call(mode) => write!(f, "Call {}", mode),
            Ret => write!(f, "Return"),
        }
    }
}
