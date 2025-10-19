mod execution;
mod interrupts;
mod readwrite;

use super::*;
pub use interrupts::*;
pub use readwrite::*;

/// The main processing unit
#[allow(clippy::upper_case_acronyms)]
#[derive(Deserialize, Serialize)]
pub struct CPU {
    mem: Memory,
    reg: Registers,
    ppu: PPU,
    apu: APU,
    timer: Timer,
    input: InputReg,
    istate: InterruptState,
    halt: bool,
    pub frame_counter: u8,
    cycle_counter: u32,
}

impl CPU {
    pub fn new(rom_file: Vec<u8>) -> Result<Self, ROMValidationError> {
        let mem = Memory::new(rom_file)?;
        Ok(Self {
            mem,
            reg: Registers::new(),
            ppu: PPU::new(),
            apu: APU::new(),
            timer: Timer::new(),
            input: InputReg::new(),
            istate: InterruptState::new(),
            halt: false,
            frame_counter: 0,
            cycle_counter: 0,
        })
    }

    /// Initializes a ring buffer for audio playback and returns its consumer.
    /// Remember to set sample rate using set_audio_sample_rate
    pub fn init_audio_buffer(
        &mut self,
        sample_capacity: usize,
        channels: usize,
    ) -> AudioBufferConsumer {
        self.apu.init_buffer(sample_capacity, channels)
    }

    /// Sets the sample rate for the audio processing unit.
    /// Is set separately so audio emulation can be adjusted to possible emulation speed changes
    pub fn set_audio_sample_rate(&mut self, sample_rate: u32) {
        self.apu.set_sample_rate(sample_rate);
    }

    /// Returns the latest fully drawn display buffer for rendering
    pub fn get_display_buffer(&self) -> &DisplayBuffer {
        &self.ppu.display
    }

    /// Updates input state
    pub fn update_input(&mut self, input: &InputFlag) {
        if self.input.update(*input) {
            self.request_interrupt(InterruptFlag::JOYPAD);
        }
    }

    const MS_PER_M_CYCLE: f32 = 0.0009536743;

    /// Runs Game Boy for given amount of milliseconds
    pub fn run(&mut self, millis: f32) {
        let target_cycles = (millis / Self::MS_PER_M_CYCLE).floor() as u32;
        while self.cycle_counter < target_cycles {
            self.run_instruction();
        }
        self.cycle_counter = 0;
    }
}
