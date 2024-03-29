use super::CH3_WAVE_PATTERN_RAM_SIZE;

#[derive(Default)]
enum SweepDirection {
    #[default]
    Increase,
    Decrease,
}

#[derive(Default, Clone, Copy)]
enum DutyCycle {
    #[default]
    OneToEight,
    OneToFour,
    Half,
    ThreeToFour,
}

impl From<u8> for DutyCycle {
    fn from(value: u8) -> Self {
        match value & 0b00000011 {
            0 => DutyCycle::OneToEight,
            1 => DutyCycle::OneToFour,
            2 => DutyCycle::Half,
            3 => DutyCycle::ThreeToFour,
            _ => panic!("Cannot convert {} to DutyCycle", value),
        }
    }
}

impl From<DutyCycle> for u8 {
    fn from(value: DutyCycle) -> Self {
        match value {
            DutyCycle::OneToEight => 0,
            DutyCycle::OneToFour => 1,
            DutyCycle::Half => 2,
            DutyCycle::ThreeToFour => 3,
        }
    }
}

#[derive(Default)]
struct ChannelControl {
    trigger: bool,
    sound_length_enable: bool,
}

impl ChannelControl {
    pub fn write(&mut self, value: u8) {
        self.trigger = value & 0b10000000 != 0;
        self.sound_length_enable = value & 0b01000000 != 0;
    }
}

#[derive(Default)]
pub struct Sweep {
    pace: u8,
    direction: SweepDirection,
    slope_ctrl: u8,
}

impl Sweep {
    fn write(&mut self, value: u8) {
        self.slope_ctrl = value & 0b00000111;
        self.direction = if value & 0b00001000 != 0 {
            SweepDirection::Decrease
        } else {
            SweepDirection::Increase
        };
        self.pace = (value & 0b01110000) >> 4;
    }
}

#[derive(Default)]
pub struct Envelope {
    volume: u8,
    direction: SweepDirection,
    sweep_pace: u8,
}

impl Envelope {
    fn write(&mut self, value: u8) {
        self.sweep_pace = value & 0b00000111;
        self.direction = if value & 0b00001000 == 0 {
            SweepDirection::Decrease
        } else {
            SweepDirection::Increase
        };
        self.volume = value & 0b11110000;
    }
}

#[derive(Default)]
pub struct Pulse {
    duty_cycle: DutyCycle,
    length_timer: u8,
    envelope: Envelope,
    period: u16,
}

impl Pulse {
    pub fn write_wave_and_timer(&mut self, value: u8) {
        self.duty_cycle = (value & 0b00000011).into();
        self.length_timer = value & 0b11111100;
    }

    pub fn read_duty_cycle(&self) -> u8 {
        (self.duty_cycle as u8) << 5
    }

    pub fn write_period_low(&mut self, period_low: u8) {
        self.period = self.period & 0xFF00 + period_low as u16;
    }

    pub fn write_period_high(&mut self, period_high: u8) {
        let period_high = (period_high & 0b00000111) as u16;
        self.period = period_high << 8 + self.period & 0x00FF;
    }

    pub fn write_envelope(&mut self, value: u8) {
        self.envelope.write(value);
    }
}

#[derive(Default)]
pub enum OutputLevel {
    #[default]
    Mute,
    MaxVolume,
    HalfVolume,
    QuarterVolume,
}

impl From<u8> for OutputLevel {
    fn from(value: u8) -> Self {
        match value & 0b00000011 {
            0 => OutputLevel::Mute,
            1 => OutputLevel::MaxVolume,
            2 => OutputLevel::HalfVolume,
            3 => OutputLevel::QuarterVolume,
            _ => panic!("Cannot convert {} to OutputLevel", value),
        }
    }
}

impl From<OutputLevel> for u8 {
    fn from(value: OutputLevel) -> Self {
        match value {
            OutputLevel::Mute => 0,
            OutputLevel::MaxVolume => 1,
            OutputLevel::HalfVolume => 2,
            OutputLevel::QuarterVolume => 3,
        }
    }
}

#[derive(Default)]
pub enum LfsrWidth {
    #[default]
    FifteenBits,
    SevenBits,
}

#[derive(Default)]
pub struct Channel1 {
    sweep: Sweep,
    pulse: Pulse,
    ctrl: ChannelControl,
}

impl Channel1 {
    pub fn write_sweep(&mut self, value: u8) {
        self.sweep.write(value);
    }

    pub fn write_wave_and_timer(&mut self, value: u8) {
        self.pulse.write_wave_and_timer(value);
    }

    pub fn write_envelope(&mut self, value: u8) {
        self.pulse.write_envelope(value);
    }

    pub fn write_period_low(&mut self, value: u8) {
        self.pulse.write_period_low(value);
    }

    pub fn write_period_high_and_ctrl(&mut self, value: u8) {
        self.pulse.write_period_high(value);
        self.ctrl.write(value);
    }

    pub fn read_duty_cycle(&self) -> u8 {
        self.pulse.read_duty_cycle()
    }
}

#[derive(Default)]
pub struct Channel2 {
    pulse: Pulse,
    ctrl: ChannelControl,
}

impl Channel2 {
    pub fn write_wave_and_timer(&mut self, value: u8) {
        self.pulse.write_wave_and_timer(value);
    }

    pub fn write_envelope(&mut self, value: u8) {
        self.pulse.write_envelope(value);
    }

    pub fn write_period_low(&mut self, value: u8) {
        self.pulse.write_period_low(value);
    }

    pub fn write_period_high_and_ctrl(&mut self, value: u8) {
        self.pulse.write_period_high(value);
        self.ctrl.write(value);
    }

    pub fn read_duty_cycle(&self) -> u8 {
        self.pulse.read_duty_cycle()
    }
}

#[derive(Default)]
pub struct Channel3 {
    enable: bool,
    length_timer: u8,
    output_level: OutputLevel,
    period: u16,
    ctrl: ChannelControl,
    pub wave_pattern: [u8; CH3_WAVE_PATTERN_RAM_SIZE],
}

impl Channel3 {
    pub fn write_enable(&mut self, value: u8) {
        self.enable = value & 0b10000000 != 0;
    }

    pub fn write_length_timer(&mut self, value: u8) {
        self.length_timer = value;
    }

    pub fn write_output_level(&mut self, value: u8) {
        self.output_level = value.into();
    }

    pub fn write_period_low(&mut self, value: u8) {
        self.period = self.period & 0xFF00 + value as u16;
    }

    pub fn write_period_high_and_ctrl(&mut self, value: u8) {
        let period_high = (value & 0b00000111) as u16;
        self.period = period_high << 8 + self.period & 0x00FF;

        self.ctrl.write(value);
    }
}

#[derive(Default)]
pub struct Channel4 {
    length_timer: u8,
    envelope: Envelope,
    clock_shift: u8,
    clock_divider: u8,
    lfsr_width: LfsrWidth,
    control: ChannelControl,
}

impl Channel4 {
    pub fn write_envelope(&mut self, value: u8) {
        self.envelope.write(value);
    }

    pub fn write_control(&mut self, value: u8) {
        self.control.write(value);
    }

    pub fn write_length_timer(&mut self, value: u8) {
        self.length_timer = value;
    }

    pub fn write_freq_and_randomness(&mut self, value: u8) {
        self.clock_divider = value & 0b00000111;
        self.lfsr_width = if value & 0b00001000 != 0 {
            LfsrWidth::SevenBits
        } else {
            LfsrWidth::FifteenBits
        };

        self.clock_shift = (value & 0b11110000) >> 4;
    }
}
