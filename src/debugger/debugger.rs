use std::{
    collections::HashSet,
    sync::{
        mpsc::{channel, Sender},
        Arc, RwLock,
    },
};

use crate::gbr::game_boy::GameBoy;
use crate::gbr::io_registers::IORegisters;
use crate::gbr::{cpu::CpuState, ppu::ScreenBuffer};
use crate::gbr::{instruction::Instruction, ppu::PpuState};

pub type AsmState = Vec<(u16, Option<Instruction>)>;

pub enum DebuggerCommand {
    Run,
    Stop,
    Pause,
    Step,
    SetBreakpoint(u16),
    UnsetBreakpoint(u16),
}

pub enum EmuState {
    Idle,
    Running,
    Error,
}

#[derive(Default)]
pub struct GbState {
    pub cpu: CpuState,
    pub io_registers: IORegisters,
    pub ppu: PpuState,
}

pub struct Debugger {
    emu_state: (flume::Sender<EmuState>, flume::Receiver<EmuState>),
    cmd_ch: Option<Sender<DebuggerCommand>>,
    render_slot: flume::Receiver<ScreenBuffer>,
    asm: AsmState,
    pub gb_state: Arc<RwLock<GbState>>,
}

impl Debugger {
    pub fn new(game_boy: Arc<RwLock<GameBoy>>) -> Self {
        let render_slot = game_boy.read().unwrap().ppu().render_watch();
        let asm = Self::disassemble(game_boy);

        let debugger = Debugger {
            emu_state: flume::bounded(1),
            cmd_ch: None,
            render_slot,
            asm,
            gb_state: Default::default(),
        };

        debugger.emu_state.0.try_send(EmuState::Idle).ok();

        debugger
    }

    pub fn attach(gb: Arc<RwLock<GameBoy>>) -> Self {
        let mut debugger = Debugger::new(gb.clone());

        debugger.run(gb, debugger.gb_state.clone());

        debugger
    }

    fn run(&mut self, emu: Arc<RwLock<GameBoy>>, state: Arc<RwLock<GbState>>) {
        let (cmd_sig, cmd_slot) = channel::<DebuggerCommand>();
        self.cmd_ch = Some(cmd_sig);

        let emu_state_sig = self.emu_state.0.clone();

        self.emu_state.0.try_send(EmuState::Running).ok();
        std::thread::spawn(move || {
            let mut emu: std::sync::RwLockWriteGuard<'_, GameBoy> = emu.write().unwrap();

            let mut running = false;
            let mut stepping = false;

            let mut breakpoints = HashSet::<u16>::new();

            loop {
                if let Ok(mut state) = state.try_write() {
                    state.cpu = emu.cpu().state();
                    state.io_registers = emu.bus().io_registers().clone();
                    state.ppu = emu.bus().ppu().state();
                }

                let cmd = if !running {
                    cmd_slot.recv().ok()
                } else {
                    cmd_slot.try_recv().ok()
                };

                match cmd {
                    Some(DebuggerCommand::Run) => {
                        emu_state_sig.try_send(EmuState::Running).ok();
                        running = true;
                    }
                    Some(DebuggerCommand::Stop) => {
                        emu_state_sig.try_send(EmuState::Idle).ok();
                        emu.reset();
                        running = false;
                    }
                    Some(DebuggerCommand::Pause) => {
                        emu_state_sig.try_send(EmuState::Idle).ok();
                        running = false;
                    }
                    Some(DebuggerCommand::SetBreakpoint(pc)) => {
                        breakpoints.insert(pc);
                    }
                    Some(DebuggerCommand::UnsetBreakpoint(pc)) => {
                        breakpoints.remove(&pc);
                    }
                    Some(DebuggerCommand::Step) => stepping = true,
                    None => (),
                }

                if running || stepping {
                    stepping = false;

                    if let Err(e) = emu.step() {
                        log::error!("emu error: {}", e);
                        emu_state_sig.try_send(EmuState::Error).ok();
                        break;
                    }
                }

                if breakpoints.contains(&emu.cpu().read_pc()) {
                    running = false;
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

            let new_pc = pc + instruction.len() as u16;
            disassembly.push((pc, Some(instruction)));
            pc = new_pc;
        }

        disassembly
    }
}
