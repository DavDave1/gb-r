use log::error;
use std::fmt;

use crate::gbr::game_boy::GameBoy;
use crate::gbr::instruction::Instruction;
use crate::gbr::io_registers::IORegisters;

pub struct CpuState {
    pub af: u16,
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
    pub pc: u16,
    pub sp: u16,

    pub zero: bool,
    pub carry: bool,
    pub bcd_n: bool,
    pub bcd_h: bool,
}

impl fmt::Display for CpuState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Regsisters:\n
            AF {:#06X}, BC {:#06X}, DE {:#06X}, HL {:#06X}, PC {:#06X}, SP {:#06X}\n
            Flags:\n
            Z {}, C {}, BCD-N {}, BCD-H {}",
            self.af,
            self.bc,
            self.de,
            self.hl,
            self.pc,
            self.sp,
            self.zero,
            self.carry,
            self.bcd_n,
            self.bcd_h
        )
    }
}

pub struct Debugger {
    emu: GameBoy,
    is_running: bool,
    panicked: bool,
}

impl Debugger {
    pub fn new(game_boy: GameBoy) -> Self {
        Debugger {
            emu: game_boy,
            is_running: false,
            panicked: false,
        }
    }

    pub fn step(&mut self) {
        if self.panicked {
            error!("Cannot step because system is panicked");
            self.is_running = false;
        } else {
            match self.emu.step() {
                Ok(()) => (),
                Err(()) => self.panicked = true,
            }
        }
    }

    pub fn set_running(&mut self, is_running: bool) {
        self.is_running = is_running;
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn disassemble(&self) -> Vec<(u16, Option<Instruction>)> {
        let cpu = self.emu.cpu();
        let mut pc = cpu.read_pc();

        let mut disassembly = Vec::<(u16, Option<Instruction>)>::new();

        for _ in 0..20 {
            let instruction = match self.emu.bus().fetch_instruction(pc) {
                Ok(instr) => instr,
                Err(()) => {
                    disassembly.push((pc, None));
                    continue;
                }
            };

            let new_pc = pc
                + match instruction.opcode() {
                    Some(_) => instruction.length().unwrap(),
                    None => 1,
                };

            disassembly.push((pc, Some(instruction)));
            pc = new_pc;
        }

        disassembly
    }

    pub fn cpu_state(&self) -> CpuState {
        let cpu = self.emu.cpu();

        CpuState {
            af: cpu.read_af(),
            bc: cpu.read_bc(),
            de: cpu.read_de(),
            hl: cpu.read_hl(),
            pc: cpu.read_pc(),
            sp: cpu.read_sp(),
            zero: cpu.get_zero_flag(),
            carry: cpu.get_carry_flag(),
            bcd_h: cpu.get_bcd_h_flag(),
            bcd_n: cpu.get_bcd_n_flag(),
        }
    }

    pub fn io_registers(&self) -> &IORegisters {
        &self.emu.bus().io_registers()
    }
}
