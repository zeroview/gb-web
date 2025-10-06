use crate::cpu::InputFlag;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use winit::keyboard::KeyCode;

#[repr(C)]
#[derive(
    Debug, Copy, Clone, PartialEq, Deserialize, Serialize, bytemuck::Pod, bytemuck::Zeroable,
)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}
impl Color {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
}

#[repr(C)]
#[derive(
    Debug, Copy, Clone, Deserialize, Serialize, PartialEq, bytemuck::Pod, bytemuck::Zeroable,
)]
/// Represents color palette for display
pub struct Palette(pub Color, pub Color, pub Color, pub Color);

impl Palette {
    pub fn original() -> Self {
        Self(
            Color::new(1.0, 1.0, 1.0),
            Color::new(0.6666, 0.6666, 0.6666),
            Color::new(0.3333, 0.3333, 0.3333),
            Color::new(0.0000, 0.0000, 0.0000),
        )
    }

    pub fn lcd() -> Self {
        Self(
            Color::new(0.8784, 0.9725, 0.8156),
            Color::new(0.5333, 0.7529, 0.4392),
            Color::new(0.2039, 0.4078, 0.3372),
            Color::new(0.0156, 0.0470, 0.0627),
        )
    }

    pub fn get_col(&self, index: u8) -> Color {
        match index {
            0 => self.0,
            1 => self.1,
            2 => self.2,
            3 => self.3,
            _ => panic!("Index too high: Only 4 colors in palette"),
        }
    }

    pub fn get_mut(&mut self, index: u8) -> &mut Color {
        match index {
            0 => &mut self.0,
            1 => &mut self.1,
            2 => &mut self.2,
            3 => &mut self.3,
            _ => panic!("Index too high: Only 4 colors in palette"),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct Options {
    pub keybinds: HashMap<InputFlag, KeyCode>,
    pub palette_preset: u8,
    pub custom_palette: Palette,
    pub audio_sample_rate: u32,
    pub volume: u8,
}

impl Options {
    pub fn load() -> Self {
        todo!();
    }

    fn init_default() -> Self {
        let options = Options::default();
        options.save();
        options
    }

    pub fn save(&self) {
        todo!();
    }

    pub fn default_keybinds() -> HashMap<InputFlag, KeyCode> {
        HashMap::from([
            (InputFlag::RIGHT, KeyCode::ArrowRight),
            (InputFlag::LEFT, KeyCode::ArrowLeft),
            (InputFlag::UP, KeyCode::ArrowUp),
            (InputFlag::DOWN, KeyCode::ArrowDown),
            (InputFlag::A, KeyCode::KeyX),
            (InputFlag::B, KeyCode::KeyZ),
            (InputFlag::SELECT, KeyCode::Backspace),
            (InputFlag::START, KeyCode::Enter),
        ])
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            keybinds: Self::default_keybinds(),
            palette_preset: 0,
            custom_palette: Palette::original(),
            audio_sample_rate: 48000,
            volume: 100,
        }
    }
}
