enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum Opcode {
    Nop = 0x00,
    LdBCd16 = 0x01,
    IncB = 0x04,
    DecB = 0x05,
    IncC = 0x0C,
    DecC = 0x0D,
    LdBd8 = 0x06,
    DecBC = 0x0B,
    LdCd8 = 0x0E,
    Stop = 0x10,
    LdDEd16 = 0x11,
    IncDE = 0x13,
    DecD = 0x15,
    LdDd8 = 0x16,
    RlA = 0x17,
    Jr = 0x18,
    LdADE = 0x1A,
    DecE = 0x1D,
    LdEd8 = 0x1E,
    Jrnz = 0x20,
    LdHLd16 = 0x21,
    LdHLincA = 0x22,
    IncHL = 0x23,
    IncH = 0x24,
    Jrz = 0x28,
    LdAHLinc = 0x2A,
    LdLd8 = 0x2E,
    LdSPd16 = 0x31,
    LdHLdecA = 0x32,
    LdHLd8 = 0x36,
    DecA = 0x3D,
    LdAd8 = 0x3E,
    LdBA = 0x47,
    LdCA = 0x4F,
    LdDA = 0x57,
    LdHB = 0x60,
    LdHA = 0x67,
    LdHLA = 0x77,
    LdAB = 0x78,
    LdAD = 0x7A,
    LdAE = 0x7B,
    LdAH = 0x7C,
    LdAL = 0x7D,
    AddAB = 0x80,
    AddAHL = 0x86,
    AddAA = 0x87,
    SubAB = 0x90,
    SubAL = 0x95,
    AndB = 0xA0,
    XorA = 0xAF,
    OrC = 0xB1,
    CpHL = 0xBE,
    PopBC = 0xC1,
    Jp = 0xC3,
    PushBC = 0xC5,
    Ret = 0xC9,
    Prefix = 0xCB,
    Calla16 = 0xCD,
    PopDE = 0xD1,
    PushDE = 0xD5,
    Ldha8A = 0xE0,
    LdhCA = 0xE2,
    Andd8 = 0xE6,
    Lda16A = 0xEA,
    LdhAa8 = 0xF0,
    DI = 0xF3,
    Cpd8 = 0xFE,
}
}

impl Opcode {
    pub fn length(&self) -> u8 {
        match self {
            Self::Nop => 1,
            Self::DecB => 1,
            Self::IncB => 1,
            Self::IncH => 1,
            Self::IncC => 1,
            Self::DecC => 1,
            Self::DecD => 1,
            Self::DecE => 1,
            Self::DecBC => 1,
            Self::LdBd8 => 2,
            Self::LdCd8 => 2,
            Self::LdDd8 => 2,
            Self::LdEd8 => 2,
            Self::Stop => 1,
            Self::IncDE => 1,
            Self::RlA => 1,
            Self::LdADE => 1,
            Self::Jr => 2,
            Self::Jrnz => 2,
            Self::Jrz => 2,
            Self::Jp => 3,
            Self::LdBCd16 => 3,
            Self::LdDEd16 => 3,
            Self::LdHLd16 => 3,
            Self::LdHLd8 => 2,
            Self::LdHLincA => 1,
            Self::IncHL => 1,
            Self::LdLd8 => 2,
            Self::LdSPd16 => 3,
            Self::LdHLdecA => 1,
            Self::DecA => 1,
            Self::LdAHLinc => 1,
            Self::LdAd8 => 2,
            Self::LdCA => 1,
            Self::LdDA => 1,
            Self::LdHA => 1,
            Self::LdHB => 1,
            Self::LdHLA => 1,
            Self::LdAB => 1,
            Self::LdAD => 1,
            Self::LdAE => 1,
            Self::LdAH => 1,
            Self::LdAL => 1,
            Self::LdBA => 1,
            Self::AddAA => 1,
            Self::AddAB => 1,
            Self::AddAHL => 1,
            Self::SubAB => 1,
            Self::SubAL => 1,
            Self::Andd8 => 2,
            Self::AndB => 1,
            Self::OrC => 1,
            Self::XorA => 1,
            Self::PopBC => 1,
            Self::PopDE => 1,
            Self::PushBC => 1,
            Self::PushDE => 1,
            Self::Ret => 1,
            Self::Prefix => 2,
            Self::Calla16 => 3,
            Self::Ldha8A => 2,
            Self::Lda16A => 3,
            Self::LdhCA => 1,
            Self::LdhAa8 => 2,
            Self::Cpd8 => 2,
            Self::CpHL => 1,
            Self::DI => 1,
        }
    }

    // Get number of cycles of opcode.
    //
    // First element is cycles if jump is not takem
    // Second element is cycles if jump is taken
    //
    // Note: For prefix instructions number of cycles is 0,
    // check CpOpcode to get correct cycles
    pub fn cycles(&self, jumped: bool) -> u8 {
        match self {
            Self::Nop => 1,
            Self::DecB => 1,
            Self::IncB => 1,
            Self::IncH => 1,
            Self::IncC => 1,
            Self::DecC => 1,
            Self::DecD => 1,
            Self::DecE => 1,
            Self::DecBC => 2,
            Self::LdBd8 => 2,
            Self::LdCd8 => 2,
            Self::LdDd8 => 2,
            Self::LdEd8 => 2,
            Self::Stop => 1,
            Self::LdBCd16 => 3,
            Self::LdDEd16 => 3,
            Self::LdHLd16 => 3,
            Self::LdHLd8 => 3,
            Self::IncDE => 2,
            Self::RlA => 1,
            Self::LdADE => 2,
            Self::Jr => 3,
            Self::Jrnz => {
                if jumped {
                    3
                } else {
                    2
                }
            }
            Self::Jrz => {
                if jumped {
                    3
                } else {
                    2
                }
            }
            Self::Jp => 4,
            Self::LdHLincA => 2,
            Self::IncHL => 2,
            Self::LdLd8 => 2,
            Self::LdSPd16 => 3,
            Self::LdHLdecA => 2,
            Self::DecA => 2,
            Self::LdAHLinc => 2,
            Self::LdAd8 => 2,
            Self::LdCA => 1,
            Self::LdDA => 1,
            Self::LdHA => 1,
            Self::LdHB => 1,
            Self::LdHLA => 2,
            Self::LdAB => 1,
            Self::LdAD => 1,
            Self::LdAE => 1,
            Self::LdAH => 1,
            Self::LdAL => 1,
            Self::LdBA => 1,
            Self::AddAA => 1,
            Self::AddAB => 1,
            Self::AddAHL => 2,
            Self::SubAB => 1,
            Self::SubAL => 1,
            Self::Andd8 => 2,
            Self::AndB => 1,
            Self::OrC => 1,
            Self::XorA => 1,
            Self::PopBC => 3,
            Self::PopDE => 3,
            Self::PushBC => 4,
            Self::PushDE => 4,
            Self::Ret => 4,
            Self::Prefix => 0,
            Self::Calla16 => 6,
            Self::Ldha8A => 3,
            Self::Lda16A => 4,
            Self::LdhCA => 2,
            Self::LdhAa8 => 3,
            Self::Cpd8 => 2,
            Self::CpHL => 2,
            Self::DI => 1,
        }
    }
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum CbOpcode {
    RlC = 0x11,
    SlaB = 0x20,
    Bit7H = 0x7C,
    Res0A = 0x87
}
}

impl CbOpcode {
    pub fn cycles(&self) -> u8 {
        match self {
            Self::RlC => 2,
            Self::SlaB => 2,
            Self::Bit7H => 2,
            Self::Res0A => 2,
        }
    }
}
