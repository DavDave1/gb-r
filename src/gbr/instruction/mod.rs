pub mod opcode;

use std::fmt::Display;

use byteorder::{ByteOrder, LittleEndian};
use enum_primitive::FromPrimitive;

use super::GbError;

use self::opcode::{CbOpcode, Opcode};

#[derive(Clone)]
pub enum InstructionType {
    Nop,
    Stop,
    Halt,
    FlipCarry,
    ClearCarry,
    MasterInterrupt(bool), // enable/disable
    Arithmetic(ArithmeticType),
    Jump(JumpCondition, JumpType),
    Load(GenericRegType, Source),
    LoadWithOp(GenericRegType, Source, PostLoad),
    LoadSP(DoubleRegType),
    Store(Dest, Source),
    StoreWithOp(Dest, Source, PostStore),
    StoreSP(u16),
    Push(DoubleRegType),
    Pop(DoubleRegType),
    Call(u16, JumpCondition),
    Ret(JumpCondition),
    RetI,
}

#[derive(Clone)]
pub enum JumpType {
    Offset(i8),
    Addr(u16),
    RegAddr(DoubleRegType),
}

impl Display for JumpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Offset(offset) => write!(f, "Rel({:#04X})", offset),
            Self::Addr(addr) => write!(f, "Abs({:#06X})", addr),
            Self::RegAddr(reg) => write!(f, "Abs(Addr({}))", reg),
        }
    }
}

#[derive(Clone)]
pub enum ArithmeticType {
    Inc(GenericRegType),
    IncAddr(DoubleRegType),
    Dec(GenericRegType),
    DecAddr(DoubleRegType),
    Add(SingleRegType, Operand),
    Adc(SingleRegType, Operand),
    Add16(DoubleRegType, DoubleRegType),
    AddSP(i8),
    Sub(SingleRegType, Operand),
    Sbc(SingleRegType, Operand),
    Cmp(SingleRegType, Operand),
    Rl(GenericRegType, bool),  // target, clear_z_flag
    Rlc(GenericRegType, bool), // target, clear_z_flag
    Rr(GenericRegType, bool),  // target, clear_z_flag
    Rrc(GenericRegType, bool), // target, clear_z_flag
    Sla(GenericRegType),
    Sra(GenericRegType),
    Srl(GenericRegType),
    And(SingleRegType, Operand),
    Or(SingleRegType, Operand),
    Xor(SingleRegType, Operand),
    Cpl(SingleRegType),
    TestBit(GenericRegType, u8),
    SetBit(GenericRegType, u8),
    ResetBit(GenericRegType, u8),
    Swap(GenericRegType),
    Da(SingleRegType),
}

impl Display for ArithmeticType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inc(reg) => write!(f, "{}++", reg),
            Self::IncAddr(reg) => write!(f, "({})++", reg),
            Self::Dec(reg) => write!(f, "{}--", reg),
            Self::DecAddr(reg) => write!(f, "({})--", reg),
            Self::Add(dest, src) => write!(f, "{} += {}", dest, src),
            Self::Add16(dest, src) => write!(f, "{} += {}", dest, src),
            Self::AddSP(v) => write!(f, "SP += {}", v),
            Self::Adc(dest, src) => write!(f, "{} += {} + CY", dest, src),
            Self::Sub(dest, src) => write!(f, "{} -= {}", dest, src),
            Self::Sbc(dest, src) => write!(f, "{} -= {} - CY", dest, src),
            Self::Cmp(dest, src) => write!(f, "{} == {}", dest, src),
            Self::Rl(reg, clear_z) => {
                if *clear_z {
                    write!(f, "Rl({}), Z=0", reg)
                } else {
                    write!(f, "Rl({})", reg)
                }
            }
            Self::Rlc(reg, clear_z) => {
                if *clear_z {
                    write!(f, "RlC({}), Z=0", reg)
                } else {
                    write!(f, "RlC({})", reg)
                }
            }
            Self::Rr(reg, clear_z) => {
                if *clear_z {
                    write!(f, "Rr({}), Z=0", reg)
                } else {
                    write!(f, "Rr({})", reg)
                }
            }
            Self::Rrc(reg, clear_z) => {
                if *clear_z {
                    write!(f, "RrC({}), Z=0", reg)
                } else {
                    write!(f, "RrC({})", reg)
                }
            }
            Self::Sla(reg) => write!(f, "Sla({})", reg),
            Self::Sra(reg) => write!(f, "Sra({})", reg),
            Self::Srl(reg) => write!(f, "Srl({})", reg),
            Self::And(dest, op) => write!(f, "{} &= {}", dest, op),
            Self::Or(dest, op) => write!(f, "{} |= {}", dest, op),
            Self::Xor(dest, src) => write!(f, "{} = {} ^ {}", dest, src, dest),
            Self::Cpl(src) => write!(f, "!{}", src),
            Self::TestBit(reg, bit) => write!(f, "TestBit({}, {})", reg, bit),
            Self::ResetBit(reg, bit) => write!(f, "ResetBit({}, {})", reg, bit),
            Self::SetBit(reg, bit) => write!(f, "SetBit({}, {})", reg, bit),
            Self::Swap(reg) => write!(f, "Swap({})", reg),
            Self::Da(reg) => write!(f, "Da({})", reg),
        }
    }
}

#[derive(Clone)]
pub enum Operand {
    Imm(u8),
    Reg(SingleRegType),
    RegAddr(DoubleRegType),
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Imm(val) => write!(f, "{:#04X}", val),
            Self::Reg(reg) => write!(f, "{}", reg),
            Self::RegAddr(reg) => write!(f, "addr({})", reg),
        }
    }
}

#[derive(Clone)]
pub enum Source {
    RegImm(SingleRegType),
    RegAddr(DoubleRegType),
    Imm8(u8),
    Imm16(u16),
    Addr(u16),
    IoPortImm(u8),
    IoPortReg(SingleRegType),
    SpWithOffset(i8),
}

impl Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RegImm(reg) => write!(f, "{}", reg),
            Self::RegAddr(reg) => write!(f, "Addr({})", reg),
            Self::Imm8(val) => write!(f, "{:#04X}", val),
            Self::Imm16(val) => write!(f, "{:#06X}", val),
            Self::Addr(addr) => write!(f, "Addr({:#06X})", addr),
            Self::IoPortImm(val) => write!(f, "IO({:#04X})", val),
            Self::IoPortReg(reg) => write!(f, "IO({})", reg),
            Self::SpWithOffset(offs) => write!(f, "SP + ({})", offs),
        }
    }
}

#[derive(Clone)]
pub enum Dest {
    Addr(u16),
    RegAddr(DoubleRegType),
    IoPort(u8),
    IoPortReg(SingleRegType),
}

impl Display for Dest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Addr(addr) => write!(f, "{:#06X}", addr),
            Self::RegAddr(reg) => write!(f, "Addr({})", reg),
            Self::IoPort(val) => write!(f, "IO({:#04X})", val),
            Self::IoPortReg(reg) => write!(f, "IO({})", reg),
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum SingleRegType {
    A,
    B,
    C,
    D,
    E,
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
            Self::H => write!(f, "H"),
            Self::L => write!(f, "L"),
        }
    }
}

#[derive(Clone)]
pub enum DoubleRegType {
    AF,
    BC,
    DE,
    HL,
    SP,
}

impl Display for DoubleRegType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AF => write!(f, "AF"),
            Self::BC => write!(f, "BC"),
            Self::DE => write!(f, "DE"),
            Self::HL => write!(f, "HL"),
            Self::SP => write!(f, "SP"),
        }
    }
}

#[derive(Clone)]
pub enum GenericRegType {
    Single(SingleRegType),
    Double(DoubleRegType),
}

impl Display for GenericRegType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Single(reg) => write!(f, "{}", reg),
            Self::Double(reg) => write!(f, "{}", reg),
        }
    }
}

#[derive(Clone)]
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

#[derive(Clone)]
pub enum PostStore {
    Inc,
    Dec,
}

pub type PostLoad = PostStore;

impl Display for PostStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inc => write!(f, "HL+"),
            Self::Dec => write!(f, "HL-"),
        }
    }
}

#[derive(Clone)]
pub struct Instruction {
    instr: InstructionType,
    opcode: Opcode,
    cb_opcode: Option<CbOpcode>,
}

impl Instruction {
    pub fn decode(memory: &[u8]) -> Result<Self, GbError> {
        use ArithmeticType::*;
        use DoubleRegType::*;
        use GenericRegType::*;
        use InstructionType::*;
        use JumpCondition::*;
        use SingleRegType::*;
        use Source::*;

        type LE = LittleEndian;

        let opcode = Opcode::from_u8(memory[0]).ok_or(GbError::UnknownInstruction(memory[0]))?;
        let mut cb_opcode = None;
        let instr = match opcode {
            Opcode::Nop => Nop,
            Opcode::Stop => Stop,
            Opcode::Halt => Halt,
            Opcode::DaA => Arithmetic(Da(A)),
            Opcode::Scf => FlipCarry,
            Opcode::Ccf => ClearCarry,
            Opcode::Di => MasterInterrupt(false),
            Opcode::Ei => MasterInterrupt(true),
            Opcode::IncA => Arithmetic(Inc(Single(A))),
            Opcode::IncB => Arithmetic(Inc(Single(B))),
            Opcode::IncC => Arithmetic(Inc(Single(C))),
            Opcode::IncD => Arithmetic(Inc(Single(D))),
            Opcode::IncE => Arithmetic(Inc(Single(E))),
            Opcode::IncH => Arithmetic(Inc(Single(H))),
            Opcode::IncL => Arithmetic(Inc(Single(L))),
            Opcode::IncBC => Arithmetic(Inc(Double(BC))),
            Opcode::IncDE => Arithmetic(Inc(Double(DE))),
            Opcode::IncHL => Arithmetic(Inc(Double(HL))),
            Opcode::IncSP => Arithmetic(Inc(Double(SP))),
            Opcode::Inca16HL => Arithmetic(IncAddr(HL)),
            Opcode::DecA => Arithmetic(Dec(Single(A))),
            Opcode::DecB => Arithmetic(Dec(Single(B))),
            Opcode::DecC => Arithmetic(Dec(Single(C))),
            Opcode::DecD => Arithmetic(Dec(Single(D))),
            Opcode::DecE => Arithmetic(Dec(Single(E))),
            Opcode::DecH => Arithmetic(Dec(Single(H))),
            Opcode::DecL => Arithmetic(Dec(Single(L))),
            Opcode::DecBC => Arithmetic(Dec(Double(BC))),
            Opcode::DecDE => Arithmetic(Dec(Double(DE))),
            Opcode::DecHL => Arithmetic(Dec(Double(HL))),
            Opcode::DecSP => Arithmetic(Dec(Double(SP))),
            Opcode::Deca16HL => Arithmetic(DecAddr(HL)),
            Opcode::RlA => Arithmetic(Rl(Single(A), true)),
            Opcode::RlcA => Arithmetic(Rlc(Single(A), true)),
            Opcode::RrA => Arithmetic(Rr(Single(A), true)),
            Opcode::RrcA => Arithmetic(Rrc(Single(A), true)),
            Opcode::AddAA => Arithmetic(Add(A, Operand::Reg(A))),
            Opcode::AddAB => Arithmetic(Add(A, Operand::Reg(B))),
            Opcode::AddAC => Arithmetic(Add(A, Operand::Reg(C))),
            Opcode::AddAD => Arithmetic(Add(A, Operand::Reg(D))),
            Opcode::AddAE => Arithmetic(Add(A, Operand::Reg(E))),
            Opcode::AddAH => Arithmetic(Add(A, Operand::Reg(H))),
            Opcode::AddAL => Arithmetic(Add(A, Operand::Reg(L))),
            Opcode::AddAHL => Arithmetic(Add(A, Operand::RegAddr(HL))),
            Opcode::AddAd8 => Arithmetic(Add(A, Operand::Imm(memory[1]))),
            Opcode::AddHLBC => Arithmetic(Add16(HL, BC)),
            Opcode::AddHLDE => Arithmetic(Add16(HL, DE)),
            Opcode::AddHLHL => Arithmetic(Add16(HL, HL)),
            Opcode::AddHLSP => Arithmetic(Add16(HL, SP)),
            Opcode::AddSPs8 => Arithmetic(AddSP(memory[1] as i8)),
            Opcode::AdcAA => Arithmetic(Adc(A, Operand::Reg(A))),
            Opcode::AdcAB => Arithmetic(Adc(A, Operand::Reg(B))),
            Opcode::AdcAC => Arithmetic(Adc(A, Operand::Reg(C))),
            Opcode::AdcAD => Arithmetic(Adc(A, Operand::Reg(D))),
            Opcode::AdcAE => Arithmetic(Adc(A, Operand::Reg(E))),
            Opcode::AdcAH => Arithmetic(Adc(A, Operand::Reg(H))),
            Opcode::AdcAL => Arithmetic(Adc(A, Operand::Reg(L))),
            Opcode::AdcAHL => Arithmetic(Adc(A, Operand::RegAddr(HL))),
            Opcode::AdcAd8 => Arithmetic(Adc(A, Operand::Imm(memory[1]))),
            Opcode::SubAA => Arithmetic(Sub(A, Operand::Reg(A))),
            Opcode::SubAB => Arithmetic(Sub(A, Operand::Reg(B))),
            Opcode::SubAC => Arithmetic(Sub(A, Operand::Reg(C))),
            Opcode::SubAD => Arithmetic(Sub(A, Operand::Reg(D))),
            Opcode::SubAE => Arithmetic(Sub(A, Operand::Reg(E))),
            Opcode::SubAH => Arithmetic(Sub(A, Operand::Reg(H))),
            Opcode::SubAL => Arithmetic(Sub(A, Operand::Reg(L))),
            Opcode::SubAHL => Arithmetic(Sub(A, Operand::RegAddr(HL))),
            Opcode::SubAd8 => Arithmetic(Sub(A, Operand::Imm(memory[1]))),
            Opcode::SbcAA => Arithmetic(Sbc(A, Operand::Reg(A))),
            Opcode::SbcAB => Arithmetic(Sbc(A, Operand::Reg(B))),
            Opcode::SbcAC => Arithmetic(Sbc(A, Operand::Reg(C))),
            Opcode::SbcAD => Arithmetic(Sbc(A, Operand::Reg(D))),
            Opcode::SbcAE => Arithmetic(Sbc(A, Operand::Reg(E))),
            Opcode::SbcAH => Arithmetic(Sbc(A, Operand::Reg(H))),
            Opcode::SbcAL => Arithmetic(Sbc(A, Operand::Reg(L))),
            Opcode::SbcAHL => Arithmetic(Sbc(A, Operand::RegAddr(HL))),
            Opcode::SbcAd8 => Arithmetic(Sbc(A, Operand::Imm(memory[1]))),
            Opcode::AndA => Arithmetic(And(A, Operand::Reg(A))),
            Opcode::AndB => Arithmetic(And(A, Operand::Reg(B))),
            Opcode::AndC => Arithmetic(And(A, Operand::Reg(C))),
            Opcode::AndD => Arithmetic(And(A, Operand::Reg(D))),
            Opcode::AndE => Arithmetic(And(A, Operand::Reg(E))),
            Opcode::AndH => Arithmetic(And(A, Operand::Reg(H))),
            Opcode::AndL => Arithmetic(And(A, Operand::Reg(L))),
            Opcode::AndHL => Arithmetic(And(A, Operand::RegAddr(HL))),
            Opcode::Andd8 => Arithmetic(And(A, Operand::Imm(memory[1]))),
            Opcode::OrA => Arithmetic(Or(A, Operand::Reg(A))),
            Opcode::OrB => Arithmetic(Or(A, Operand::Reg(B))),
            Opcode::OrC => Arithmetic(Or(A, Operand::Reg(C))),
            Opcode::OrD => Arithmetic(Or(A, Operand::Reg(D))),
            Opcode::OrE => Arithmetic(Or(A, Operand::Reg(E))),
            Opcode::OrH => Arithmetic(Or(A, Operand::Reg(H))),
            Opcode::OrL => Arithmetic(Or(A, Operand::Reg(L))),
            Opcode::OrHL => Arithmetic(Or(A, Operand::RegAddr(HL))),
            Opcode::Ord8 => Arithmetic(Or(A, Operand::Imm(memory[1]))),
            Opcode::XorA => Arithmetic(Xor(A, Operand::Reg(A))),
            Opcode::XorB => Arithmetic(Xor(A, Operand::Reg(B))),
            Opcode::XorC => Arithmetic(Xor(A, Operand::Reg(C))),
            Opcode::XorD => Arithmetic(Xor(A, Operand::Reg(D))),
            Opcode::XorE => Arithmetic(Xor(A, Operand::Reg(E))),
            Opcode::XorH => Arithmetic(Xor(A, Operand::Reg(H))),
            Opcode::XorL => Arithmetic(Xor(A, Operand::Reg(L))),
            Opcode::XorHL => Arithmetic(Xor(A, Operand::RegAddr(HL))),
            Opcode::Xord8 => Arithmetic(Xor(A, Operand::Imm(memory[1]))),
            Opcode::Cpl => Arithmetic(Cpl(A)),
            Opcode::CpA => Arithmetic(Cmp(A, Operand::Reg(A))),
            Opcode::CpB => Arithmetic(Cmp(A, Operand::Reg(B))),
            Opcode::CpC => Arithmetic(Cmp(A, Operand::Reg(C))),
            Opcode::CpD => Arithmetic(Cmp(A, Operand::Reg(D))),
            Opcode::CpE => Arithmetic(Cmp(A, Operand::Reg(E))),
            Opcode::CpH => Arithmetic(Cmp(A, Operand::Reg(H))),
            Opcode::CpL => Arithmetic(Cmp(A, Operand::Reg(L))),
            Opcode::Cpd8 => Arithmetic(Cmp(A, Operand::Imm(memory[1]))),
            Opcode::CpHL => Arithmetic(Cmp(A, Operand::RegAddr(HL))),
            Opcode::Jr => Jump(Always, JumpType::Offset(memory[1] as i8)),
            Opcode::JrZ => Jump(Zero, JumpType::Offset(memory[1] as i8)),
            Opcode::JrNZ => Jump(NotZero, JumpType::Offset(memory[1] as i8)),
            Opcode::JrC => Jump(Carry, JumpType::Offset(memory[1] as i8)),
            Opcode::JrNC => Jump(NotCarry, JumpType::Offset(memory[1] as i8)),
            Opcode::Jp => Jump(Always, JumpType::Addr(LE::read_u16(&memory[1..3]))),
            Opcode::JpZ => Jump(Zero, JumpType::Addr(LE::read_u16(&memory[1..3]))),
            Opcode::JpNZ => Jump(NotZero, JumpType::Addr(LE::read_u16(&memory[1..3]))),
            Opcode::JpC => Jump(Carry, JumpType::Addr(LE::read_u16(&memory[1..3]))),
            Opcode::JpNC => Jump(NotCarry, JumpType::Addr(LE::read_u16(&memory[1..3]))),
            Opcode::JpHL => Jump(Always, JumpType::RegAddr(HL)),
            Opcode::LdAd8 => Load(Single(A), Imm8(memory[1])),
            Opcode::LdBd8 => Load(Single(B), Imm8(memory[1])),
            Opcode::LdCd8 => Load(Single(C), Imm8(memory[1])),
            Opcode::LdDd8 => Load(Single(D), Imm8(memory[1])),
            Opcode::LdEd8 => Load(Single(E), Imm8(memory[1])),
            Opcode::LdHd8 => Load(Single(H), Imm8(memory[1])),
            Opcode::LdLd8 => Load(Single(L), Imm8(memory[1])),
            Opcode::LdABC => Load(Single(A), RegAddr(BC)),
            Opcode::LdADE => Load(Single(A), RegAddr(DE)),
            Opcode::LdAHLinc => LoadWithOp(Single(A), RegAddr(HL), PostLoad::Inc),
            Opcode::LdAHLdec => LoadWithOp(Single(A), RegAddr(HL), PostLoad::Dec),
            Opcode::LdAHL => Load(Single(A), RegAddr(HL)),
            Opcode::LdBHL => Load(Single(B), RegAddr(HL)),
            Opcode::LdCHL => Load(Single(C), RegAddr(HL)),
            Opcode::LdDHL => Load(Single(D), RegAddr(HL)),
            Opcode::LdEHL => Load(Single(E), RegAddr(HL)),
            Opcode::LdHHL => Load(Single(H), RegAddr(HL)),
            Opcode::LdLHL => Load(Single(L), RegAddr(HL)),
            Opcode::LdBCd16 => Load(Double(BC), Imm16(LE::read_u16(&memory[1..3]))),
            Opcode::LdDEd16 => Load(Double(DE), Imm16(LE::read_u16(&memory[1..3]))),
            Opcode::LdHLd16 => Load(Double(HL), Imm16(LE::read_u16(&memory[1..3]))),
            Opcode::LdSPd16 => Load(Double(SP), Imm16(LE::read_u16(&memory[1..3]))),
            Opcode::LdAA => Load(Single(A), RegImm(A)),
            Opcode::LdAB => Load(Single(A), RegImm(B)),
            Opcode::LdAC => Load(Single(A), RegImm(C)),
            Opcode::LdAD => Load(Single(A), RegImm(D)),
            Opcode::LdAE => Load(Single(A), RegImm(E)),
            Opcode::LdAH => Load(Single(A), RegImm(H)),
            Opcode::LdAL => Load(Single(A), RegImm(L)),
            Opcode::LdBA => Load(Single(B), RegImm(A)),
            Opcode::LdBB => Load(Single(B), RegImm(B)),
            Opcode::LdBC => Load(Single(B), RegImm(C)),
            Opcode::LdBD => Load(Single(B), RegImm(D)),
            Opcode::LdBE => Load(Single(B), RegImm(E)),
            Opcode::LdBH => Load(Single(B), RegImm(H)),
            Opcode::LdBL => Load(Single(B), RegImm(L)),
            Opcode::LdCA => Load(Single(C), RegImm(A)),
            Opcode::LdCB => Load(Single(C), RegImm(B)),
            Opcode::LdCC => Load(Single(C), RegImm(C)),
            Opcode::LdCD => Load(Single(C), RegImm(D)),
            Opcode::LdCE => Load(Single(C), RegImm(E)),
            Opcode::LdCH => Load(Single(C), RegImm(H)),
            Opcode::LdCL => Load(Single(C), RegImm(L)),
            Opcode::LdDA => Load(Single(D), RegImm(A)),
            Opcode::LdDB => Load(Single(D), RegImm(B)),
            Opcode::LdDC => Load(Single(D), RegImm(C)),
            Opcode::LdDD => Load(Single(D), RegImm(D)),
            Opcode::LdDE => Load(Single(D), RegImm(E)),
            Opcode::LdDH => Load(Single(D), RegImm(H)),
            Opcode::LdDL => Load(Single(D), RegImm(L)),
            Opcode::LdEA => Load(Single(E), RegImm(A)),
            Opcode::LdEB => Load(Single(E), RegImm(B)),
            Opcode::LdEC => Load(Single(E), RegImm(C)),
            Opcode::LdED => Load(Single(E), RegImm(D)),
            Opcode::LdEE => Load(Single(E), RegImm(E)),
            Opcode::LdEH => Load(Single(E), RegImm(H)),
            Opcode::LdEL => Load(Single(E), RegImm(L)),
            Opcode::LdHA => Load(Single(H), RegImm(A)),
            Opcode::LdHB => Load(Single(H), RegImm(B)),
            Opcode::LdHC => Load(Single(H), RegImm(C)),
            Opcode::LdHD => Load(Single(H), RegImm(D)),
            Opcode::LdHE => Load(Single(H), RegImm(E)),
            Opcode::LdHH => Load(Single(H), RegImm(H)),
            Opcode::LdHL => Load(Single(H), RegImm(L)),
            Opcode::LdLA => Load(Single(L), RegImm(A)),
            Opcode::LdLB => Load(Single(L), RegImm(B)),
            Opcode::LdLC => Load(Single(L), RegImm(C)),
            Opcode::LdLD => Load(Single(L), RegImm(D)),
            Opcode::LdLE => Load(Single(L), RegImm(E)),
            Opcode::LdLH => Load(Single(L), RegImm(H)),
            Opcode::LdLL => Load(Single(L), RegImm(L)),
            Opcode::LdAioC => Load(Single(A), Source::IoPortReg(C)),
            Opcode::LdhAa8 => Load(Single(A), Source::IoPortImm(memory[1])),
            Opcode::LdAa16 => Load(Single(A), Source::Addr(LE::read_u16(&memory[1..3]))),
            Opcode::LdHLSPs8 => Load(Double(HL), Source::SpWithOffset(memory[1] as i8)),
            Opcode::LdSPHL => LoadSP(HL),
            Opcode::LdHLA => Store(Dest::RegAddr(HL), RegImm(A)),
            Opcode::LdHLB => Store(Dest::RegAddr(HL), RegImm(B)),
            Opcode::LdHLC => Store(Dest::RegAddr(HL), RegImm(C)),
            Opcode::LdHLD => Store(Dest::RegAddr(HL), RegImm(D)),
            Opcode::LdHLE => Store(Dest::RegAddr(HL), RegImm(E)),
            Opcode::LdHLH => Store(Dest::RegAddr(HL), RegImm(H)),
            Opcode::LdHLL => Store(Dest::RegAddr(HL), RegImm(L)),
            Opcode::LdHLd8 => Store(Dest::RegAddr(HL), Imm8(memory[1])),
            Opcode::LdBCA => Store(Dest::RegAddr(BC), RegImm(A)),
            Opcode::LdDEA => Store(Dest::RegAddr(DE), RegImm(A)),
            Opcode::Lda16A => Store(Dest::Addr(LE::read_u16(&memory[1..3])), RegImm(A)),
            Opcode::Ldha8A => Store(Dest::IoPort(memory[1]), Source::RegImm(A)),
            Opcode::LdioCA => Store(Dest::IoPortReg(C), Source::RegImm(A)),
            Opcode::LdHLincA => StoreWithOp(Dest::RegAddr(HL), RegImm(A), PostStore::Inc),
            Opcode::LdHLdecA => StoreWithOp(Dest::RegAddr(HL), RegImm(A), PostStore::Dec),
            Opcode::Lda16SP => StoreSP(LE::read_u16(&memory[1..3])),
            Opcode::PushAF => Push(AF),
            Opcode::PushBC => Push(BC),
            Opcode::PushDE => Push(DE),
            Opcode::PushHL => Push(HL),
            Opcode::PopAF => Pop(AF),
            Opcode::PopBC => Pop(BC),
            Opcode::PopDE => Pop(DE),
            Opcode::PopHL => Pop(HL),
            Opcode::Call => Call(LE::read_u16(&memory[1..3]), Always),
            Opcode::CallZ => Call(LE::read_u16(&memory[1..3]), Zero),
            Opcode::CallNZ => Call(LE::read_u16(&memory[1..3]), NotZero),
            Opcode::CallC => Call(LE::read_u16(&memory[1..3]), Carry),
            Opcode::CallNC => Call(LE::read_u16(&memory[1..3]), NotCarry),
            Opcode::Rst0 => Call(0x0000, Always),
            Opcode::Rst1 => Call(0x0008, Always),
            Opcode::Rst2 => Call(0x0010, Always),
            Opcode::Rst3 => Call(0x0018, Always),
            Opcode::Rst4 => Call(0x0020, Always),
            Opcode::Rst5 => Call(0x0028, Always),
            Opcode::Rst6 => Call(0x0030, Always),
            Opcode::Rst7 => Call(0x0038, Always),
            Opcode::Ret => Ret(Always),
            Opcode::RetZ => Ret(Zero),
            Opcode::RetNZ => Ret(NotZero),
            Opcode::RetC => Ret(Carry),
            Opcode::RetNC => Ret(NotCarry),
            Opcode::RetI => RetI,
            Opcode::Prefix => {
                cb_opcode = Some(
                    CbOpcode::from_u8(memory[1]).ok_or(GbError::UnknownCbInstruction(memory[1]))?,
                );
                Instruction::decode_cb(memory[1])?
            }
        };

        Ok(Self {
            instr,
            opcode,
            cb_opcode,
        })
    }

    fn decode_cb(opcode: u8) -> Result<InstructionType, GbError> {
        use ArithmeticType::*;
        use DoubleRegType::*;
        use GenericRegType::*;
        use InstructionType::*;
        use SingleRegType::*;

        let instr = match CbOpcode::from_u8(opcode).ok_or(GbError::UnknownCbInstruction(opcode))? {
            CbOpcode::RlA => Arithmetic(Rl(Single(A), false)),
            CbOpcode::RlB => Arithmetic(Rl(Single(B), false)),
            CbOpcode::RlC => Arithmetic(Rl(Single(C), false)),
            CbOpcode::RlD => Arithmetic(Rl(Single(D), false)),
            CbOpcode::RlE => Arithmetic(Rl(Single(E), false)),
            CbOpcode::RlH => Arithmetic(Rl(Single(H), false)),
            CbOpcode::RlL => Arithmetic(Rl(Single(L), false)),
            CbOpcode::RlHL => Arithmetic(Rl(Double(HL), false)),
            CbOpcode::RlcA => Arithmetic(Rlc(Single(A), false)),
            CbOpcode::RlcB => Arithmetic(Rlc(Single(B), false)),
            CbOpcode::RlcC => Arithmetic(Rlc(Single(C), false)),
            CbOpcode::RlcD => Arithmetic(Rlc(Single(D), false)),
            CbOpcode::RlcE => Arithmetic(Rlc(Single(E), false)),
            CbOpcode::RlcH => Arithmetic(Rlc(Single(H), false)),
            CbOpcode::RlcL => Arithmetic(Rlc(Single(L), false)),
            CbOpcode::RlcHL => Arithmetic(Rlc(Double(HL), false)),
            CbOpcode::RrA => Arithmetic(Rr(Single(A), false)),
            CbOpcode::RrB => Arithmetic(Rr(Single(B), false)),
            CbOpcode::RrC => Arithmetic(Rr(Single(C), false)),
            CbOpcode::RrD => Arithmetic(Rr(Single(D), false)),
            CbOpcode::RrE => Arithmetic(Rr(Single(E), false)),
            CbOpcode::RrH => Arithmetic(Rr(Single(H), false)),
            CbOpcode::RrL => Arithmetic(Rr(Single(L), false)),
            CbOpcode::RrHL => Arithmetic(Rr(Double(HL), false)),
            CbOpcode::RrcA => Arithmetic(Rrc(Single(A), false)),
            CbOpcode::RrcB => Arithmetic(Rrc(Single(B), false)),
            CbOpcode::RrcC => Arithmetic(Rrc(Single(C), false)),
            CbOpcode::RrcD => Arithmetic(Rrc(Single(D), false)),
            CbOpcode::RrcE => Arithmetic(Rrc(Single(E), false)),
            CbOpcode::RrcH => Arithmetic(Rrc(Single(H), false)),
            CbOpcode::RrcL => Arithmetic(Rrc(Single(L), false)),
            CbOpcode::RrcHL => Arithmetic(Rlc(Double(HL), false)),
            CbOpcode::SlaA => Arithmetic(Sla(Single(A))),
            CbOpcode::SlaB => Arithmetic(Sla(Single(B))),
            CbOpcode::SlaC => Arithmetic(Sla(Single(C))),
            CbOpcode::SlaD => Arithmetic(Sla(Single(D))),
            CbOpcode::SlaE => Arithmetic(Sla(Single(E))),
            CbOpcode::SlaH => Arithmetic(Sla(Single(H))),
            CbOpcode::SlaL => Arithmetic(Sla(Single(L))),
            CbOpcode::SlaHL => Arithmetic(Sla(Double(HL))),
            CbOpcode::SraA => Arithmetic(Sra(Single(A))),
            CbOpcode::SraB => Arithmetic(Sra(Single(B))),
            CbOpcode::SraC => Arithmetic(Sra(Single(C))),
            CbOpcode::SraD => Arithmetic(Sra(Single(D))),
            CbOpcode::SraE => Arithmetic(Sra(Single(E))),
            CbOpcode::SraH => Arithmetic(Sra(Single(H))),
            CbOpcode::SraL => Arithmetic(Sra(Single(L))),
            CbOpcode::SraHL => Arithmetic(Sra(Double(HL))),
            CbOpcode::SrlA => Arithmetic(Srl(Single(A))),
            CbOpcode::SrlB => Arithmetic(Srl(Single(B))),
            CbOpcode::SrlC => Arithmetic(Srl(Single(C))),
            CbOpcode::SrlD => Arithmetic(Srl(Single(D))),
            CbOpcode::SrlE => Arithmetic(Srl(Single(E))),
            CbOpcode::SrlH => Arithmetic(Srl(Single(H))),
            CbOpcode::SrlL => Arithmetic(Srl(Single(L))),
            CbOpcode::SrlHL => Arithmetic(Srl(Double(HL))),
            CbOpcode::SwapA => Arithmetic(Swap(Single(A))),
            CbOpcode::SwapB => Arithmetic(Swap(Single(B))),
            CbOpcode::SwapC => Arithmetic(Swap(Single(C))),
            CbOpcode::SwapD => Arithmetic(Swap(Single(D))),
            CbOpcode::SwapE => Arithmetic(Swap(Single(E))),
            CbOpcode::SwapH => Arithmetic(Swap(Single(H))),
            CbOpcode::SwapL => Arithmetic(Swap(Single(L))),
            CbOpcode::SwapHL => Arithmetic(Swap(Double(HL))),
            CbOpcode::Bit0A => Arithmetic(TestBit(Single(A), 0)),
            CbOpcode::Bit1A => Arithmetic(TestBit(Single(A), 1)),
            CbOpcode::Bit2A => Arithmetic(TestBit(Single(A), 2)),
            CbOpcode::Bit3A => Arithmetic(TestBit(Single(A), 3)),
            CbOpcode::Bit4A => Arithmetic(TestBit(Single(A), 4)),
            CbOpcode::Bit5A => Arithmetic(TestBit(Single(A), 5)),
            CbOpcode::Bit6A => Arithmetic(TestBit(Single(A), 6)),
            CbOpcode::Bit7A => Arithmetic(TestBit(Single(A), 7)),
            CbOpcode::Bit0B => Arithmetic(TestBit(Single(B), 0)),
            CbOpcode::Bit1B => Arithmetic(TestBit(Single(B), 1)),
            CbOpcode::Bit2B => Arithmetic(TestBit(Single(B), 2)),
            CbOpcode::Bit3B => Arithmetic(TestBit(Single(B), 3)),
            CbOpcode::Bit4B => Arithmetic(TestBit(Single(B), 4)),
            CbOpcode::Bit5B => Arithmetic(TestBit(Single(B), 5)),
            CbOpcode::Bit6B => Arithmetic(TestBit(Single(B), 6)),
            CbOpcode::Bit7B => Arithmetic(TestBit(Single(B), 7)),
            CbOpcode::Bit0C => Arithmetic(TestBit(Single(C), 0)),
            CbOpcode::Bit1C => Arithmetic(TestBit(Single(C), 1)),
            CbOpcode::Bit2C => Arithmetic(TestBit(Single(C), 2)),
            CbOpcode::Bit3C => Arithmetic(TestBit(Single(C), 3)),
            CbOpcode::Bit4C => Arithmetic(TestBit(Single(C), 4)),
            CbOpcode::Bit5C => Arithmetic(TestBit(Single(C), 5)),
            CbOpcode::Bit6C => Arithmetic(TestBit(Single(C), 6)),
            CbOpcode::Bit7C => Arithmetic(TestBit(Single(C), 7)),
            CbOpcode::Bit0D => Arithmetic(TestBit(Single(D), 0)),
            CbOpcode::Bit1D => Arithmetic(TestBit(Single(D), 1)),
            CbOpcode::Bit2D => Arithmetic(TestBit(Single(D), 2)),
            CbOpcode::Bit3D => Arithmetic(TestBit(Single(D), 3)),
            CbOpcode::Bit4D => Arithmetic(TestBit(Single(D), 4)),
            CbOpcode::Bit5D => Arithmetic(TestBit(Single(D), 5)),
            CbOpcode::Bit6D => Arithmetic(TestBit(Single(D), 6)),
            CbOpcode::Bit7D => Arithmetic(TestBit(Single(D), 7)),
            CbOpcode::Bit0E => Arithmetic(TestBit(Single(E), 0)),
            CbOpcode::Bit1E => Arithmetic(TestBit(Single(E), 1)),
            CbOpcode::Bit2E => Arithmetic(TestBit(Single(E), 2)),
            CbOpcode::Bit3E => Arithmetic(TestBit(Single(E), 3)),
            CbOpcode::Bit4E => Arithmetic(TestBit(Single(E), 4)),
            CbOpcode::Bit5E => Arithmetic(TestBit(Single(E), 5)),
            CbOpcode::Bit6E => Arithmetic(TestBit(Single(E), 6)),
            CbOpcode::Bit7E => Arithmetic(TestBit(Single(E), 7)),
            CbOpcode::Bit0H => Arithmetic(TestBit(Single(H), 0)),
            CbOpcode::Bit1H => Arithmetic(TestBit(Single(H), 1)),
            CbOpcode::Bit2H => Arithmetic(TestBit(Single(H), 2)),
            CbOpcode::Bit3H => Arithmetic(TestBit(Single(H), 3)),
            CbOpcode::Bit4H => Arithmetic(TestBit(Single(H), 4)),
            CbOpcode::Bit5H => Arithmetic(TestBit(Single(H), 5)),
            CbOpcode::Bit6H => Arithmetic(TestBit(Single(H), 6)),
            CbOpcode::Bit7H => Arithmetic(TestBit(Single(H), 7)),
            CbOpcode::Bit0L => Arithmetic(TestBit(Single(L), 0)),
            CbOpcode::Bit1L => Arithmetic(TestBit(Single(L), 1)),
            CbOpcode::Bit2L => Arithmetic(TestBit(Single(L), 2)),
            CbOpcode::Bit3L => Arithmetic(TestBit(Single(L), 3)),
            CbOpcode::Bit4L => Arithmetic(TestBit(Single(L), 4)),
            CbOpcode::Bit5L => Arithmetic(TestBit(Single(L), 5)),
            CbOpcode::Bit6L => Arithmetic(TestBit(Single(L), 6)),
            CbOpcode::Bit7L => Arithmetic(TestBit(Single(L), 7)),
            CbOpcode::Bit0HL => Arithmetic(TestBit(Double(HL), 0)),
            CbOpcode::Bit1HL => Arithmetic(TestBit(Double(HL), 1)),
            CbOpcode::Bit2HL => Arithmetic(TestBit(Double(HL), 2)),
            CbOpcode::Bit3HL => Arithmetic(TestBit(Double(HL), 3)),
            CbOpcode::Bit4HL => Arithmetic(TestBit(Double(HL), 4)),
            CbOpcode::Bit5HL => Arithmetic(TestBit(Double(HL), 5)),
            CbOpcode::Bit6HL => Arithmetic(TestBit(Double(HL), 6)),
            CbOpcode::Bit7HL => Arithmetic(TestBit(Double(HL), 7)),

            CbOpcode::Res0A => Arithmetic(ResetBit(Single(A), 0)),
            CbOpcode::Res1A => Arithmetic(ResetBit(Single(A), 1)),
            CbOpcode::Res2A => Arithmetic(ResetBit(Single(A), 2)),
            CbOpcode::Res3A => Arithmetic(ResetBit(Single(A), 3)),
            CbOpcode::Res4A => Arithmetic(ResetBit(Single(A), 4)),
            CbOpcode::Res5A => Arithmetic(ResetBit(Single(A), 5)),
            CbOpcode::Res6A => Arithmetic(ResetBit(Single(A), 6)),
            CbOpcode::Res7A => Arithmetic(ResetBit(Single(A), 7)),
            CbOpcode::Res0B => Arithmetic(ResetBit(Single(B), 0)),
            CbOpcode::Res1B => Arithmetic(ResetBit(Single(B), 1)),
            CbOpcode::Res2B => Arithmetic(ResetBit(Single(B), 2)),
            CbOpcode::Res3B => Arithmetic(ResetBit(Single(B), 3)),
            CbOpcode::Res4B => Arithmetic(ResetBit(Single(B), 4)),
            CbOpcode::Res5B => Arithmetic(ResetBit(Single(B), 5)),
            CbOpcode::Res6B => Arithmetic(ResetBit(Single(B), 6)),
            CbOpcode::Res7B => Arithmetic(ResetBit(Single(B), 7)),
            CbOpcode::Res0C => Arithmetic(ResetBit(Single(C), 0)),
            CbOpcode::Res1C => Arithmetic(ResetBit(Single(C), 1)),
            CbOpcode::Res2C => Arithmetic(ResetBit(Single(C), 2)),
            CbOpcode::Res3C => Arithmetic(ResetBit(Single(C), 3)),
            CbOpcode::Res4C => Arithmetic(ResetBit(Single(C), 4)),
            CbOpcode::Res5C => Arithmetic(ResetBit(Single(C), 5)),
            CbOpcode::Res6C => Arithmetic(ResetBit(Single(C), 6)),
            CbOpcode::Res7C => Arithmetic(ResetBit(Single(C), 7)),
            CbOpcode::Res0D => Arithmetic(ResetBit(Single(D), 0)),
            CbOpcode::Res1D => Arithmetic(ResetBit(Single(D), 1)),
            CbOpcode::Res2D => Arithmetic(ResetBit(Single(D), 2)),
            CbOpcode::Res3D => Arithmetic(ResetBit(Single(D), 3)),
            CbOpcode::Res4D => Arithmetic(ResetBit(Single(D), 4)),
            CbOpcode::Res5D => Arithmetic(ResetBit(Single(D), 5)),
            CbOpcode::Res6D => Arithmetic(ResetBit(Single(D), 6)),
            CbOpcode::Res7D => Arithmetic(ResetBit(Single(D), 7)),
            CbOpcode::Res0E => Arithmetic(ResetBit(Single(E), 0)),
            CbOpcode::Res1E => Arithmetic(ResetBit(Single(E), 1)),
            CbOpcode::Res2E => Arithmetic(ResetBit(Single(E), 2)),
            CbOpcode::Res3E => Arithmetic(ResetBit(Single(E), 3)),
            CbOpcode::Res4E => Arithmetic(ResetBit(Single(E), 4)),
            CbOpcode::Res5E => Arithmetic(ResetBit(Single(E), 5)),
            CbOpcode::Res6E => Arithmetic(ResetBit(Single(E), 6)),
            CbOpcode::Res7E => Arithmetic(ResetBit(Single(E), 7)),
            CbOpcode::Res0H => Arithmetic(ResetBit(Single(H), 0)),
            CbOpcode::Res1H => Arithmetic(ResetBit(Single(H), 1)),
            CbOpcode::Res2H => Arithmetic(ResetBit(Single(H), 2)),
            CbOpcode::Res3H => Arithmetic(ResetBit(Single(H), 3)),
            CbOpcode::Res4H => Arithmetic(ResetBit(Single(H), 4)),
            CbOpcode::Res5H => Arithmetic(ResetBit(Single(H), 5)),
            CbOpcode::Res6H => Arithmetic(ResetBit(Single(H), 6)),
            CbOpcode::Res7H => Arithmetic(ResetBit(Single(H), 7)),
            CbOpcode::Res0L => Arithmetic(ResetBit(Single(L), 0)),
            CbOpcode::Res1L => Arithmetic(ResetBit(Single(L), 1)),
            CbOpcode::Res2L => Arithmetic(ResetBit(Single(L), 2)),
            CbOpcode::Res3L => Arithmetic(ResetBit(Single(L), 3)),
            CbOpcode::Res4L => Arithmetic(ResetBit(Single(L), 4)),
            CbOpcode::Res5L => Arithmetic(ResetBit(Single(L), 5)),
            CbOpcode::Res6L => Arithmetic(ResetBit(Single(L), 6)),
            CbOpcode::Res7L => Arithmetic(ResetBit(Single(L), 7)),
            CbOpcode::Res0HL => Arithmetic(ResetBit(Double(HL), 0)),
            CbOpcode::Res1HL => Arithmetic(ResetBit(Double(HL), 1)),
            CbOpcode::Res2HL => Arithmetic(ResetBit(Double(HL), 2)),
            CbOpcode::Res3HL => Arithmetic(ResetBit(Double(HL), 3)),
            CbOpcode::Res4HL => Arithmetic(ResetBit(Double(HL), 4)),
            CbOpcode::Res5HL => Arithmetic(ResetBit(Double(HL), 5)),
            CbOpcode::Res6HL => Arithmetic(ResetBit(Double(HL), 6)),
            CbOpcode::Res7HL => Arithmetic(ResetBit(Double(HL), 7)),

            CbOpcode::Set0A => Arithmetic(SetBit(Single(A), 0)),
            CbOpcode::Set1A => Arithmetic(SetBit(Single(A), 1)),
            CbOpcode::Set2A => Arithmetic(SetBit(Single(A), 2)),
            CbOpcode::Set3A => Arithmetic(SetBit(Single(A), 3)),
            CbOpcode::Set4A => Arithmetic(SetBit(Single(A), 4)),
            CbOpcode::Set5A => Arithmetic(SetBit(Single(A), 5)),
            CbOpcode::Set6A => Arithmetic(SetBit(Single(A), 6)),
            CbOpcode::Set7A => Arithmetic(SetBit(Single(A), 7)),
            CbOpcode::Set0B => Arithmetic(SetBit(Single(B), 0)),
            CbOpcode::Set1B => Arithmetic(SetBit(Single(B), 1)),
            CbOpcode::Set2B => Arithmetic(SetBit(Single(B), 2)),
            CbOpcode::Set3B => Arithmetic(SetBit(Single(B), 3)),
            CbOpcode::Set4B => Arithmetic(SetBit(Single(B), 4)),
            CbOpcode::Set5B => Arithmetic(SetBit(Single(B), 5)),
            CbOpcode::Set6B => Arithmetic(SetBit(Single(B), 6)),
            CbOpcode::Set7B => Arithmetic(SetBit(Single(B), 7)),
            CbOpcode::Set0C => Arithmetic(SetBit(Single(C), 0)),
            CbOpcode::Set1C => Arithmetic(SetBit(Single(C), 1)),
            CbOpcode::Set2C => Arithmetic(SetBit(Single(C), 2)),
            CbOpcode::Set3C => Arithmetic(SetBit(Single(C), 3)),
            CbOpcode::Set4C => Arithmetic(SetBit(Single(C), 4)),
            CbOpcode::Set5C => Arithmetic(SetBit(Single(C), 5)),
            CbOpcode::Set6C => Arithmetic(SetBit(Single(C), 6)),
            CbOpcode::Set7C => Arithmetic(SetBit(Single(C), 7)),
            CbOpcode::Set0D => Arithmetic(SetBit(Single(D), 0)),
            CbOpcode::Set1D => Arithmetic(SetBit(Single(D), 1)),
            CbOpcode::Set2D => Arithmetic(SetBit(Single(D), 2)),
            CbOpcode::Set3D => Arithmetic(SetBit(Single(D), 3)),
            CbOpcode::Set4D => Arithmetic(SetBit(Single(D), 4)),
            CbOpcode::Set5D => Arithmetic(SetBit(Single(D), 5)),
            CbOpcode::Set6D => Arithmetic(SetBit(Single(D), 6)),
            CbOpcode::Set7D => Arithmetic(SetBit(Single(D), 7)),
            CbOpcode::Set0E => Arithmetic(SetBit(Single(E), 0)),
            CbOpcode::Set1E => Arithmetic(SetBit(Single(E), 1)),
            CbOpcode::Set2E => Arithmetic(SetBit(Single(E), 2)),
            CbOpcode::Set3E => Arithmetic(SetBit(Single(E), 3)),
            CbOpcode::Set4E => Arithmetic(SetBit(Single(E), 4)),
            CbOpcode::Set5E => Arithmetic(SetBit(Single(E), 5)),
            CbOpcode::Set6E => Arithmetic(SetBit(Single(E), 6)),
            CbOpcode::Set7E => Arithmetic(SetBit(Single(E), 7)),
            CbOpcode::Set0H => Arithmetic(SetBit(Single(H), 0)),
            CbOpcode::Set1H => Arithmetic(SetBit(Single(H), 1)),
            CbOpcode::Set2H => Arithmetic(SetBit(Single(H), 2)),
            CbOpcode::Set3H => Arithmetic(SetBit(Single(H), 3)),
            CbOpcode::Set4H => Arithmetic(SetBit(Single(H), 4)),
            CbOpcode::Set5H => Arithmetic(SetBit(Single(H), 5)),
            CbOpcode::Set6H => Arithmetic(SetBit(Single(H), 6)),
            CbOpcode::Set7H => Arithmetic(SetBit(Single(H), 7)),
            CbOpcode::Set0L => Arithmetic(SetBit(Single(L), 0)),
            CbOpcode::Set1L => Arithmetic(SetBit(Single(L), 1)),
            CbOpcode::Set2L => Arithmetic(SetBit(Single(L), 2)),
            CbOpcode::Set3L => Arithmetic(SetBit(Single(L), 3)),
            CbOpcode::Set4L => Arithmetic(SetBit(Single(L), 4)),
            CbOpcode::Set5L => Arithmetic(SetBit(Single(L), 5)),
            CbOpcode::Set6L => Arithmetic(SetBit(Single(L), 6)),
            CbOpcode::Set7L => Arithmetic(SetBit(Single(L), 7)),
            CbOpcode::Set0HL => Arithmetic(SetBit(Double(HL), 0)),
            CbOpcode::Set1HL => Arithmetic(SetBit(Double(HL), 1)),
            CbOpcode::Set2HL => Arithmetic(SetBit(Double(HL), 2)),
            CbOpcode::Set3HL => Arithmetic(SetBit(Double(HL), 3)),
            CbOpcode::Set4HL => Arithmetic(SetBit(Double(HL), 4)),
            CbOpcode::Set5HL => Arithmetic(SetBit(Double(HL), 5)),
            CbOpcode::Set6HL => Arithmetic(SetBit(Double(HL), 6)),
            CbOpcode::Set7HL => Arithmetic(SetBit(Double(HL), 7)),
        };

        Ok(instr)
    }

    pub fn peek_len(opcode: u8) -> Result<u8, GbError> {
        Ok(Opcode::from_u8(opcode)
            .ok_or(GbError::UnknownInstruction(opcode))?
            .length())
    }

    pub fn len(&self) -> u8 {
        self.opcode.length()
    }

    pub fn instr_type(&self) -> &InstructionType {
        &self.instr
    }

    pub fn cycles(&self, jumped: bool) -> u8 {
        match self.cb_opcode.as_ref() {
            Some(op) => op.cycles(),
            None => self.opcode.cycles(jumped),
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use InstructionType::*;

        write!(f, "({:#04X}) ", self.opcode as u8)?;
        match &self.instr {
            Nop => write!(f, "Nop"),
            Stop => write!(f, "Stop"),
            Halt => write!(f, "Halt"),
            FlipCarry => write!(f, "Flip CY"),
            ClearCarry => write!(f, "Clear CY"),
            MasterInterrupt(enable) => write!(f, "IME {}", enable),
            Arithmetic(ar_type) => write!(f, "{}", ar_type),
            Jump(cond, jump_type) => write!(f, "J {}, {}", cond, jump_type),
            Load(reg, source) => write!(f, "Load {} {}", reg, source),
            LoadWithOp(reg, source, post_load) => {
                write!(f, "Load {} {} {}", reg, source, post_load)
            }
            Store(dest, reg) => write!(f, "Store {} {}", dest, reg),
            LoadSP(src) => write!(f, "SP = {}", src),
            StoreWithOp(dest, reg, op) => {
                write!(f, "Store {} {} {}", dest, reg, op)
            }
            StoreSP(addr) => write!(f, "Store SP {}", addr),
            Push(reg) => write!(f, "Push {}", reg),
            Pop(reg) => write!(f, "Pop {}", reg),
            Call(addr, cond) => write!(f, "Call {}, {}", cond, addr),
            Ret(cond) => write!(f, "Ret {}", cond),
            RetI => write!(f, "RetI"),
        }
    }
}
