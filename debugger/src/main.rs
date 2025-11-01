use dmg_2025_core::CPU;
use dotenv::dotenv;
use std::{env, error, fs};

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
