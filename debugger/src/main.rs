use dotenv::dotenv;
use gb_web_core::CPU;
use std::{env, error, fs};

/// This module simply loads a ROM from file path and runs it on the CPU
/// It's ran locally instead of on WASM, so it can be used for debugging the emulator core
///
/// The configuration happens through a .env file in the working directory
/// that has the following variables:
/// EXECUTION_TIME: determines how many milliseconds the core is ran for at a time
/// ROM_PATH: the local path to a ROM file
pub fn main() -> Result<(), Box<dyn error::Error + 'static>> {
    dotenv().expect("No .env file found in working directory");

    let time = env::var("EXECUTION_TIME")?.parse::<f32>()?;
    let rom_path = env::var("ROM_PATH")?;
    let rom = fs::read(rom_path)?;

    let mut cpu = CPU::new(rom).unwrap();
    cpu.set_audio_sample_rate(44100);
    let mut elapsed = 0.0;
    loop {
        cpu.run(time);
        elapsed += time;
        println!("Ran CPU for {elapsed} ms")
    }
}
