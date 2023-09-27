mod sound_channel;

use sound_channel::{Channel1, Channel2, Channel3, Channel4};

use super::GbError;

const CH1_SWEEP_REG_ADDR: u16 = 0xFF10;
const CH1_WAVE_AND_TIMER_REG_ADDR: u16 = 0xFF11;
const CH1_ENVELOPE_REG_ADDR: u16 = 0xFF12;
const CH1_PERIOD_LOW_REG_ADDR: u16 = 0xFF13;
const CH1_PERIOD_HIGH_AND_CTRL_REG_ADDR: u16 = 0xFF14;

const CH2_WAVE_AND_TIMER_REG_ADDR: u16 = 0xFF16;
const CH2_ENVELOPE_REG_ADDR: u16 = 0xFF17;
const CH2_PERIOD_LOW_REG_ADDR: u16 = 0xFF18;
const CH2_PERIOD_HIGH_AND_CTRL_REG_ADDR: u16 = 0xFF19;

const CH3_DAC_ENABLE_REG_ADDR: u16 = 0xFF1A;

const CH4_ENVELOPE_REG_ADDR: u16 = 0xFF21;
const CH4_CONTROL_REG_ADDR: u16 = 0xFF23;

const VOLUME_CTRL_REG_ADDR: u16 = 0xFF24;
const OUTPUT_SELECT_REG_ADDR: u16 = 0xFF25;
const SOUND_ENABLE_REG_ADDR: u16 = 0xFF26;

pub struct APU {
    sound_enable: u8,
    sound_output_terminal_selection: u8,
    sound_channel_volume_control: u8,
    ch1: Channel1,
    ch2: Channel2,
    ch3: Channel3,
    ch4: Channel4,
}

impl APU {
    pub fn new() -> Self {
        Self {
            sound_enable: 0,
            sound_output_terminal_selection: 0,
            sound_channel_volume_control: 0,
            ch1: Default::default(),
            ch2: Default::default(),
            ch3: Default::default(),
            ch4: Default::default(),
        }
    }

    pub fn step(&mut self, _cpu_cycles: u8) -> Result<(), GbError> {
        // TODO
        Ok(())
    }

    pub fn read_reg(&self, addr: u16) -> Result<u8, GbError> {
        match addr {
            _ => Err(GbError::IllegalOp(format!(
                "Write to invalid APU reg {:#06X}",
                addr
            ))),
        }
    }

    pub fn write_reg(&mut self, addr: u16, value: u8) -> Result<(), GbError> {
        match addr {
            CH1_SWEEP_REG_ADDR => self.ch1.write_sweep(value),
            CH1_WAVE_AND_TIMER_REG_ADDR => self.ch1.write_wave_and_timer(value),
            CH1_ENVELOPE_REG_ADDR => self.ch1.write_envelope(value),
            CH1_PERIOD_LOW_REG_ADDR => self.ch1.write_period_low(value),
            CH1_PERIOD_HIGH_AND_CTRL_REG_ADDR => self.ch1.write_period_high_and_ctrl(value),
            CH2_WAVE_AND_TIMER_REG_ADDR => self.ch2.write_wave_and_timer(value),
            CH2_ENVELOPE_REG_ADDR => self.ch2.write_envelope(value),
            CH2_PERIOD_LOW_REG_ADDR => self.ch2.write_period_low(value),
            CH2_PERIOD_HIGH_AND_CTRL_REG_ADDR => self.ch2.write_period_high_and_ctrl(value),
            CH3_DAC_ENABLE_REG_ADDR => self.ch3.write_enable(value),
            CH4_ENVELOPE_REG_ADDR => self.ch4.write_envelope(value),
            CH4_CONTROL_REG_ADDR => self.ch4.write_control(value),
            VOLUME_CTRL_REG_ADDR => self.sound_channel_volume_control = value,
            OUTPUT_SELECT_REG_ADDR => self.sound_output_terminal_selection = value,
            SOUND_ENABLE_REG_ADDR => {
                if value & 0x7F != 0 {
                    return Err(GbError::IllegalOp(format!(
                        "attempting to write {:#04X} to sound enable register (NR52)",
                        value
                    )));
                } else {
                    self.sound_enable = value
                }
            }
            _ => {
                return Err(GbError::IllegalOp(format!(
                    "Write to invalid APU reg {:#06X}",
                    addr
                )))
            }
        }

        Ok(())
    }
}
