use std::{collections::HashSet, sync::RwLockWriteGuard};

use enum_primitive::FromPrimitive;
use flume::{Receiver, Sender};

use crate::gbr::game_boy::{self, DebugEvent, GameBoy, GbState};
use crate::gbr::{
    bus::BusAccess,
    instruction::{opcode::Opcode, Instruction},
    GbError,
};

pub type AsmState = Vec<(u16, Option<Instruction>)>;

pub struct Debugger {
    gb_state: (Sender<GbState>, Receiver<GbState>),
    asm_state: (Sender<AsmState>, Receiver<AsmState>),
    breakpoints: HashSet<u16>,
}

impl Debugger {
    pub fn new() -> Self {
        Debugger {
            gb_state: flume::bounded(1),
            asm_state: flume::bounded(1),
            breakpoints: HashSet::new(),
        }
    }

    pub fn gb_state_recv(&self) -> Receiver<GbState> {
        self.gb_state.1.clone()
    }

    pub fn asm_state_recv(&self) -> Receiver<AsmState> {
        self.asm_state.1.clone()
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

impl game_boy::Debugger for Debugger {
    fn handle_event(&mut self, emu: &RwLockWriteGuard<GameBoy>, ev: &game_boy::DebugEvent) {
        match ev {
            DebugEvent::SetBreakpoint(pc) => {
                self.breakpoints.insert(*pc);
            }
            DebugEvent::ClearBreakpoint(pc) => {
                self.breakpoints.remove(pc);
            }
            DebugEvent::DumpVram => log::info!("\n{}", emu.ppu().vram_dump()),
        }
    }

    fn send_state(&mut self, gb: &RwLockWriteGuard<GameBoy>) {
        self.gb_state.0.send(gb.collect_state()).ok();
        self.asm_state.0.send(Debugger::disassemble(gb)).ok();
    }

    fn should_break(&self, gb: &RwLockWriteGuard<GameBoy>) -> bool {
        self.breakpoints.contains(&gb.cpu().read_pc())
    }
}
