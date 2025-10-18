use super::*;
use serde::{Deserialize, Serialize};

/// A color in linear RGB space
#[wasm_bindgen]
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

#[wasm_bindgen]
impl Color {
    #[wasm_bindgen(constructor)]
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
}

/// Represents color palette for display
#[wasm_bindgen]
#[repr(C)]
#[derive(
    Debug, Copy, Clone, Deserialize, Serialize, PartialEq, bytemuck::Pod, bytemuck::Zeroable,
)]
pub struct Palette(pub Color, pub Color, pub Color, pub Color);

#[wasm_bindgen]
impl Palette {
    #[wasm_bindgen(constructor)]
    pub fn new(col1: Color, col2: Color, col3: Color, col4: Color) -> Self {
        Self(col1, col2, col3, col4)
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::new(
            Color::new(1.0, 1.0, 1.0),
            Color::new(0.6666, 0.6666, 0.6666),
            Color::new(0.3333, 0.3333, 0.3333),
            Color::new(0.0, 0.0, 0.0),
        )
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct Options {
    pub palette: Palette,
    pub speed: f32,
    pub volume: f32,
}

#[wasm_bindgen]
impl Options {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_palette(&mut self, palette: &Palette) {
        self.palette = *palette;
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            palette: Palette::default(),
            speed: 1.0,
            volume: 1.0,
        }
    }
}

#[derive(Debug)]
pub enum UserEvent {
    InitRenderer(Box<Renderer>),
    LoadRom(Vec<u8>),
    RunCPU(f32),
    UpdateInput(String, bool),
    UpdateOptions(Options),
    Test(String),
}

// A proxy to communicate with the event loop from frontend
#[wasm_bindgen]
pub struct Proxy {
    pub(crate) proxy: EventLoopProxy<UserEvent>,
}

#[wasm_bindgen]
impl Proxy {
    fn send(&self, event: UserEvent) {
        self.proxy
            .send_event(event)
            .expect("Couldn't send event to EventLoop");
    }
    pub fn test(&self, str: String) {
        self.send(UserEvent::Test(str));
    }

    pub fn load_rom(&self, rom: Vec<u8>) {
        self.send(UserEvent::LoadRom(rom));
    }

    pub fn run_cpu(&self, millis: f32) {
        self.send(UserEvent::RunCPU(millis));
    }

    pub fn update_input(&self, key: String, pressed: bool) {
        self.send(UserEvent::UpdateInput(key, pressed));
    }

    pub fn update_options(&self, options: &Options) {
        self.send(UserEvent::UpdateOptions(*options));
    }
}
