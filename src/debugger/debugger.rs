use std::sync::RwLock;

use crate::gbr::cpu::CpuState;
use crate::gbr::game_boy::GameBoy;
use crate::gbr::instruction::Instruction;
use crate::gbr::io_registers::IORegisters;
use crate::gbr::video_driver::VideoDriver;

pub struct Debugger {
    emu: RwLock<GameBoy>,
    // video_driver: VideoDriver,
}

impl Debugger {
    pub fn new(game_boy: GameBoy) -> Self {
        Debugger {
            emu: RwLock::new(game_boy),
            // video_driver: VideoDriver::new(190, 144),
        }
    }

    pub fn step(&self) -> bool {
        let mut emu = self.emu.write().unwrap();
        match emu.step() {
            Ok(()) => true,
            Err(()) => false,
        }
        // self.video_driver.draw(&self.emu);
    }

    pub fn disassemble(&self) -> Vec<(u16, Option<Instruction>)> {
        let mut pc = self.emu.read().unwrap().cpu().read_pc();

        let mut disassembly = Vec::<(u16, Option<Instruction>)>::new();

        for _ in 0..20 {
            let instruction = match self.emu.read().unwrap().bus().fetch_instruction(pc) {
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
        self.emu.read().unwrap().cpu().state()
    }

    pub fn io_registers(&self) -> IORegisters {
        *self.emu.read().unwrap().bus().io_registers()
    }
}
