use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::mem::{Discriminant, discriminant};
use web_sys::js_sys;

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
#[derive(Default, Debug, Clone, Copy)]
pub struct EmulatorOptions {
    pub palette: Palette,
    pub volume: f32,
    pub scale_offset: i32,
    pub display_glow_strength: f32,
    pub background_glow_strength: f32,
    pub glow_iterations: usize,
    pub glow_radius: f32,
    pub ambient_light: f32,
}

#[wasm_bindgen]
impl EmulatorOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_palette(&mut self, palette: &Palette) {
        self.palette = *palette;
    }
}

pub enum Callback {
    /// Called when new ROM is loaded
    /// Arguments: ROM title (from header), hash of ROM file
    ROMLoaded(String, u32),
    /// Called when CPU is successfully serialized into a save state
    /// Arguments: the serialized CPU in binary
    CPUSerialized(Vec<u8>),
    /// Called when CPU is successfully deserialized, a.k.a. save state is loaded
    CPUDeserialized,
    /// Called when something goes wrong
    /// Arguments: information about error
    Error(String),
}

#[wasm_bindgen]
#[derive(Debug, Default, Clone)]
pub struct ProxyCallbacks {
    callbacks: HashMap<Discriminant<Callback>, js_sys::Function>,
}

#[wasm_bindgen]
impl ProxyCallbacks {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    fn set_callback(&mut self, callback_type: Callback, function: &js_sys::Function) {
        self.callbacks
            .insert(discriminant(&callback_type), function.clone());
    }

    pub fn set_rom_loaded(&mut self, function: &js_sys::Function) {
        self.set_callback(Callback::ROMLoaded(String::new(), 0), function);
    }

    pub fn set_cpu_serialized(&mut self, function: &js_sys::Function) {
        self.set_callback(Callback::CPUSerialized(Vec::new()), function);
    }

    pub fn set_cpu_deserialized(&mut self, function: &js_sys::Function) {
        self.set_callback(Callback::CPUDeserialized, function);
    }

    pub fn set_error(&mut self, function: &js_sys::Function) {
        self.set_callback(Callback::Error(String::new()), function);
    }

    pub(crate) fn call(&self, callback: Callback) {
        if let Some(function) = self.callbacks.get(&discriminant(&callback)) {
            match callback {
                Callback::ROMLoaded(title, hash) => {
                    function.call2(&JsValue::NULL, &title.into(), &hash.into())
                }
                Callback::CPUSerialized(buffer) => {
                    function.call1(&JsValue::NULL, &js_sys::Uint8Array::new_from_slice(&buffer))
                }
                Callback::CPUDeserialized => function.call0(&JsValue::NULL),
                Callback::Error(error) => function.call1(&JsValue::NULL, &error.into()),
            }
            .unwrap_throw();
        }
    }
}

#[derive(Debug)]
pub enum UserEvent {
    InitRenderer(Box<Renderer>),
    LoadRom(Vec<u8>, bool),
    RunCPU(f32),
    SerializeCPU,
    DeserializeCPU(Vec<u8>),
    SetPaused(bool),
    SetSpeed(f32),
    UpdateInput(String, bool),
    UpdateOptions(EmulatorOptions),
    SetCallbacks(ProxyCallbacks),
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

    pub fn load_rom(&self, rom: Vec<u8>, is_zip: bool) {
        self.send(UserEvent::LoadRom(rom, is_zip));
    }

    pub fn run_cpu(&self, millis: f32) {
        self.send(UserEvent::RunCPU(millis));
    }

    pub fn serialize_cpu(&self) {
        self.send(UserEvent::SerializeCPU);
    }

    pub fn deserialize_cpu(&self, cpu: Vec<u8>) {
        self.send(UserEvent::DeserializeCPU(cpu));
    }

    pub fn set_paused(&self, paused: bool) {
        self.send(UserEvent::SetPaused(paused));
    }

    pub fn set_speed(&self, speed: f32) {
        self.send(UserEvent::SetSpeed(speed));
    }

    pub fn update_input(&self, key: String, pressed: bool) {
        self.send(UserEvent::UpdateInput(key, pressed));
    }

    pub fn update_options(&self, options: &EmulatorOptions) {
        self.send(UserEvent::UpdateOptions(*options));
    }

    pub fn set_callbacks(&self, callbacks: &ProxyCallbacks) {
        self.send(UserEvent::SetCallbacks(callbacks.clone()));
    }
}
