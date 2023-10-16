use std::{
    path::PathBuf,
    sync::{
        mpsc::{channel, Sender},
        Arc, RwLock, RwLockWriteGuard,
    },
    time::{Duration, SystemTime},
};

use flume::Receiver;

use crate::gbr::{bus::Bus, cpu::CPU, ppu::PPU, GbError};

use super::{
    bus::BusAccess,
    cpu::CpuState,
    interrupts::InterruptHandlerState,
    joypad::{Buttons, Directions, Joypad},
    mbc::MbcState,
    oam::ObjAttribute,
    ppu::PpuState,
};

#[derive(Default, Clone)]
pub struct GbState {
    pub cpu: CpuState,
    pub ir_handler: InterruptHandlerState,
    pub ppu: PpuState,
    pub mbc: MbcState,
    pub oam: Vec<ObjAttribute>,
    pub joypad: Joypad,
}

pub struct GameBoy {
    cpu: CPU,
    bus: Bus,
}

impl GameBoy {
    pub fn new(boot_rom_filename: Option<PathBuf>, cart_rom_filename: Option<PathBuf>) -> Self {
        Self {
            cpu: CPU::new(),
            bus: Bus::new(boot_rom_filename, cart_rom_filename),
        }
    }

    pub fn step(&mut self) -> Result<bool, GbError> {
        let cycles = self.cpu.step(&mut self.bus)?;

        self.bus.step(4 * cycles)
    }

    pub fn run_to_vblank(&mut self) -> Result<(), GbError> {
        loop {
            if self.step()? {
                break;
            }
        }

        Ok(())
    }

    pub fn handle_input(&mut self, input: InputType) {
        match input {
            InputType::Pressed(GenericInput::Button(button)) => {
                self.bus.joypad_mut().press_button(button)
            }
            InputType::Released(GenericInput::Button(button)) => {
                self.bus.joypad_mut().release_button(button)
            }
            InputType::Pressed(GenericInput::Direction(direction)) => {
                self.bus.joypad_mut().press_direction(direction)
            }
            InputType::Released(GenericInput::Direction(direction)) => {
                self.bus.joypad_mut().release_direction(direction)
            }
        }
    }

    pub fn update_settings(&mut self, settings: &EmuSettings) {
        todo!()
    }

    pub fn reset(&mut self) {
        self.cpu = CPU::new();
        self.bus.reset();
    }

    pub fn cpu(&self) -> &CPU {
        &self.cpu
    }

    pub fn bus(&self) -> &Bus {
        &self.bus
    }

    pub fn ppu(&self) -> &PPU {
        &self.bus.ppu()
    }

    pub fn collect_state(&self) -> GbState {
        GbState {
            cpu: self.cpu.state(),
            ir_handler: self.bus.ir_handler().state(),
            ppu: self.bus.ppu().state(),
            mbc: self.bus.mbc().state(),
            oam: self.bus.oam().state(),
            joypad: self.bus.joypad().clone(),
        }
    }
}

pub enum GbrEvent {
    Start,
    Stop,
    Pause,
    Step,
    Input(InputType),
    UpdateSettings(EmuSettings),
    Debug(DebugEvent),
}

#[derive(Clone)]
pub enum GenericInput {
    Button(Buttons),
    Direction(Directions),
}
pub enum InputType {
    Pressed(GenericInput),
    Released(GenericInput),
}

pub struct EmuSettings {
    skip_bootrom: bool,
    fps_limiter: bool,
}

pub enum DebugEvent {
    SetBreakpoint(u16),
    ClearBreakpoint(u16),
    DumpVram,
}

pub trait Debugger {
    fn handle_event(&mut self, gb: &RwLockWriteGuard<GameBoy>, ev: &DebugEvent);

    fn send_state(&mut self, gb: &RwLockWriteGuard<GameBoy>);

    fn should_break(&self, gb: &RwLockWriteGuard<GameBoy>) -> bool;
}

pub enum EmuState {
    Idle,
    Running,
    Error,
}

pub fn start_gb_thread<DebuggerType: Debugger + Sync + Send + 'static>(
    gb: Arc<RwLock<GameBoy>>,
    mut debugger: DebuggerType,
) -> (Sender<GbrEvent>, Receiver<EmuState>) {
    let (ev_sender, ev_listener) = channel();
    let (emu_state_sig, emu_state_slot) = flume::bounded(1);

    std::thread::spawn(move || {
        let mut running = false;
        let mut stepping = false;
        let mut gb = gb.write().unwrap();

        let frame_time = Duration::from_secs_f64(1.0 / 59.7);
        let mut now = SystemTime::now();
        loop {
            if let Ok(ev) = ev_listener.try_recv() {
                match ev {
                    GbrEvent::Start => {
                        running = true;
                        emu_state_sig.send(EmuState::Running).ok();
                    }
                    GbrEvent::Pause => {
                        running = false;
                        emu_state_sig.send(EmuState::Idle).ok();
                    }
                    GbrEvent::Step => stepping = true,
                    GbrEvent::Stop => {
                        running = false;
                        gb.reset();

                        emu_state_sig.send(EmuState::Idle).ok();
                    }
                    GbrEvent::Input(input) => gb.handle_input(input),
                    GbrEvent::UpdateSettings(settings) => gb.update_settings(&settings),
                    GbrEvent::Debug(ev) => debugger.handle_event(&gb, &ev),
                }
            }

            if running {
                gb.run_to_vblank().unwrap();

                let elapsed = SystemTime::now().duration_since(now).unwrap();
                now = SystemTime::now();

                if elapsed < frame_time {
                    std::thread::sleep(frame_time - elapsed);
                }
            } else if stepping {
                gb.step().unwrap();
            }

            if debugger.should_break(&gb) {
                running = false;
                emu_state_sig.send(EmuState::Idle).ok();
            }

            debugger.send_state(&gb);
        }
    });

    (ev_sender, emu_state_slot)
}
