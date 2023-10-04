macro_rules! jumped {
    ($jumped:expr, $if_true:expr, $if_false:expr) => {
        if $jumped {
            $if_true
        } else {
            $if_false
        }
    };
}

enum_from_primitive! {
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Opcode {
    Nop = 0x00,
    LdBCd16 = 0x01,
    LdBCA  = 0x02,
    IncBC = 0x03,
    IncB = 0x04,
    DecB = 0x05,
    LdBd8 = 0x06,
    RlcA = 0x07,
    Lda16SP = 0x08,
    AddHLBC = 0x09,
    LdABC = 0x0A,
    DecBC = 0x0B,
    IncC = 0x0C,
    DecC = 0x0D,
    LdCd8 = 0x0E,
    RrcA = 0x0F,
    Stop = 0x10,
    LdDEd16 = 0x11,
    LdDEA = 0x12,
    IncDE = 0x13,
    IncD = 0x14,
    DecD = 0x15,
    LdDd8 = 0x16,
    RlA = 0x17,
    Jr = 0x18,
    AddHLDE = 0x19,
    LdADE = 0x1A,
    DecDE = 0x1B,
    IncE = 0x1C,
    DecE = 0x1D,
    LdEd8 = 0x1E,
    RrA = 0x1F,
    JrNZ = 0x20,
    LdHLd16 = 0x21,
    LdHLincA = 0x22,
    IncHL = 0x23,
    IncH = 0x24,
    DecH = 0x25,
    LdHd8 = 0x26,
    DaA = 0x27,
    JrZ = 0x28,
    AddHLHL = 0x29,
    LdAHLinc = 0x2A,
    DecHL = 0x2B,
    IncL = 0x2C,
    DecL = 0x2D,
    LdLd8 = 0x2E,
    Cpl = 0x2F,
    JrNC = 0x30,
    LdSPd16 = 0x31,
    LdHLdecA = 0x32,
    IncSP = 0x33,
    Inca16HL = 0x34,
    Deca16HL = 0x35,
    LdHLd8 = 0x36,
    Scf = 0x37,
    JrC = 0x38,
    AddHLSP = 0x39,
    LdAHLdec = 0x3A,
    DecSP = 0x3B,
    IncA = 0x3C,
    DecA = 0x3D,
    LdAd8 = 0x3E,
    Ccf = 0x3F,
    LdBB = 0x40,
    LdBC = 0x41,
    LdBD = 0x42,
    LdBE = 0x43,
    LdBH = 0x44,
    LdBL = 0x45,
    LdBHL = 0x46,
    LdBA = 0x47,
    LdCB = 0x48,
    LdCC = 0x49,
    LdCD = 0x4A,
    LdCE = 0x4B,
    LdCH = 0x4C,
    LdCL = 0x4D,
    LdCHL = 0x4E,
    LdCA = 0x4F,
    LdDB = 0x50,
    LdDC = 0x51,
    LdDD = 0x52,
    LdDE = 0x53,
    LdDH = 0x54,
    LdDL = 0x55,
    LdDHL = 0x56,
    LdDA = 0x57,
    LdEB = 0x58,
    LdEC = 0x59,
    LdED = 0x5A,
    LdEE = 0x5B,
    LdEH = 0x5C,
    LdEL = 0x5D,
    LdEHL = 0x5E,
    LdEA = 0x5F,
    LdHB = 0x60,
    LdHC = 0x61,
    LdHD = 0x62,
    LdHE = 0x63,
    LdHH = 0x64,
    LdHL = 0x65,
    LdHHL = 0x66,
    LdHA = 0x67,
    LdLB = 0x68,
    LdLC = 0x69,
    LdLD = 0x6A,
    LdLE = 0x6B,
    LdLH = 0x6C,
    LdLL = 0x6D,
    LdLHL = 0x6E,
    LdLA = 0x6F,
    LdHLB = 0x70,
    LdHLC = 0x71,
    LdHLD = 0x72,
    LdHLE = 0x73,
    LdHLH = 0x74,
    LdHLL = 0x75,
    Halt = 0x76,
    LdHLA = 0x77,
    LdAB = 0x78,
    LdAC = 0x79,
    LdAD = 0x7A,
    LdAE = 0x7B,
    LdAH = 0x7C,
    LdAL = 0x7D,
    LdAHL = 0x7E,
    LdAA = 0x7F,
    AddAB = 0x80,
    AddAC = 0x81,
    AddAD = 0x82,
    AddAE = 0x83,
    AddAH = 0x84,
    AddAL = 0x85,
    AddAHL = 0x86,
    AddAA = 0x87,
    AdcAB = 0x88,
    AdcAC = 0x89,
    AdcAD = 0x8A,
    AdcAE = 0x8B,
    AdcAH = 0x8C,
    AdcAL = 0x8D,
    AdcAHL = 0x8E,
    AdcAA = 0x8F,
    SubAB = 0x90,
    SubAC = 0x91,
    SubAD = 0x92,
    SubAE = 0x93,
    SubAH = 0x94,
    SubAL = 0x95,
    SubAHL = 0x96,
    SubAA = 0x97,
    SbcAB = 0x98,
    SbcAC = 0x99,
    SbcAD = 0x9A,
    SbcAE = 0x9B,
    SbcAH = 0x9C,
    SbcAL = 0x9D,
    SbcAHL = 0x9E,
    SbcAA = 0x9F,
    AndB = 0xA0,
    AndC = 0xA1,
    AndD = 0xA2,
    AndE = 0xA3,
    AndH = 0xA4,
    AndL = 0xA5,
    AndHL = 0xA6,
    AndA = 0xA7,
    XorB = 0xA8,
    XorC = 0xA9,
    XorD = 0xAA,
    XorE = 0xAB,
    XorH = 0xAC,
    XorL = 0xAD,
    XorHL = 0xAE,
    XorA = 0xAF,
    OrB = 0xB0,
    OrC = 0xB1,
    OrD = 0xB2,
    OrE = 0xB3,
    OrH = 0xB4,
    OrL = 0xB5,
    OrHL = 0xB6,
    OrA = 0xB7,
    CpB = 0xB8,
    CpC = 0xB9,
    CpD = 0xBA,
    CpE = 0xBB,
    CpH = 0xBC,
    CpL = 0xBD,
    CpHL = 0xBE,
    CpA = 0xBF,
    RetNZ = 0xC0,
    PopBC = 0xC1,
    JpNZ = 0xC2,
    Jp = 0xC3,
    CallNZ = 0xC4,
    PushBC = 0xC5,
    AddAd8 = 0xC6,
    Rst0 = 0xC7,
    RetZ = 0xC8,
    Ret = 0xC9,
    JpZ = 0xCA,
    Prefix = 0xCB,
    CallZ = 0xCC,
    Call = 0xCD,
    AdcAd8 = 0xCE,
    Rst1 = 0xCF,
    RetNC = 0xD0,
    PopDE = 0xD1,
    JpNC = 0xD2,
    CallNC = 0xD4,
    PushDE = 0xD5,
    SubAd8 = 0xD6,
    Rst2 = 0xD7,
    RetC = 0xD8,
    RetI = 0xD9,
    JpC = 0xDA,
    CallC = 0xDC,
    SbcAd8 = 0xDE,
    Rst3 = 0xDF,
    Ldha8A = 0xE0,
    PopHL = 0xE1,
    LdioCA = 0xE2,
    PushHL = 0xE5,
    Andd8 = 0xE6,
    Rst4 = 0xE7,
    AddSPs8 = 0xE8,
    JpHL = 0xE9,
    Lda16A = 0xEA,
    Xord8 = 0xEE,
    Rst5 = 0xEF,
    LdhAa8 = 0xF0,
    PopAF = 0xF1,
    LdAioC = 0xF2,
    Di = 0xF3,
    PushAF = 0xF5,
    Ord8 = 0xF6,
    Rst6 = 0xF7,
    LdHLSPs8 = 0xF8,
    LdSPHL = 0xF9,
    LdAa16 = 0xFA,
    Ei = 0xFB,
    Cpd8 = 0xFE,
    Rst7 = 0xFF,
}
}

impl Opcode {
    pub fn length(&self) -> u8 {
        match self {
            Self::LdAd8
            | Self::LdBd8
            | Self::LdCd8
            | Self::LdDd8
            | Self::LdEd8
            | Self::LdHd8
            | Self::LdLd8
            | Self::LdHLd8
            | Self::Ldha8A
            | Self::LdhAa8
            | Self::LdHLSPs8
            | Self::AddAd8
            | Self::AdcAd8
            | Self::SubAd8
            | Self::SbcAd8
            | Self::Andd8
            | Self::Xord8
            | Self::Ord8
            | Self::Cpd8
            | Self::AddSPs8
            | Self::Jr
            | Self::JrZ
            | Self::JrNZ
            | Self::JrC
            | Self::JrNC
            | Self::Prefix => 2,

            Self::LdBCd16
            | Self::LdDEd16
            | Self::LdHLd16
            | Self::LdSPd16
            | Self::LdAa16
            | Self::Lda16A
            | Self::Lda16SP
            | Self::Jp
            | Self::JpZ
            | Self::JpNZ
            | Self::JpC
            | Self::JpNC
            | Self::Call
            | Self::CallZ
            | Self::CallNZ
            | Self::CallC
            | Self::CallNC => 3,
            _ => 1,
        }
    }

    // Get number of cycles of opcode.
    //
    // First element is cycles if jump is not takem
    // Second element is cycles if jump is taken
    //
    // Note: For prefix instructions number of cycles is 0,
    // check CpOpcode to get correct cycles
    pub fn cycles(&self, j: bool) -> u8 {
        match self {
            Self::Prefix => 0,
            Self::LdAd8
            | Self::LdBd8
            | Self::LdCd8
            | Self::LdDd8
            | Self::LdEd8
            | Self::LdHd8
            | Self::LdLd8
            | Self::LdBCA
            | Self::LdDEA
            | Self::LdHLincA
            | Self::LdHLdecA
            | Self::LdHLB
            | Self::LdHLC
            | Self::LdHLD
            | Self::LdHLE
            | Self::LdHLH
            | Self::LdHLL
            | Self::LdABC
            | Self::LdADE
            | Self::LdAHLinc
            | Self::LdAHLdec
            | Self::LdAHL
            | Self::LdBHL
            | Self::LdCHL
            | Self::LdDHL
            | Self::LdEHL
            | Self::LdHHL
            | Self::LdLHL
            | Self::LdSPHL
            | Self::LdAioC
            | Self::LdioCA
            | Self::AddAd8
            | Self::AdcAd8
            | Self::SubAd8
            | Self::SbcAd8
            | Self::Andd8
            | Self::Xord8
            | Self::Ord8
            | Self::Cpd8
            | Self::IncBC
            | Self::IncDE
            | Self::IncHL
            | Self::IncSP
            | Self::DecBC
            | Self::DecDE
            | Self::DecHL
            | Self::DecSP
            | Self::AddHLBC
            | Self::AddHLDE
            | Self::AddHLHL
            | Self::AddHLSP
            | Self::AddAHL
            | Self::AdcAHL
            | Self::SubAHL
            | Self::SbcAHL
            | Self::AndHL
            | Self::XorHL
            | Self::OrHL
            | Self::CpHL => 2,

            Self::LdBCd16
            | Self::LdDEd16
            | Self::LdHLd16
            | Self::LdSPd16
            | Self::LdHLd8
            | Self::Ldha8A
            | Self::LdhAa8
            | Self::LdHLSPs8
            | Self::PopAF
            | Self::PopBC
            | Self::PopDE
            | Self::PopHL
            | Self::Inca16HL
            | Self::Deca16HL => 3,

            Self::Jr | Self::JrZ | Self::JrNZ | Self::JrC | Self::JrNC => jumped!(j, 3, 2),
            Self::Jp | Self::JpZ | Self::JpNZ | Self::JpC | Self::JpNC => jumped!(j, 4, 3),
            Self::Call | Self::CallZ | Self::CallNZ | Self::CallC | Self::CallNC => {
                jumped!(j, 6, 3)
            }
            Self::RetZ | Self::RetNZ | Self::RetC | Self::RetNC => jumped!(j, 5, 2),
            Self::Lda16A
            | Self::LdAa16
            | Self::AddSPs8
            | Self::PushAF
            | Self::PushBC
            | Self::PushDE
            | Self::PushHL
            | Self::Rst0
            | Self::Rst1
            | Self::Rst2
            | Self::Rst3
            | Self::Rst4
            | Self::Rst5
            | Self::Rst6
            | Self::Rst7
            | Self::Ret
            | Self::RetI => 4,
            Self::Lda16SP => 5,
            _ => 1,
        }
    }
}

enum_from_primitive! {
#[derive(Debug, PartialEq, Clone)]
pub enum CbOpcode {
    RlcB = 0x00,
    RlcC = 0x01,
    RlcD = 0x02,
    RlcE = 0x03,
    RlcH = 0x04,
    RlcL = 0x05,
    RlcHL = 0x06,
    RlcA = 0x07,
    RrcB = 0x08,
    RrcC = 0x09,
    RrcD = 0x0A,
    RrcE = 0x0B,
    RrcH = 0x0C,
    RrcL = 0x0D,
    RrcHL = 0x0E,
    RrcA = 0x0F,
    RlB = 0x10,
    RlC = 0x11,
    RlD = 0x12,
    RlE = 0x13,
    RlH = 0x14,
    RlL = 0x15,
    RlHL = 0x16,
    RlA = 0x17,
    RrB = 0x18,
    RrC = 0x19,
    RrD = 0x1A,
    RrE = 0x1B,
    RrH = 0x1C,
    RrL = 0x1D,
    RrHL = 0x1E,
    RrA = 0x1F,
    SlaB = 0x20,
    SlaC = 0x21,
    SlaD = 0x22,
    SlaE = 0x23,
    SlaH = 0x24,
    SlaL = 0x25,
    SlaHL = 0x26,
    SlaA = 0x27,
    SraB = 0x28,
    SraC = 0x29,
    SraD = 0x2A,
    SraE = 0x2B,
    SraH = 0x2C,
    SraL = 0x2D,
    SraHL = 0x2E,
    SraA = 0x2F,
    SwapB = 0x30,
    SwapC = 0x31,
    SwapD = 0x32,
    SwapE = 0x33,
    SwapH = 0x34,
    SwapL = 0x35,
    SwapHL = 0x36,
    SwapA = 0x37,
    SrlB = 0x38,
    SrlC = 0x39,
    SrlD = 0x3A,
    SrlE = 0x3B,
    SrlH = 0x3C,
    SrlL = 0x3D,
    SrlHL = 0x3E,
    SrlA = 0x3F,
    Bit0B = 0x40,
    Bit0C = 0x41,
    Bit0D = 0x42,
    Bit0E = 0x43,
    Bit0H = 0x44,
    Bit0L = 0x45,
    Bit0HL = 0x46,
    Bit0A = 0x47,
    Bit1B = 0x48,
    Bit1C = 0x49,
    Bit1D = 0x4A,
    Bit1E = 0x4B,
    Bit1H = 0x4C,
    Bit1L = 0x4D,
    Bit1HL = 0x4E,
    Bit1A = 0x4F,
    Bit2B = 0x50,
    Bit2C = 0x51,
    Bit2D = 0x52,
    Bit2E = 0x53,
    Bit2H = 0x54,
    Bit2L = 0x55,
    Bit2HL = 0x56,
    Bit2A = 0x57,
    Bit3B = 0x58,
    Bit3C = 0x59,
    Bit3D = 0x5A,
    Bit3E = 0x5B,
    Bit3H = 0x5C,
    Bit3L = 0x5D,
    Bit3HL = 0x5E,
    Bit3A = 0x5F,
    Bit4B = 0x60,
    Bit4C = 0x61,
    Bit4D = 0x62,
    Bit4E = 0x63,
    Bit4H = 0x64,
    Bit4L = 0x65,
    Bit4HL = 0x66,
    Bit4A = 0x67,
    Bit5B = 0x68,
    Bit5C = 0x69,
    Bit5D = 0x6A,
    Bit5E = 0x6B,
    Bit5H = 0x6C,
    Bit5L = 0x6D,
    Bit5HL = 0x6E,
    Bit5A = 0x6F,
    Bit6B = 0x70,
    Bit6C = 0x71,
    Bit6D = 0x72,
    Bit6E = 0x73,
    Bit6H = 0x74,
    Bit6L = 0x75,
    Bit6HL = 0x76,
    Bit6A = 0x77,
    Bit7B = 0x78,
    Bit7C = 0x79,
    Bit7D = 0x7A,
    Bit7E = 0x7B,
    Bit7H = 0x7C,
    Bit7L = 0x7D,
    Bit7HL = 0x7E,
    Bit7A = 0x7F,
    Res0B = 0x80,
    Res0C = 0x81,
    Res0D = 0x82,
    Res0E = 0x83,
    Res0H = 0x84,
    Res0L = 0x85,
    Res0HL= 0x86,
    Res0A = 0x87,
    Res1B = 0x88,
    Res1C = 0x89,
    Res1D = 0x8A,
    Res1E = 0x8B,
    Res1H = 0x8C,
    Res1L = 0x8D,
    Res1HL= 0x8E,
    Res1A = 0x8F,
    Res2B = 0x90,
    Res2C = 0x91,
    Res2D = 0x92,
    Res2E = 0x93,
    Res2H = 0x94,
    Res2L = 0x95,
    Res2HL= 0x96,
    Res2A = 0x97,
    Res3B = 0x98,
    Res3C = 0x99,
    Res3D = 0x9A,
    Res3E = 0x9B,
    Res3H = 0x9C,
    Res3L = 0x9D,
    Res3HL= 0x9E,
    Res3A = 0x9F,
    Res4B = 0xA0,
    Res4C = 0xA1,
    Res4D = 0xA2,
    Res4E = 0xA3,
    Res4H = 0xA4,
    Res4L = 0xA5,
    Res4HL= 0xA6,
    Res4A = 0xA7,
    Res5B = 0xA8,
    Res5C = 0xA9,
    Res5D = 0xAA,
    Res5E = 0xAB,
    Res5H = 0xAC,
    Res5L = 0xAD,
    Res5HL= 0xAE,
    Res5A = 0xAF,
    Res6B = 0xB0,
    Res6C = 0xB1,
    Res6D = 0xB2,
    Res6E = 0xB3,
    Res6H = 0xB4,
    Res6L = 0xB5,
    Res6HL= 0xB6,
    Res6A = 0xB7,
    Res7B = 0xB8,
    Res7C = 0xB9,
    Res7D = 0xBA,
    Res7E = 0xBB,
    Res7H = 0xBC,
    Res7L = 0xBD,
    Res7HL= 0xBE,
    Res7A = 0xBF,
    Set0B = 0xC0,
    Set0C = 0xC1,
    Set0D = 0xC2,
    Set0E = 0xC3,
    Set0H = 0xC4,
    Set0L = 0xC5,
    Set0HL= 0xC6,
    Set0A = 0xC7,
    Set1B = 0xC8,
    Set1C = 0xC9,
    Set1D = 0xCA,
    Set1E = 0xCB,
    Set1H = 0xCC,
    Set1L = 0xCD,
    Set1HL= 0xCE,
    Set1A = 0xCF,
    Set2B = 0xD0,
    Set2C = 0xD1,
    Set2D = 0xD2,
    Set2E = 0xD3,
    Set2H = 0xD4,
    Set2L = 0xD5,
    Set2HL= 0xD6,
    Set2A = 0xD7,
    Set3B = 0xD8,
    Set3C = 0xD9,
    Set3D = 0xDA,
    Set3E = 0xDB,
    Set3H = 0xDC,
    Set3L = 0xDD,
    Set3HL= 0xDE,
    Set3A = 0xDF,
    Set4B = 0xE0,
    Set4C = 0xE1,
    Set4D = 0xE2,
    Set4E = 0xE3,
    Set4H = 0xE4,
    Set4L = 0xE5,
    Set4HL= 0xE6,
    Set4A = 0xE7,
    Set5B = 0xE8,
    Set5C = 0xE9,
    Set5D = 0xEA,
    Set5E = 0xEB,
    Set5H = 0xEC,
    Set5L = 0xED,
    Set5HL= 0xEE,
    Set5A = 0xEF,
    Set6B = 0xF0,
    Set6C = 0xF1,
    Set6D = 0xF2,
    Set6E = 0xF3,
    Set6H = 0xF4,
    Set6L = 0xF5,
    Set6HL= 0xF6,
    Set6A = 0xF7,
    Set7B = 0xF8,
    Set7C = 0xF9,
    Set7D = 0xFA,
    Set7E = 0xFB,
    Set7H = 0xFC,
    Set7L = 0xFD,
    Set7HL= 0xFE,
    Set7A = 0xFF,
}
}

impl CbOpcode {
    pub fn cycles(&self) -> u8 {
        match self {
            Self::RlcHL
            | Self::RrcHL
            | Self::RlHL
            | Self::RrHL
            | Self::SlaHL
            | Self::SraHL
            | Self::SwapHL
            | Self::SrlHL
            | Self::Bit0HL
            | Self::Bit1HL
            | Self::Bit2HL
            | Self::Bit3HL
            | Self::Bit4HL
            | Self::Bit5HL
            | Self::Bit6HL
            | Self::Bit7HL
            | Self::Res0HL
            | Self::Res1HL
            | Self::Res2HL
            | Self::Res3HL
            | Self::Res4HL
            | Self::Res5HL
            | Self::Res6HL
            | Self::Res7HL
            | Self::Set0HL
            | Self::Set1HL
            | Self::Set2HL
            | Self::Set3HL
            | Self::Set4HL
            | Self::Set5HL
            | Self::Set6HL
            | Self::Set7HL => 4,
            _ => 2,
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn jump_macro() {
        assert_eq!(jumped!(true, 4, 3), 4);
        assert_eq!(jumped!(false, 4, 3), 3)
    }
}
