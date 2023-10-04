use std::{
    collections::HashSet,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::gbr::{bus::BusAccess, instruction::Instruction};
use crate::gbr::{
    game_boy::{GameBoy, GbState},
    memory_map::BOOT_ROM_SIZE,
};

pub type AsmState = Vec<(u16, Option<Instruction>)>;

pub enum DebuggerCommand {
    Run,
    Stop,
    Pause,
    Step,
    SetBreakpoint(u16),
    UnsetBreakpoint(u16),
    DumpVram,
}

pub struct Debugger {
    pub gb_state: Arc<RwLock<GbState>>,
    pub asm_state: Arc<RwLock<AsmState>>,
    breakpoints: HashSet<u16>,
}

impl Debugger {
    pub fn new() -> Self {
        Debugger {
            gb_state: Default::default(),
            asm_state: Default::default(),
            breakpoints: HashSet::new(),
        }
    }

    pub fn add_breakpoint(&mut self, pc: u16) {
        self.breakpoints.insert(pc);
    }

    pub fn remove_breakpoint(&mut self, pc: u16) {
        self.breakpoints.remove(&pc);
    }

    pub fn should_break(&self, pc: u16) -> bool {
        self.breakpoints.contains(&pc)
    }

    pub fn disassemble(emu: &RwLockWriteGuard<GameBoy>) -> AsmState {
        let mut disassembly = AsmState::new();
        disassembly.reserve(20);

        let mut pc = emu.cpu().read_pc();

        loop {
            let instruction = match emu.bus().fetch_instruction(pc) {
                Ok(instr) => instr,
                Err(_) => {
                    // log::error!("disassemble error: {}", e);
                    disassembly.push((pc, None));
                    pc += 1;
                    continue;
                }
            };

            let new_pc = pc + instruction.len() as u16;
            disassembly.push((pc, Some(instruction)));
            pc = new_pc;

            if disassembly.len() >= 20 {
                break;
            }
        }

        disassembly
    }
}
