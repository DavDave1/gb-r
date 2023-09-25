pub mod opcode;

use std::fmt::Display;

use byteorder::{ByteOrder, LittleEndian};
use enum_primitive::FromPrimitive;

use super::GbError;

use self::opcode::{CbOpcode, Opcode};

pub enum InstructionType {
    Nop,
    Stop,
    MasterInterrupt(bool), // enable/disable
    Arithmetic(ArithmeticType),
    Jump(JumpCondition, JumpType),
    Load(GenericRegType, SourceType),
    Store(DestType, SourceType, PostStore),
    Push(DoubleRegType),
    Pop(DoubleRegType),
    Call(CallMode),
    Ret,
}

pub enum JumpType {
    Relative(i8),
    Absolute(u16),
}

impl Display for JumpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Relative(offset) => write!(f, "Rel({:#04X})", offset),
            Self::Absolute(addr) => write!(f, "Abs({:#06X})", addr),
        }
    }
}

pub enum ArithmeticType {
    Inc(GenericRegType),
    Dec(GenericRegType),
    Add(SingleRegType, OperandType),   // target, source
    Sub(SingleRegType, SingleRegType), // target, source
    Cmp(SingleRegType, OperandType),
    Rl(SingleRegType, bool),  // target, clear_z_flag
    RlC(SingleRegType, bool), // target, clear_z_flag
    Sla(SingleRegType),
    And(SingleRegType, OperandType),
    Or(SingleRegType, OperandType),
    Xor(SingleRegType, SingleRegType), // target, source
    TestBit(SingleRegType, u8),
    ResetBit(SingleRegType, u8),
}

impl Display for ArithmeticType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inc(reg) => write!(f, "{}++", reg),
            Self::Dec(reg) => write!(f, "{}--", reg),
            Self::Add(dest, src) => write!(f, "{} += {}", dest, src),
            Self::Sub(dest, src) => write!(f, "{} -= {}", dest, src),
            Self::Cmp(dest, src) => write!(f, "{} == {}", dest, src),
            Self::Rl(reg, clear_z) => {
                if *clear_z {
                    write!(f, "Rl({}), Z=0", reg)
                } else {
                    write!(f, "Rl({})", reg)
                }
            }
            Self::RlC(reg, clear_z) => {
                if *clear_z {
                    write!(f, "RlC({}), Z=0", reg)
                } else {
                    write!(f, "RlC({})", reg)
                }
            }
            Self::Sla(reg) => write!(f, "Sla({})", reg),
            Self::And(dest, op) => write!(f, "{} &= {}", dest, op),
            Self::Or(dest, op) => write!(f, "{} |= {}", dest, op),
            Self::Xor(dest, src) => write!(f, "{} = {} ^ {}", dest, src, dest),
            Self::TestBit(reg, bit) => write!(f, "TestBit({}, {})", reg, bit),
            Self::ResetBit(reg, bit) => write!(f, "ResetBit({}, {})", reg, bit),
        }
    }
}

pub enum OperandType {
    Imm(u8),
    Reg(SingleRegType),
    RegAddr(DoubleRegType),
}

impl Display for OperandType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Imm(val) => write!(f, "{:#04X}", val),
            Self::Reg(reg) => write!(f, "{}", reg),
            Self::RegAddr(reg) => write!(f, "addr({})", reg),
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

#[derive(PartialEq)]
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
        use JumpType::*;
        use SingleRegType::*;
        use SourceType::*;

        let opcode = Opcode::from_u8(memory[0]).ok_or(GbError::UnknownInstruction(memory[0]))?;
        let mut cb_opcode = None;
        let instr = match opcode {
            Opcode::Nop => Nop,
            Opcode::Stop => Stop,
            Opcode::DI => MasterInterrupt(false),
            Opcode::IncB => Arithmetic(Inc(Single(B))),
            Opcode::IncC => Arithmetic(Inc(Single(C))),
            Opcode::IncH => Arithmetic(Inc(Single(H))),
            Opcode::IncDE => Arithmetic(Inc(Double(DE))),
            Opcode::IncHL => Arithmetic(Inc(Double(HL))),
            Opcode::DecA => Arithmetic(Dec(Single(A))),
            Opcode::DecB => Arithmetic(Dec(Single(B))),
            Opcode::DecC => Arithmetic(Dec(Single(C))),
            Opcode::DecD => Arithmetic(Dec(Single(D))),
            Opcode::DecE => Arithmetic(Dec(Single(E))),
            Opcode::DecBC => Arithmetic(Dec(Double(BC))),
            Opcode::AddAA => Arithmetic(Add(A, OperandType::Reg(A))),
            Opcode::AddAB => Arithmetic(Add(A, OperandType::Reg(B))),
            Opcode::AddAHL => Arithmetic(Add(A, OperandType::RegAddr(HL))),
            Opcode::SubAB => Arithmetic(Sub(A, B)),
            Opcode::SubAL => Arithmetic(Sub(A, L)),
            Opcode::RlA => Arithmetic(Rl(A, true)),
            Opcode::Andd8 => Arithmetic(And(A, OperandType::Imm(memory[1]))),
            Opcode::OrC => Arithmetic(Or(A, OperandType::Reg(C))),
            Opcode::XorA => Arithmetic(Xor(A, A)),
            Opcode::Cpd8 => Arithmetic(Cmp(A, OperandType::Imm(memory[1]))),
            Opcode::CpHL => Arithmetic(Cmp(A, OperandType::RegAddr(HL))),
            Opcode::Jr => Jump(Always, Relative(memory[1] as i8)),
            Opcode::Jrz => Jump(Zero, Relative(memory[1] as i8)),
            Opcode::Jrnz => Jump(NotZero, Relative(memory[1] as i8)),
            Opcode::Jp => Jump(Always, Absolute(LittleEndian::read_u16(&memory[1..3]))),
            Opcode::LdAd8 => Load(Single(A), Imm8(memory[1])),
            Opcode::LdBd8 => Load(Single(B), Imm8(memory[1])),
            Opcode::LdCd8 => Load(Single(C), Imm8(memory[1])),
            Opcode::LdDd8 => Load(Single(D), Imm8(memory[1])),
            Opcode::LdEd8 => Load(Single(E), Imm8(memory[1])),
            Opcode::LdLd8 => Load(Single(L), Imm8(memory[1])),
            Opcode::LdADE => Load(Single(A), RegAddr(DE)),
            Opcode::LdSPd16 => Load(Double(SP), Imm16(LittleEndian::read_u16(&memory[1..3]))),
            Opcode::LdBCd16 => Load(Double(BC), Imm16(LittleEndian::read_u16(&memory[1..3]))),
            Opcode::LdDEd16 => Load(Double(DE), Imm16(LittleEndian::read_u16(&memory[1..3]))),
            Opcode::LdHLd16 => Load(Double(HL), Imm16(LittleEndian::read_u16(&memory[1..3]))),
            Opcode::LdHLincA => Store(DestType::RegAddr(HL), SourceType::RegImm(A), PostStore::Inc),
            Opcode::LdHLdecA => Store(DestType::RegAddr(HL), SourceType::RegImm(A), PostStore::Dec),
            Opcode::LdHLA => Store(
                DestType::RegAddr(HL),
                SourceType::RegImm(A),
                PostStore::None,
            ),
            Opcode::LdHLd8 => Store(
                DestType::RegAddr(HL),
                SourceType::Imm8(memory[1]),
                PostStore::None,
            ),
            Opcode::LdBA => Load(Single(B), RegImm(A)),
            Opcode::LdCA => Load(Single(C), RegImm(A)),
            Opcode::LdDA => Load(Single(D), RegImm(A)),
            Opcode::LdHA => Load(Single(H), RegImm(A)),
            Opcode::LdAB => Load(Single(A), RegImm(B)),
            Opcode::LdAD => Load(Single(A), RegImm(D)),
            Opcode::LdAE => Load(Single(A), RegImm(E)),
            Opcode::LdAH => Load(Single(A), RegImm(H)),
            Opcode::LdAL => Load(Single(A), RegImm(L)),
            Opcode::Lda16A => Store(
                DestType::Addr(LittleEndian::read_u16(&memory[1..3])),
                SourceType::RegImm(A),
                PostStore::None,
            ),
            Opcode::Ldha8A => Store(
                DestType::IoPort(memory[1]),
                SourceType::RegImm(A),
                PostStore::None,
            ),
            Opcode::LdhAa8 => Load(Single(A), SourceType::IoPortImm(memory[1])),
            Opcode::LdhCA => Store(
                DestType::IoPortReg(C),
                SourceType::RegImm(A),
                PostStore::None,
            ),
            Opcode::PushBC => Push(BC),
            Opcode::PushDE => Push(DE),
            Opcode::PopBC => Pop(BC),
            Opcode::PopDE => Pop(DE),
            Opcode::Calla16 => Call(CallMode::Absolute(LittleEndian::read_u16(&memory[1..3]))),
            Opcode::Ret => Ret,
            Opcode::Prefix => {
                cb_opcode = Some(
                    CbOpcode::from_u8(memory[1]).ok_or(GbError::UnknownCbInstruction(memory[1]))?,
                );
                match cb_opcode.as_ref().unwrap() {
                    CbOpcode::RlC => Arithmetic(RlC(C, false)),
                    CbOpcode::SlaB => Arithmetic(Sla(B)),
                    CbOpcode::Bit7H => Arithmetic(TestBit(H, 7)),
                    CbOpcode::Res0A => Arithmetic(ResetBit(A, 0)),
                }
            }
        };

        Ok(Self {
            instr,
            opcode,
            cb_opcode,
        })
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
        match &self.instr {
            Nop => write!(f, "Nop"),
            Stop => write!(f, "Stop"),
            MasterInterrupt(enable) => write!(f, "IME {}", enable),
            Arithmetic(ar_type) => write!(f, "{}", ar_type),
            Jump(cond, jump_type) => write!(f, "J {}, {}", cond, jump_type),
            Load(reg, source) => write!(f, "Load {} {}", reg, source),
            Store(dest, reg, post_store) => write!(f, "Store {} {} {}", dest, reg, post_store),
            Push(reg) => write!(f, "Push {}", reg),
            Pop(reg) => write!(f, "Pop {}", reg),
            Call(mode) => write!(f, "Call {}", mode),
            Ret => write!(f, "Return"),
        }
    }
}
