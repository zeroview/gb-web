use super::*;

pub trait Channel {
    /// Reads value from channel register at global address
    /// Note: Unused bits, length counters and frequencies are set to 1s when read back,
    /// as defined here: https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware#Register_Reading
    fn read_register(&self, address: u16) -> u8;

    /// Writes value to channel register at global address
    fn write_register(&mut self, address: u16, value: u8);

    /// Returns the next sample
    fn get_sample(&self) -> f32;

    /// Triggers the channel
    /// (https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware#Trigger_Event)
    fn trigger(&mut self);

    /// Converts internal digital sample between 0 and 15
    /// to an analog value between -1.0 and 1.0
    fn convert_sample(&self, digital: u8) -> f32 {
        ((digital as f32) / 7.5) - 1.0
    }
}

#[derive(Deserialize, Serialize)]
pub struct SquareChannel {
    // State variables
    pub channel_on: bool,
    pub dac_on: bool,
    pub period_div: u16,
    pub duty_cycle_pointer: u8,
    pub length_timer: u8,
    pub volume: u8,
    pub period: u16,
    pub sweep_timer: u8,
    pub envelope_timer: u8,
    // Register variables
    pub sweep_pace: u8,
    pub sweep_increase: bool,
    pub sweep_step: u8,
    pub duty_cycle_index: u8,
    pub initial_length_timer: u8,
    pub length_timer_enabled: bool,
    pub initial_period: u16,
    pub initial_volume: u8,
    pub envelope_increase: bool,
    pub envelope_pace: u8,
}

impl SquareChannel {
    pub fn new() -> Self {
        Self {
            channel_on: false,
            dac_on: false,
            period_div: 0,
            duty_cycle_pointer: 0,
            length_timer: 0,
            volume: 0,
            period: 0,
            sweep_timer: 1,
            envelope_timer: 1,

            sweep_pace: 0,
            sweep_increase: true,
            sweep_step: 0,
            duty_cycle_index: 0,
            initial_length_timer: 0,
            length_timer_enabled: false,
            initial_period: 0,
            initial_volume: 0,
            envelope_increase: false,
            envelope_pace: 0,
        }
    }

    pub fn update_length_timer(&mut self) {
        if self.length_timer_enabled {
            if self.length_timer == 0 {
                self.channel_on = false;
            } else {
                self.length_timer -= 1;
            }
        }
    }

    pub fn update_sweep(&mut self) {
        if self.sweep_pace == 0 {
            return;
        }
        if self.sweep_timer < self.sweep_pace {
            self.sweep_timer += 1;
        } else {
            self.sweep_timer = 1;
            let period_change = self.period / 2u16.pow(self.sweep_step.into());
            if self.sweep_increase {
                self.period += period_change;
            } else if self.volume > 0 {
                self.period -= period_change;
            }
            if self.period > 0x7FF {
                self.period = 0x7FF;
                self.channel_on = false;
            }
        }
    }

    pub fn update_envelope(&mut self) {
        if self.envelope_pace == 0 {
            return;
        }
        if self.envelope_timer < self.envelope_pace {
            self.envelope_timer += 1;
        } else {
            self.envelope_timer = 1;
            if self.envelope_increase {
                if self.volume < 15 {
                    self.volume += 1;
                }
            } else if self.volume > 0 {
                self.volume -= 1;
            }
        }
    }

    pub fn update_period(&mut self) {
        if self.period_div == 0x7FF {
            if self.duty_cycle_pointer == 7 {
                self.duty_cycle_pointer = 0;
            } else {
                self.duty_cycle_pointer += 1;
            }
            self.period_div = self.period;
        } else {
            self.period_div += 1;
        }
    }

    fn get_duty_cycle_val(&self, index: u8) -> u8 {
        let duty_cycle = match self.duty_cycle_index {
            // 12.5 %
            0 => [1, 1, 1, 1, 1, 1, 1, 0],
            // 25 %
            1 => [0, 1, 1, 1, 1, 1, 1, 0],
            // 50 %
            2 => [0, 1, 1, 1, 1, 0, 0, 0],
            // 75 %
            3 => [1, 0, 0, 0, 0, 0, 0, 1],
            _ => unreachable!(),
        };
        duty_cycle[index as usize]
    }
}

impl Channel for SquareChannel {
    fn read_register(&self, address: u16) -> u8 {
        match address {
            // NR10 (Sweep)
            0xFF10 => {
                (self.sweep_pace << 4)
                    | ((!self.sweep_increase as u8) << 3)
                    | self.sweep_step
                    | 0x80
            }
            // NR11 / NR21 (Length and duty cycle)
            0xFF11 | 0xFF16 => (self.duty_cycle_index << 6) | 0x3F,
            // NR12 / NR22 (Volume and envelope)
            0xFF12 | 0xFF17 => {
                (self.initial_volume << 4)
                    | ((self.envelope_increase as u8) << 3)
                    | self.envelope_pace
            }
            // NR13 / NR23 (Period low bits)
            0xFF13 | 0xFF18 => 0xFF,
            // NR14 / NR14 (Control and period high bits)
            0xFF14 | 0xFF19 => ((self.length_timer_enabled as u8) << 6) | 0xBF,
            _ => unreachable!(),
        }
    }

    fn write_register(&mut self, address: u16, value: u8) {
        match address {
            // NR10 (Sweep)
            0xFF10 => {
                self.sweep_pace = value >> 4;
                // 0 == increase
                self.sweep_increase = value & 0b1000 == 0;
                self.sweep_step = value & 0b0111;
            }
            // NR11 / NR21 (Length and duty cycle)
            0xFF11 | 0xFF16 => {
                self.duty_cycle_index = value >> 6;
                self.initial_length_timer = 64 - (value & 0b11_1111);
            }
            // NR12 / NR22 (Volume and envelope)
            0xFF12 | 0xFF17 => {
                self.initial_volume = value >> 4;
                self.envelope_increase = value & 0b1000 > 0;
                self.envelope_pace = value & 0b0111;
                // Channel volume unit is controlled by these control bits
                // If envelope is set to decrease volume from 0, the DAC is off
                self.dac_on = self.initial_volume > 0 || self.envelope_increase;
                if !self.dac_on {
                    self.channel_on = false;
                }
            }
            // NR13 / NR23 (Period low bits)
            0xFF13 | 0xFF18 => self.initial_period = (self.initial_period & 0xFF00) | value as u16,
            // NR14 / NR14 (Control and period high bits)
            0xFF14 | 0xFF19 => {
                self.length_timer_enabled = value & 0b0100_0000 > 0;
                self.initial_period =
                    (self.initial_period & 0xFF) | (((value & 0b111) as u16) << 8);
                if value & 0b1000_0000 > 0 {
                    self.trigger();
                }
            }
            _ => unreachable!(),
        }
    }

    fn trigger(&mut self) {
        self.channel_on = self.dac_on;
        self.period = self.initial_period;
        self.period_div = self.period;
        self.volume = self.initial_volume;
        self.envelope_timer = 1;
        self.sweep_timer = 1;
        self.length_timer = 64;
    }

    fn get_sample(&self) -> f32 {
        if self.dac_on {
            let volume = if self.channel_on {
                self.volume * self.get_duty_cycle_val(self.duty_cycle_pointer)
            } else {
                0
            };
            self.convert_sample(volume)
        } else {
            0.0
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct WaveChannel {
    // State variables
    pub dac_on: bool,
    pub channel_on: bool,
    pub period_div: u16,
    pub wave_pointer: u8,
    pub length_timer: u16,
    pub output_level: u8,
    pub period: u16,
    // Register variables
    pub initial_length_timer: u16,
    pub length_timer_enabled: bool,
    pub initial_period: u16,
    pub wave_ram: [u8; 0x10],
}

impl WaveChannel {
    pub fn new() -> Self {
        Self {
            dac_on: false,
            channel_on: false,
            period_div: 0,
            wave_pointer: 0,
            length_timer: 0,
            output_level: 0,
            period: 0,

            initial_length_timer: 0,
            length_timer_enabled: false,
            initial_period: 0,
            wave_ram: [0; 0x10],
        }
    }

    pub fn update_length_timer(&mut self) {
        if self.length_timer_enabled {
            if self.length_timer == 0 {
                self.channel_on = false;
            } else {
                self.length_timer -= 1;
            }
        }
    }

    pub fn update_period(&mut self) {
        if self.period_div == 0x7FF {
            if self.wave_pointer == 31 {
                self.wave_pointer = 0;
            } else {
                self.wave_pointer += 1;
            }
            self.period_div = self.period;
        } else {
            self.period_div += 1;
        }
    }
}

impl Channel for WaveChannel {
    fn read_register(&self, address: u16) -> u8 {
        // Unused bits, length counters and frequencies are set to 1s when read back,
        // as defined here: https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware#Register_Reading
        match address {
            // NR30 (DAC)
            0xFF1A => (self.dac_on as u8) << 7 | 0x7F,
            // NR31 (Length timer)
            0xFF1B => 0xFF,
            // NR32 (Output level)
            0xFF1C => self.output_level << 5 | 0x9F,
            // NR33 (Period low bits)
            0xFF1D => 0xFF,
            // NR34 (Control and period high bits)
            0xFF1E => ((self.length_timer_enabled as u8) << 6) | 0xBF,
            // Wave RAM
            0xFF30..=0xFF3F => self.wave_ram[(address - 0xFF30) as usize],
            _ => unreachable!(),
        }
    }

    fn write_register(&mut self, address: u16, value: u8) {
        match address {
            // NR30 (DAC)
            0xFF1A => {
                self.dac_on = value & 0b1000_0000 > 0;
                if !self.dac_on {
                    self.channel_on = false;
                }
            }
            // NR31 (Length timer)
            0xFF1B => self.initial_length_timer = 256 - (value as u16),
            // NR32 (Output level)
            0xFF1C => self.output_level = (value >> 5) & 0b11,
            // NR33 (Period low bits)
            0xFF1D => self.initial_period = (self.initial_period & 0xFF00) | value as u16,
            // NR34 (Control and period high bits)
            0xFF1E => {
                self.length_timer_enabled = value & 0b0100_0000 > 0;
                self.initial_period =
                    (self.initial_period & 0xFF) | (((value & 0b111) as u16) << 8);
                if value & 0b1000_0000 > 0 {
                    self.trigger();
                }
            }
            // Wave RAM
            0xFF30..=0xFF3F => self.wave_ram[(address - 0xFF30) as usize] = value,
            _ => unreachable!(),
        }
    }

    fn trigger(&mut self) {
        self.channel_on = self.dac_on;
        self.period = self.initial_period;
        self.period_div = self.period;
        self.length_timer = 256;
        self.wave_pointer = 0;
    }

    fn get_sample(&self) -> f32 {
        if self.dac_on {
            let volume = if self.channel_on {
                let byte = self.wave_ram[(self.wave_pointer / 2) as usize];
                let nibble = if self.wave_pointer & 2 == 0 {
                    byte >> 4
                } else {
                    byte & 0xF
                };
                match self.output_level {
                    0 => 0,
                    1 => nibble,
                    2 => nibble >> 1,
                    3 => nibble >> 2,
                    _ => unreachable!(),
                }
            } else {
                0
            };
            self.convert_sample(volume)
        } else {
            0.0
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct NoiseChannel {
    // State variables
    pub dac_on: bool,
    pub channel_on: bool,
    pub duty_cycle_pointer: u8,
    pub length_timer: u8,
    pub volume: u8,
    pub envelope_timer: u8,
    pub lfsr: u16,
    pub lfsr_bit: bool,
    pub lfsr_timer: u32,
    pub lfsr_pace: u32,
    // Register variables
    pub clock_shift: u8,
    pub short_lfsr: bool,
    pub clock_divider: u8,
    pub initial_length_timer: u8,
    pub length_timer_enabled: bool,
    pub initial_volume: u8,
    pub envelope_increase: bool,
    pub envelope_pace: u8,
}

impl NoiseChannel {
    pub fn new() -> Self {
        Self {
            channel_on: false,
            dac_on: false,
            duty_cycle_pointer: 0,
            length_timer: 0,
            volume: 0,
            envelope_timer: 1,
            lfsr: 0,
            lfsr_bit: false,
            lfsr_timer: 1,
            lfsr_pace: 1,

            clock_shift: 0,
            short_lfsr: false,
            clock_divider: 0,
            initial_length_timer: 0,
            length_timer_enabled: false,
            initial_volume: 0,
            envelope_increase: false,
            envelope_pace: 0,
        }
    }

    pub fn update_length_timer(&mut self) {
        if self.length_timer_enabled {
            if self.length_timer == 0 {
                self.channel_on = false;
            } else {
                self.length_timer -= 1;
            }
        }
    }

    pub fn update_envelope(&mut self) {
        if self.envelope_pace == 0 {
            return;
        }
        if self.envelope_timer < self.envelope_pace {
            self.envelope_timer += 1;
        } else {
            self.envelope_timer = 1;
            if self.envelope_increase {
                if self.volume < 15 {
                    self.volume += 1;
                }
            } else if self.volume > 0 {
                self.volume -= 1;
            }
        }
    }

    pub fn update_lfsr(&mut self) {
        if self.lfsr_timer < self.lfsr_pace {
            self.lfsr_timer += 1;
        } else {
            self.lfsr_timer = 1;
            let xnor = self.lfsr & 0b1 == (self.lfsr & 0b10) >> 1;
            if xnor {
                self.lfsr |= (xnor as u16) << 15;
                if self.short_lfsr {
                    self.lfsr |= (xnor as u16) << 7;
                }
            }
            self.lfsr >>= 1;
            self.lfsr_bit = self.lfsr & 0b1 > 0;
        }
    }
}

impl Channel for NoiseChannel {
    fn read_register(&self, address: u16) -> u8 {
        // Unused bits, length counters and frequencies are set to 1s when read back,
        // as defined here: https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware#Register_Reading
        match address {
            // NR41 (Length timer)
            0xFF20 => 0xFF,
            // NR42 (Volume and envelope)
            0xFF21 => {
                (self.initial_volume << 4)
                    | ((self.envelope_increase as u8) << 3)
                    | self.envelope_pace
            }
            // NR43 (Frequency and randomness)
            0xFF22 => (self.clock_shift << 4) | ((self.short_lfsr as u8) << 3) | self.clock_divider,
            // NR44 (Control)
            0xFF23 => (self.length_timer_enabled as u8) << 6 | 0xBF,
            _ => unreachable!(),
        }
    }

    fn write_register(&mut self, address: u16, value: u8) {
        match address {
            // NR41 (Length timer)
            0xFF20 => self.initial_length_timer = 64 - (value & 0b0011_1111),
            // NR42 (Volume and envelope)
            0xFF21 => {
                self.initial_volume = value >> 4;
                self.envelope_increase = value & 0b1000 > 0;
                self.envelope_pace = value & 0b0111;
                // Channel volume unit is controlled by these control bits
                // If envelope is set to decrease volume from 0, the DAC is off
                self.dac_on = self.initial_volume > 0 || self.envelope_increase;
                if !self.dac_on {
                    self.channel_on = false;
                }
            }
            // NR43 (Frequency and randomness)
            0xFF22 => {
                self.clock_shift = value >> 4;
                self.short_lfsr = value & 0b1000 > 0;
                self.clock_divider = value & 0b0111;
                // Divider value 0 is treated as 0.5
                self.lfsr_pace = if self.clock_divider == 0 {
                    2u32.pow(self.clock_shift as u32) / 2
                } else {
                    (self.clock_divider as u32) * 2u32.pow(self.clock_shift as u32)
                };
            }
            // NR44 (Control)
            0xFF23 => {
                self.length_timer_enabled = value & 0b0100_0000 > 0;
                if value & 0b1000_0000 > 0 {
                    self.trigger();
                }
            }
            _ => unreachable!(),
        }
    }

    fn trigger(&mut self) {
        self.channel_on = self.dac_on;
        self.lfsr = 0;
        self.volume = self.initial_volume;
        self.envelope_timer = 1;
        self.length_timer = 64;
    }

    fn get_sample(&self) -> f32 {
        if self.dac_on {
            let volume = if self.channel_on && self.lfsr_bit {
                self.volume
            } else {
                0
            };
            self.convert_sample(volume)
        } else {
            0.0
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq)]
pub struct PanRegister(u8);

bitflags! {
    impl PanRegister: u8 {
        const CH4_LEFT  = 0b1000_0000;
        const CH3_LEFT  = 0b0100_0000;
        const CH2_LEFT  = 0b0010_0000;
        const CH1_LEFT  = 0b0001_0000;
        const CH4_RIGHT = 0b0000_1000;
        const CH3_RIGHT = 0b0000_0100;
        const CH2_RIGHT = 0b0000_0010;
        const CH1_RIGHT = 0b0000_0001;
    }
}

use ringbuf::{
    HeapRb, SharedRb,
    storage::Heap,
    traits::{Producer, Split},
    wrap::caching::Caching,
};
use std::sync::Arc;

pub type AudioBufferProducer = Caching<Arc<SharedRb<Heap<f32>>>, true, false>;
pub type AudioBufferConsumer = Caching<Arc<SharedRb<Heap<f32>>>, false, true>;

/// Audio processing unit
#[allow(clippy::upper_case_acronyms)]
#[derive(Deserialize, Serialize)]
pub struct APU {
    #[serde(skip)]
    buffer_producer: Option<AudioBufferProducer>,
    #[serde(skip)]
    pub sample_delay: u32,
    #[serde(skip)]
    pub hpf_capacitor_charge_factor: f32,
    #[serde(skip)]
    pub channels: usize,

    pub on: bool,
    pub sample_delay_counter: u32,
    pub period_delay_counter: u8,
    pub div_apu: u8,
    pub last_div_bit: bool,
    pub pan_options: PanRegister,
    pub left_volume: u8,
    pub right_volume: u8,
    pub left_hpf_capacitor: f32,
    pub right_hpf_capacitor: f32,

    pub square_channel_1: SquareChannel,
    pub square_channel_2: SquareChannel,
    pub wave_channel: WaveChannel,
    pub noise_channel: NoiseChannel,
}

impl APU {
    pub fn new() -> Self {
        Self {
            buffer_producer: None,
            sample_delay: 0,
            channels: 0,
            hpf_capacitor_charge_factor: 0.0,

            on: true,
            sample_delay_counter: 0,
            period_delay_counter: 0,
            div_apu: 0,
            last_div_bit: false,
            pan_options: PanRegister::from_bits_truncate(0),
            left_volume: 1,
            right_volume: 1,
            left_hpf_capacitor: 0.0,
            right_hpf_capacitor: 0.0,

            square_channel_1: SquareChannel::new(),
            square_channel_2: SquareChannel::new(),
            wave_channel: WaveChannel::new(),
            noise_channel: NoiseChannel::new(),
        }
    }

    const CLOCK_SPEED: u32 = 4194304;
    const CAPACITOR_CHARGE_FACTOR: f64 = 0.999958;

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_delay = Self::CLOCK_SPEED / sample_rate;
        self.hpf_capacitor_charge_factor = Self::CAPACITOR_CHARGE_FACTOR
            .powf((Self::CLOCK_SPEED as f64) / (sample_rate as f64))
            as f32
    }

    pub fn init_buffer(&mut self, sample_capacity: usize, channels: usize) -> AudioBufferConsumer {
        let ring = HeapRb::<f32>::new(sample_capacity);
        let (producer, consumer) = ring.split();

        self.channels = channels;
        self.buffer_producer = Some(producer);
        consumer
    }

    pub fn cycle(&mut self, timer_div: u16) {
        // Increment DIV-APU when DIV register bit 4 (actual divider bit 12)
        // goes from 1 to 0
        let div_bit = timer_div & 0b1_0000_0000_0000 > 0;
        if self.last_div_bit && !div_bit {
            self.div_apu = self.div_apu.wrapping_add(1);
            // Update length timers at 256hz (every 2 ticks)
            if self.div_apu.is_multiple_of(2) {
                self.square_channel_1.update_length_timer();
                self.square_channel_2.update_length_timer();
                self.wave_channel.update_length_timer();
                self.noise_channel.update_length_timer();
                // Update CH1 period sweep at 128hz (every 4 ticks)
                if self.div_apu.is_multiple_of(4) {
                    self.square_channel_1.update_sweep();
                    // Update envelopes at 64hz (every 8 ticks)
                    if self.div_apu.is_multiple_of(8) {
                        self.square_channel_1.update_envelope();
                        self.square_channel_2.update_envelope();
                        self.noise_channel.update_envelope();
                    }
                }
            }
        }
        self.last_div_bit = div_bit;

        self.period_delay_counter = self.period_delay_counter.wrapping_add(1);
        // Update wave channel period every 2 T-cycles
        if self.period_delay_counter.is_multiple_of(2) {
            self.wave_channel.update_period();
            // Update square channel period every 4 T-cycles
            if self.period_delay_counter.is_multiple_of(4) {
                self.square_channel_1.update_period();
                self.square_channel_2.update_period();
                // Update noise channel frequency every 16 T-cycles
                if self.period_delay_counter.is_multiple_of(16) {
                    self.noise_channel.update_lfsr();
                }
            }
        }

        // Only calculate next sample when needed
        if self.sample_delay_counter < self.sample_delay {
            self.sample_delay_counter += 1;
            return;
        }

        if let Some(buffer) = &mut self.buffer_producer {
            self.sample_delay_counter = 0;

            // If APU or all DACs are turned off, just push silence to the buffer
            if !self.on
                || (!self.square_channel_1.dac_on
                    && !self.square_channel_2.dac_on
                    && !self.wave_channel.dac_on
                    && !self.noise_channel.dac_on)
            {
                for _ in 0..self.channels {
                    let _ = buffer.try_push(0.0);
                }
                return;
            }

            // Get samples from all channels
            let ch1 = self.square_channel_1.get_sample();
            let ch2 = self.square_channel_2.get_sample();
            let ch3 = self.wave_channel.get_sample();
            let ch4 = self.noise_channel.get_sample();

            // Combine left and right channels
            let mut left_sample = 0f32;
            if self.pan_options.intersects(PanRegister::CH1_LEFT) {
                left_sample += ch1;
            }
            if self.pan_options.intersects(PanRegister::CH2_LEFT) {
                left_sample += ch2;
            }
            if self.pan_options.intersects(PanRegister::CH3_LEFT) {
                left_sample += ch3;
            }
            if self.pan_options.intersects(PanRegister::CH4_LEFT) {
                left_sample += ch4;
            }
            let mut right_sample = 0f32;
            if self.pan_options.intersects(PanRegister::CH1_RIGHT) {
                right_sample += ch1;
            }
            if self.pan_options.intersects(PanRegister::CH2_RIGHT) {
                right_sample += ch2;
            }
            if self.pan_options.intersects(PanRegister::CH3_RIGHT) {
                right_sample += ch3;
            }
            if self.pan_options.intersects(PanRegister::CH4_RIGHT) {
                right_sample += ch4;
            }

            // Apply volume
            left_sample *= self.left_volume as f32;
            right_sample *= self.right_volume as f32;

            // Apply a high pass filter by simulating a capacitor
            let left_output = left_sample - self.left_hpf_capacitor;
            self.left_hpf_capacitor =
                (left_sample - left_output) * self.hpf_capacitor_charge_factor;
            let right_output = right_sample - self.right_hpf_capacitor;
            self.right_hpf_capacitor =
                (right_sample - right_output) * self.hpf_capacitor_charge_factor;

            // Scale final mixed sample in between -1.0 and 1.0
            // Maximum analog value can be:
            // +1.0 (max channel output voltage)
            // * 4  (max amount of channels outputting at once)
            // * 8  (max master volume)
            left_sample /= 32.0;
            right_sample /= 32.0;

            // If output has two channels, send sound as stereo
            if self.channels == 2 {
                let _ = buffer.try_push(left_output);
                let _ = buffer.try_push(right_output);
            }
            // Otherwise merge sound into mono
            else {
                for _ in 0..self.channels {
                    let _ = buffer.try_push((left_sample / 2.0) + (right_sample / 2.0));
                }
            }
        }
    }

    fn turn_off(&mut self) {
        self.on = false;
        // Reset registers
        self.sample_delay_counter = 0;
        self.period_delay_counter = 0;
        self.div_apu = 0;
        self.last_div_bit = false;
        self.pan_options = PanRegister::from_bits_truncate(0);
        self.left_volume = 1;
        self.right_volume = 1;
        // Reset channel registers
        self.square_channel_1 = SquareChannel::new();
        self.square_channel_2 = SquareChannel::new();
        let wave_ram = self.wave_channel.wave_ram;
        self.wave_channel = WaveChannel::new();
        self.wave_channel.wave_ram = wave_ram;
        self.noise_channel = NoiseChannel::new();
    }
}

impl MemoryAccess for APU {
    fn mem_read(&self, address: u16) -> u8 {
        match address {
            // NR10 - NR14
            0xFF10..=0xFF14 => self.square_channel_1.read_register(address),
            // NR21 - NR24
            0xFF16..=0xFF19 => self.square_channel_2.read_register(address),
            // NR30 - NR34 + Wave RAM
            0xFF1A..=0xFF1E | 0xFF30..=0xFF3F => self.wave_channel.read_register(address),
            // NR41 - NR44
            0xFF20..=0xFF23 => self.noise_channel.read_register(address),
            // NR50 - Master volume
            0xFF24 => ((self.left_volume - 1) << 4) | (self.right_volume - 1),
            // NR51 - Sound panning
            0xFF25 => self.pan_options.bits(),
            // NR52 - Master control
            0xFF26 => {
                ((self.on as u8) << 7)
                    | ((self.noise_channel.channel_on as u8) << 3)
                    | ((self.wave_channel.dac_on as u8) << 2)
                    | ((self.square_channel_2.channel_on as u8) << 1)
                    | (self.square_channel_1.channel_on as u8)
                    | 0x70
            }
            _ => 0xFF,
        }
    }

    fn mem_write(&mut self, address: u16, value: u8) {
        // Registers apart from master control and wave RAM cant be written to
        // when APU is turned off
        if !self.on && !matches!(address, 0xFF26 | 0xFF30..=0xFF3F) {
            return;
        }
        match address {
            // NR10 - NR14
            0xFF10..=0xFF14 => {
                self.square_channel_1.write_register(address, value);
            }
            // NR21 - NR24
            0xFF16..=0xFF19 => {
                self.square_channel_2.write_register(address, value);
            }
            // NR30 - NR34 + Wave RAM
            0xFF1A..=0xFF1E | 0xFF30..=0xFF3F => self.wave_channel.write_register(address, value),
            // NR41 - NR44
            0xFF20..=0xFF23 => self.noise_channel.write_register(address, value),
            // NR50 - Master volume
            0xFF24 => {
                self.left_volume = ((value >> 4) & 0b111) + 1;
                self.right_volume = (value & 0b111) + 1;
            }
            // NR51 - Sound panning
            0xFF25 => self.pan_options = PanRegister::from_bits_truncate(value),
            // NR52 - Master control
            0xFF26 => {
                let on = value & 0b1000_0000 > 0;
                if self.on && !on {
                    self.turn_off();
                } else {
                    self.on = on;
                }
            }
            _ => {}
        }
    }
}
