use super::{
    interrupts::{InterruptHandler, InterruptType},
    GbError,
};

const CPU_FREQ: u64 = 4_194_304; // Hz
const DIVIDER_FREQ: u64 = 16_384; // Hz
const DIVIDER_COUNTS_PER_CYCLE: u64 = CPU_FREQ / DIVIDER_FREQ;

const DIVIDER_REG_ADDR: u16 = 0xFF04;
const COUNTER_REG_ADDR: u16 = 0xFF05;
const MODULO_REG_ADDR: u16 = 0xFF06;
const CONTROL_REG_ADDR: u16 = 0xFF07;

#[derive(Default, Copy, Clone)]
pub enum ClockSelect {
    #[default]
    OneTo1024,
    OneTo16,
    OneTo64,
    OneTo256,
}

impl ClockSelect {
    fn counts_per_cycle(&self) -> u64 {
        match self {
            Self::OneTo1024 => 1024,
            Self::OneTo16 => 16,
            Self::OneTo64 => 64,
            Self::OneTo256 => 256,
        }
    }
}

impl From<u8> for ClockSelect {
    fn from(value: u8) -> Self {
        match value & 0b00000011 {
            0 => ClockSelect::OneTo1024,
            1 => ClockSelect::OneTo16,
            2 => ClockSelect::OneTo64,
            3 => ClockSelect::OneTo256,
            _ => panic!("Cannot convert {} to ClockSelect", value),
        }
    }
}

impl From<ClockSelect> for u8 {
    fn from(value: ClockSelect) -> Self {
        match value {
            ClockSelect::OneTo1024 => 0,
            ClockSelect::OneTo16 => 1,
            ClockSelect::OneTo64 => 2,
            ClockSelect::OneTo256 => 3,
        }
    }
}

#[derive(Default)]
pub struct Timer {
    divider: u8,
    counter: u8,
    modulo: u8,
    clock_select: ClockSelect,
    enable: bool,

    cycles_elapsed_divider: u64,
    cycles_elapsed_counter: u64,
}

impl Timer {
    pub fn step(&mut self, cpu_cycles: u8, ir_handler: &mut InterruptHandler) {
        self.update_divider(cpu_cycles);
        let was_reset = self.update_counter(cpu_cycles);

        if was_reset {
            ir_handler.set(InterruptType::Timer);
        }
    }

    pub fn write_reg(&mut self, addr: u16, value: u8) -> Result<(), GbError> {
        match addr {
            DIVIDER_REG_ADDR => self.divider = 0,
            COUNTER_REG_ADDR => self.counter = value,
            MODULO_REG_ADDR => self.modulo = value,
            CONTROL_REG_ADDR => {
                self.enable = (value & 0b00000100) != 0;
                self.clock_select = (value & 0b00000011).into();
            }
            _ => {
                return Err(GbError::IllegalOp(format!(
                    "Write to timer reg {:#06X}",
                    addr
                )))
            }
        }

        Ok(())
    }

    pub fn read_reg(&self, addr: u16) -> Result<u8, GbError> {
        match addr {
            DIVIDER_REG_ADDR => Ok(self.divider),
            COUNTER_REG_ADDR => Ok(self.counter),
            MODULO_REG_ADDR => Ok(self.modulo),
            CONTROL_REG_ADDR => Ok((self.enable as u8) << 3 | self.clock_select as u8),
            _ => Err(GbError::IllegalOp(format!(
                "Read from timer reg {:#06X}",
                addr
            ))),
        }
    }

    fn update_divider(&mut self, cpu_cycles: u8) {
        self.cycles_elapsed_divider += cpu_cycles as u64;
        if self.cycles_elapsed_divider >= DIVIDER_COUNTS_PER_CYCLE {
            if self.divider == 0xFF {
                self.divider = 0;
            } else {
                self.divider += 1;
            }
            self.cycles_elapsed_divider -= DIVIDER_COUNTS_PER_CYCLE;
        }
    }

    fn update_counter(&mut self, cpu_cycles: u8) -> bool {
        let mut was_reset = false;
        let counts_per_cycles = self.clock_select.counts_per_cycle();
        self.cycles_elapsed_counter += cpu_cycles as u64;
        if self.cycles_elapsed_counter >= counts_per_cycles {
            if self.counter == self.modulo {
                self.counter = 0;
                was_reset = true;
            } else {
                self.counter += 1;
            }
            self.cycles_elapsed_counter -= counts_per_cycles;
        }

        was_reset
    }
}
