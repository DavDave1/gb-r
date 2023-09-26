use std::{fs, path::PathBuf};

use byteorder::{ByteOrder, LittleEndian};

use super::{
    apu::APU, instruction::Instruction, interrupts::InterruptHandler, io_registers::IORegisters,
    mbc::MBC, memory_map::*, ppu::PPU, timer::Timer, GbError,
};

pub struct Bus {
    boot_rom_lock: bool,
    boot_rom: Box<[u8]>,
    hram: Box<[u8]>,
    wram: Box<[u8]>,
    wram_acv_bank: Box<[u8]>,
    io_registers: IORegisters,
    ppu: PPU,
    apu: APU,
    ir_handler: InterruptHandler,
    timer: Timer,
    mbc: MBC,
}

impl Bus {
    pub fn new(boot_rom_filename: Option<PathBuf>, cart_rom_filename: Option<PathBuf>) -> Self {
        let boot_rom_filename = boot_rom_filename.expect("Boot rom path not provided");
        let boot_rom = fs::read(boot_rom_filename).expect("Failed to read boot rom");

        if boot_rom.len() != BOOT_ROM_SIZE {
            panic!("Wrong boot rom size");
        }

        let mbc = match cart_rom_filename {
            Some(path) => MBC::new(&path).unwrap(),
            None => MBC::default(),
        };

        Bus {
            boot_rom_lock: true,
            boot_rom: boot_rom.into_boxed_slice(),
            hram: vec![0; HRAM_SIZE].into_boxed_slice(),
            wram: vec![0; WRAM_BANK0_SIZE].into_boxed_slice(),
            wram_acv_bank: vec![0; WRAM_ACTIVE_BANK_SIZE].into_boxed_slice(),
            io_registers: IORegisters::default(),
            ppu: PPU::new(),
            apu: APU::new(),
            ir_handler: InterruptHandler::default(),
            timer: Timer::default(),
            mbc,
        }
    }

    pub fn step(&mut self, cycles: u8) -> Result<bool, GbError> {
        self.timer.step(cycles, &mut self.ir_handler);
        self.apu.step(cycles)?;
        self.ppu.step(cycles)
    }

    pub fn reset(&mut self) {
        self.ppu.reset();
        self.io_registers = IORegisters::default();
    }

    pub fn ppu(&self) -> &PPU {
        &self.ppu
    }

    pub fn io_registers(&self) -> &IORegisters {
        &self.io_registers
    }

    pub fn ir_handler(&self) -> &InterruptHandler {
        &self.ir_handler
    }

    pub fn ir_handler_mut(&mut self) -> &mut InterruptHandler {
        &mut self.ir_handler
    }

    pub fn mbc(&self) -> &MBC {
        &self.mbc
    }

    pub fn fetch_instruction(&self, addr: u16) -> Result<Instruction, GbError> {
        match map_address(addr)? {
            MappedAddress::CartRom(addr) => {
                if self.boot_rom_lock {
                    let len = Instruction::peek_len(self.boot_rom[addr as usize])? as usize;
                    Instruction::decode(&self.boot_rom[addr as usize..addr as usize + len])
                } else {
                    let len = Instruction::peek_len(self.mbc.read_byte(addr)?)? as usize;

                    let mut instr_data = vec![0; len];
                    for i in 0..len as u16 {
                        instr_data[i as usize] = self.mbc.read_byte(addr + i)?;
                    }
                    Instruction::decode(&instr_data)
                }
            }
            _ => Err(GbError::Unimplemented(
                "fetching instruction outside bank 0".into(),
            )),
        }
    }

    pub fn read_byte(&self, addr: u16) -> Result<u8, GbError> {
        match map_address(addr)? {
            MappedAddress::CartRom(addr) => {
                if self.boot_rom_lock && (addr as usize) < BOOT_ROM_SIZE {
                    Ok(self.boot_rom[addr as usize])
                } else {
                    Ok(self.mbc.read_byte(addr)?)
                }
            }
            MappedAddress::VideoRam(addr) => self.ppu.read_byte(addr),
            MappedAddress::CartRam(addr) => self.mbc.read_byte(addr),
            MappedAddress::WorkRamBank0(addr) => Ok(self.wram[(addr - WRAM_BANK0_START) as usize]),
            MappedAddress::WorkRamActiveBank(addr) => {
                Ok(self.wram_acv_bank[(addr - WRAM_ACTIVE_BANK_START) as usize])
            }
            MappedAddress::SpriteAttributeTable(_addr) => Err(GbError::Unimplemented(
                "reading sprite attribute table".into(),
            )),
            MappedAddress::NotUsable(addr) => {
                log::warn!("Reading byte from unusable addr {:#06X}", addr);
                Ok(0xFF)
            }
            MappedAddress::TimerRegisters(addr) => self.timer.read_reg(addr),
            MappedAddress::ApuRegisters(addr) => self.apu.read_reg(addr),
            MappedAddress::PpuRegisters(addr) => self.ppu.read_reg(addr),
            MappedAddress::BootRomLockRegister => Err(GbError::IllegalOp(
                "reading from boot rom lock register".into(),
            )),
            MappedAddress::IORegisters(addr) => self.io_registers.read(addr),
            MappedAddress::HighRam(addr) => Ok(self.hram[addr as usize]),
            MappedAddress::InterruptFlagRegister => Err(GbError::Unimplemented(
                "reading interrupt flag register".into(),
            )),
            MappedAddress::InterruptEnableRegister => Ok(self.ir_handler.ime() as u8),
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) -> Result<(), GbError> {
        match map_address(addr)? {
            MappedAddress::CartRom(addr) => self.mbc.write_byte(addr, value)?,
            MappedAddress::VideoRam(addr) => self.ppu.write_byte(addr, value)?,
            MappedAddress::CartRam(addr) => self.mbc.write_byte(addr, value)?,
            MappedAddress::WorkRamBank0(addr) => {
                self.wram[(addr - WRAM_BANK0_START) as usize] = value;
            }
            MappedAddress::WorkRamActiveBank(addr) => {
                self.wram_acv_bank[(addr - WRAM_ACTIVE_BANK_START) as usize] = value;
            }
            MappedAddress::SpriteAttributeTable(_addr) => {
                return Err(GbError::Unimplemented(
                    "writing to sprite attribute table".into(),
                ));
            }
            MappedAddress::NotUsable(addr) => {
                log::warn!("Writing byte {:#04X} to unusable addr {:#06X}", value, addr);
            }
            MappedAddress::TimerRegisters(addr) => self.timer.write_reg(addr, value)?,
            MappedAddress::ApuRegisters(addr) => self.apu.write_reg(addr, value)?,
            MappedAddress::PpuRegisters(addr) => self.ppu.write_reg(addr, value)?,
            MappedAddress::BootRomLockRegister => self.boot_rom_lock = false,
            MappedAddress::IORegisters(addr) => self.io_registers.write(addr, value)?,
            MappedAddress::HighRam(addr) => self.hram[(addr - HRAM_START) as usize] = value,
            MappedAddress::InterruptFlagRegister => self.ir_handler.write_if(value),
            MappedAddress::InterruptEnableRegister => self.ir_handler.write_ie(value),
        }

        Ok(())
    }

    pub fn read_word(&self, addr: u16) -> Result<u16, GbError> {
        match map_address(addr)? {
            MappedAddress::CartRom(addr) => {
                if self.boot_rom_lock {
                    Ok(LittleEndian::read_u16(&self.boot_rom[addr as usize..]))
                } else {
                    self.mbc.read_word(addr)
                }
            }
            MappedAddress::VideoRam(addr) => self.ppu.read_word(addr),
            MappedAddress::CartRam(_addr) => self.mbc.read_word(addr),
            MappedAddress::WorkRamBank0(_addr) => {
                Err(GbError::Unimplemented("reading from work ram 0".into()))
            }
            MappedAddress::WorkRamActiveBank(addr) => Ok(LittleEndian::read_u16(
                &self.wram_acv_bank[(addr - WRAM_ACTIVE_BANK_START) as usize..],
            )),
            MappedAddress::SpriteAttributeTable(_addr) => Err(GbError::Unimplemented(
                "reading sprite attribute table".into(),
            )),
            MappedAddress::NotUsable(addr) => {
                log::warn!("Reading word from unusable addr {:#06X}", addr);
                Ok(0xFFFF)
            }
            MappedAddress::TimerRegisters(_addr) => {
                Err(GbError::IllegalOp("read word from Timer registers".into()))
            }
            MappedAddress::ApuRegisters(_addr) => {
                Err(GbError::IllegalOp("read word from APU registers".into()))
            }
            MappedAddress::PpuRegisters(_addr) => {
                Err(GbError::IllegalOp("read word from PPU registers".into()))
            }
            MappedAddress::BootRomLockRegister => Err(GbError::IllegalOp(
                "reading from boot rom lock register".into(),
            )),
            MappedAddress::IORegisters(_addr) => {
                Err(GbError::Unimplemented("reading IO registers".into()))
            }
            MappedAddress::HighRam(addr) => Ok(LittleEndian::read_u16(
                &self.hram[(addr - HRAM_START) as usize..],
            )),
            MappedAddress::InterruptFlagRegister => {
                Err(GbError::IllegalOp("reading interrupt flag register".into()))
            }
            MappedAddress::InterruptEnableRegister => Err(GbError::IllegalOp(
                "reading interrupt enable register".into(),
            )),
        }
    }
}
