use std::{
    collections::HashSet,
    sync::{Arc, RwLock, RwLockWriteGuard},
};

use enum_primitive::FromPrimitive;

use crate::gbr::game_boy::{GameBoy, GbState};
use crate::gbr::{
    bus::BusAccess,
    instruction::{opcode::Opcode, Instruction},
    GbError,
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

    fn fetch_instruction(pc: u16, bus: &dyn BusAccess) -> Result<Instruction, GbError> {
        let opcode_data = bus.read_byte(pc)?;

        let opcode =
            Opcode::from_u8(opcode_data).ok_or(GbError::UnknownInstruction(opcode_data))?;

        let byte = if opcode.length() == 2 {
            Some(bus.read_byte(pc + 1)?)
        } else {
            None
        };

        let word = if opcode.length() == 3 {
            Some(bus.read_word(pc + 1)?)
        } else {
            None
        };

        Instruction::decode(opcode, byte, word)
    }

    pub fn disassemble(emu: &RwLockWriteGuard<GameBoy>) -> AsmState {
        let mut disassembly = AsmState::new();
        disassembly.reserve(20);

        let mut pc = emu.cpu().reg_pc_prev;

        loop {
            let instruction = match Debugger::fetch_instruction(pc, emu.bus()) {
                Ok(instr) => instr,
                Err(_) => {
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
