use std::sync::{
    mpsc::{channel, Sender},
    Arc, RwLock,
};

use crate::gbr::cpu::CpuState;
use crate::gbr::instruction::Instruction;
use crate::gbr::io_registers::IORegisters;
use crate::gbr::{bus::Bus, game_boy::GameBoy};

pub type AsmState = Vec<(u16, Option<Instruction>)>;

pub enum DebuggerCommand {
    Stop,
}

pub struct Debugger {
    emu: Arc<RwLock<GameBoy>>,
    cpu_state: (flume::Sender<CpuState>, flume::Receiver<CpuState>),
    asm_state: (flume::Sender<AsmState>, flume::Receiver<AsmState>),
    io_registers_state: (flume::Sender<IORegisters>, flume::Receiver<IORegisters>),
}

impl Debugger {
    pub fn new(game_boy: Arc<RwLock<GameBoy>>) -> Self {
        Debugger {
            emu: game_boy,
            cpu_state: flume::bounded(1),
            asm_state: flume::bounded(1),
            io_registers_state: flume::bounded(1),
        }
    }

    pub fn step(&self) {
        if let Ok(mut emu) = self.emu.try_write() {
            emu.step().map_err(|e| log::error!("emu error: {}", e)).ok();

            self.cpu_state.0.try_send(emu.cpu().state()).ok();
            self.io_registers_state
                .0
                .try_send(*emu.bus().io_registers())
                .ok();

            self.asm_state
                .0
                .try_send(Debugger::disassemble(emu.cpu().read_pc(), emu.bus()))
                .ok();
        } else {
            log::warn!("emu non accessible");
        }
    }

    pub fn run(&self) -> Sender<DebuggerCommand> {
        let (cmd_sig, cmd_slot) = channel::<DebuggerCommand>();

        let emu = self.emu.clone();

        let cpu_state_sig = self.cpu_state.0.clone();
        let asm_state_sig = self.asm_state.0.clone();
        let io_registers_state_sig = self.io_registers_state.0.clone();

        std::thread::spawn(move || {
            let mut emu = emu.write().unwrap();
            loop {
                if let Err(e) = emu.step() {
                    log::error!("emu error: {}", e);
                    break;
                }

                match cmd_slot.try_recv() {
                    Ok(DebuggerCommand::Stop) => break,
                    _ => (),
                }

                cpu_state_sig.try_send(emu.cpu().state()).ok();
                io_registers_state_sig
                    .try_send(*emu.bus().io_registers())
                    .ok();

                asm_state_sig
                    .try_send(Debugger::disassemble(emu.cpu().read_pc(), emu.bus()))
                    .ok();
            }
        });

        cmd_sig
    }

    pub fn asm_state(&self) -> Option<AsmState> {
        self.asm_state.1.try_recv().ok()
    }

    pub fn cpu_state(&self) -> Option<CpuState> {
        self.cpu_state.1.try_recv().ok()
    }

    pub fn io_registers_state(&self) -> Option<IORegisters> {
        self.io_registers_state.1.try_recv().ok()
    }

    fn disassemble(mut pc: u16, bus: &Bus) -> AsmState {
        let mut disassembly = AsmState::new();

        for _ in 0..20 {
            let instruction = match bus.fetch_instruction(pc) {
                Ok(instr) => instr,
                Err(e) => {
                    log::error!("dissassemble error: {}", e);
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
}
