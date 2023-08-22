use std::sync::{
    mpsc::{channel, Sender},
    Arc, RwLock,
};

use crate::gbr::cpu::CpuState;
use crate::gbr::instruction::Instruction;
use crate::gbr::io_registers::IORegisters;
use crate::gbr::{game_boy::GameBoy, ppu::TileList};

pub type AsmState = Vec<(u16, Option<Instruction>)>;

pub enum DebuggerCommand {
    Stop,
}

pub enum EmuState {
    Reset,
    Running,
    Idle,
    Error,
}

pub struct Debugger {
    pub emu: Arc<RwLock<GameBoy>>,
    cpu_state: (flume::Sender<CpuState>, flume::Receiver<CpuState>),
    io_registers_state: (flume::Sender<IORegisters>, flume::Receiver<IORegisters>),
    tiles_state: (flume::Sender<TileList>, flume::Receiver<TileList>),
    emu_state: (flume::Sender<EmuState>, flume::Receiver<EmuState>),
}

impl Debugger {
    pub fn new(game_boy: Arc<RwLock<GameBoy>>) -> Self {
        let debugger = Debugger {
            emu: game_boy,
            cpu_state: flume::bounded(1),
            io_registers_state: flume::bounded(1),
            tiles_state: flume::bounded(1),
            emu_state: flume::bounded(1),
        };

        debugger.emu_state.0.try_send(EmuState::Reset).ok();

        debugger
    }

    pub fn step(&self) {
        if let Ok(mut emu) = self.emu.try_write() {
            emu.step()
                .map_err(|e| {
                    log::error!("emu error: {}", e);
                    self.emu_state.0.try_send(EmuState::Error).ok();
                })
                .ok();

            self.cpu_state.0.try_send(emu.cpu().state()).ok();
            self.io_registers_state
                .0
                .try_send(*emu.bus().io_registers())
                .ok();
        } else {
            log::warn!("emu non accessible");
        }
    }

    pub fn run(&self) -> Sender<DebuggerCommand> {
        let (cmd_sig, cmd_slot) = channel::<DebuggerCommand>();

        let emu = self.emu.clone();

        let cpu_state_sig = self.cpu_state.0.clone();
        let io_registers_state_sig = self.io_registers_state.0.clone();
        let tiles_state_sig = self.tiles_state.0.clone();
        let emu_state_sig = self.emu_state.0.clone();

        let cpu_state_recv = self.cpu_state.1.clone();
        let io_regs_state_recv = self.io_registers_state.1.clone();
        let tiles_state_recv = self.tiles_state.1.clone();

        self.emu_state.0.try_send(EmuState::Running).ok();
        std::thread::spawn(move || {
            let mut emu = emu.write().unwrap();
            loop {
                cpu_state_recv.drain();
                io_regs_state_recv.drain();
                tiles_state_recv.drain();

                cpu_state_sig.try_send(emu.cpu().state()).ok();
                io_registers_state_sig
                    .try_send(*emu.bus().io_registers())
                    .ok();

                tiles_state_sig
                    .try_send(emu.ppu().tile_list().to_vec())
                    .ok();

                if let Err(e) = emu.step() {
                    log::error!("emu error: {}", e);
                    emu_state_sig.try_send(EmuState::Error).ok();
                    break;
                }

                match cmd_slot.try_recv() {
                    Ok(DebuggerCommand::Stop) => {
                        emu_state_sig.try_send(EmuState::Idle).ok();
                        break;
                    }
                    _ => (),
                }
            }
        });

        cmd_sig
    }

    pub fn cpu_state(&self) -> Option<CpuState> {
        self.cpu_state.1.drain().last()
    }

    pub fn io_registers_state(&self) -> Option<IORegisters> {
        self.io_registers_state.1.drain().last()
    }

    pub fn tiles_state(&self) -> Option<TileList> {
        self.tiles_state.1.drain().last()
    }

    pub fn emu_state(&self) -> Option<EmuState> {
        self.emu_state.1.drain().last()
    }

    pub fn disassemble(&self) -> AsmState {
        let mut disassembly = AsmState::new();

        let mut pc = 0x0000;

        let emu = self.emu.read().unwrap();

        while pc < 0xFB {
            let instruction = match emu.bus().fetch_instruction(pc) {
                Ok(instr) => instr,
                Err(e) => {
                    log::error!("disassemble error: {}", e);
                    disassembly.push((pc, None));
                    pc += 1;
                    continue;
                }
            };

            let new_pc = pc + instruction.len();
            disassembly.push((pc, Some(instruction)));
            pc = new_pc;
        }

        disassembly
    }
}
