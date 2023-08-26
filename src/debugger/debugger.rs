use std::sync::{
    mpsc::{channel, Sender},
    Arc, RwLock,
};

use log::warn;

use crate::gbr::instruction::Instruction;
use crate::gbr::io_registers::IORegisters;
use crate::gbr::{cpu::CpuState, ppu::ScreenBuffer};
use crate::gbr::{game_boy::GameBoy, ppu::TileList};

pub type AsmState = Vec<(u16, Option<Instruction>)>;

pub enum DebuggerCommand {
    Run,
    Stop,
    Pause,
    Step,
    SetBreakpoint(BreakPoint),
    UnsetBreakpoint(BreakPoint),
}

pub struct BreakPoint {
    pc: u16,
}

pub enum EmuState {
    Idle,
    Running,
    Error,
}

pub struct Debugger {
    cpu_state: (flume::Sender<CpuState>, flume::Receiver<CpuState>),
    io_registers_state: (flume::Sender<IORegisters>, flume::Receiver<IORegisters>),
    tiles_state: (flume::Sender<TileList>, flume::Receiver<TileList>),
    emu_state: (flume::Sender<EmuState>, flume::Receiver<EmuState>),
    cmd_ch: Option<Sender<DebuggerCommand>>,
    render_slot: flume::Receiver<ScreenBuffer>,
    asm: AsmState,
}

impl Debugger {
    pub fn new(game_boy: Arc<RwLock<GameBoy>>) -> Self {
        let render_slot = game_boy.read().unwrap().ppu().render_watch();
        let asm = Self::disassemble(game_boy);

        let debugger = Debugger {
            cpu_state: flume::bounded(1),
            io_registers_state: flume::bounded(1),
            tiles_state: flume::bounded(1),
            emu_state: flume::bounded(1),
            cmd_ch: None,
            render_slot,
            asm,
        };

        debugger.emu_state.0.try_send(EmuState::Idle).ok();

        debugger
    }

    pub fn attach(gb: Arc<RwLock<GameBoy>>) -> Self {
        let mut debugger = Debugger::new(gb.clone());

        debugger.run(gb);

        debugger
    }

    fn run(&mut self, emu: Arc<RwLock<GameBoy>>) {
        let (cmd_sig, cmd_slot) = channel::<DebuggerCommand>();
        self.cmd_ch = Some(cmd_sig);

        let cpu_state_sig = self.cpu_state.0.clone();
        let io_registers_state_sig = self.io_registers_state.0.clone();
        let tiles_state_sig = self.tiles_state.0.clone();
        let emu_state_sig = self.emu_state.0.clone();

        let cpu_state_recv = self.cpu_state.1.clone();
        let io_regs_state_recv = self.io_registers_state.1.clone();
        let tiles_state_recv = self.tiles_state.1.clone();

        self.emu_state.0.try_send(EmuState::Running).ok();
        std::thread::spawn(move || {
            let mut emu: std::sync::RwLockWriteGuard<'_, GameBoy> = emu.write().unwrap();

            let mut run_mode = false;

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

                let cmd = if !run_mode {
                    cmd_slot.recv().ok()
                } else {
                    cmd_slot.try_recv().ok()
                };

                match cmd {
                    Some(DebuggerCommand::Run) => {
                        emu_state_sig.try_send(EmuState::Running).ok();
                        run_mode = true;
                    }
                    Some(DebuggerCommand::Stop) => {
                        emu_state_sig.try_send(EmuState::Idle).ok();
                        emu.reset();
                        run_mode = false;
                    }
                    Some(DebuggerCommand::Pause) => {
                        emu_state_sig.try_send(EmuState::Idle).ok();
                        run_mode = false;
                    }
                    Some(DebuggerCommand::SetBreakpoint(pc)) => {
                        warn!("Set breakpoint not implemented")
                    }
                    Some(DebuggerCommand::UnsetBreakpoint(pc)) => {
                        warn!("Unset breakpoint not implemented")
                    }
                    Some(DebuggerCommand::Step) | None => (),
                }

                if let Err(e) = emu.step() {
                    log::error!("emu error: {}", e);
                    emu_state_sig.try_send(EmuState::Error).ok();
                    break;
                }
            }
        });
    }

    pub fn send_cmd(&self, cmd: DebuggerCommand) -> Option<()> {
        self.cmd_ch.as_ref()?.send(cmd).unwrap();
        Some(())
    }

    pub fn asm(&self) -> &AsmState {
        &self.asm
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

    pub fn render_slot(&self) -> flume::Receiver<ScreenBuffer> {
        self.render_slot.clone()
    }

    pub fn disassemble(emu: Arc<RwLock<GameBoy>>) -> AsmState {
        let mut disassembly = AsmState::new();

        let mut pc = 0x0000;

        let emu = emu.read().unwrap();

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
