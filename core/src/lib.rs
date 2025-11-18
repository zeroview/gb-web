use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

mod apu;
mod cpu;
mod input;
mod memory;
mod ppu;
mod registers;
mod timer;
use apu::*;
use cpu::*;
use input::*;
use memory::*;
use ppu::*;
use registers::*;
use timer::*;

pub use apu::AudioBufferConsumer;
pub use cpu::CPU;
pub use input::InputFlag;
pub use memory::{CartridgeInfo, MemoryInitializationError, MemoryInitializationErrorType};
pub use ppu::{DISPLAY_BUFFER_SIZE, DisplayBuffer};
