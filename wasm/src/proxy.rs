use super::*;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use web_sys::js_sys;

/// A color in linear RGB space
#[repr(C)]
#[derive(
    Tsify, Debug, Copy, Clone, PartialEq, Deserialize, Serialize, bytemuck::Pod, bytemuck::Zeroable,
)]
#[tsify(from_wasm_abi)]
pub struct Color(pub f32, pub f32, pub f32, pub f32);

/// Represents color palette for display
#[repr(C)]
#[derive(
    Tsify, Debug, Copy, Clone, Deserialize, Serialize, PartialEq, bytemuck::Pod, bytemuck::Zeroable,
)]
#[tsify(from_wasm_abi)]
pub struct Palette(pub Color, pub Color, pub Color, pub Color);

impl Palette {
    pub fn new(col1: Color, col2: Color, col3: Color, col4: Color) -> Self {
        Self(col1, col2, col3, col4)
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::new(
            Color(1.0, 1.0, 1.0, 1.0),
            Color(0.6666, 0.6666, 0.6666, 1.0),
            Color(0.3333, 0.3333, 0.3333, 1.0),
            Color(0.0, 0.0, 0.0, 1.0),
        )
    }
}

#[derive(Tsify, Default, Debug, Clone, Copy, Deserialize, Serialize)]
#[tsify(from_wasm_abi)]
pub struct EmulatorOptions {
    pub volume: f32,
    pub scale_offset: i32,
    pub display_glow_strength: f32,
    pub background_glow_strength: f32,
    pub glow_iterations: usize,
    pub glow_radius: f32,
    pub ambient_light: f32,
    pub(crate) palette: Palette,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Default)]
pub struct ROMInfo {
    /// The ROM title, interpreted from header
    pub(crate) title: String,
    /// If RAM should be saved externally
    /// (header says that cartridge has RAM and battery for saving it)
    pub should_be_saved: bool,
    /// The hash of the ROM file
    pub hash: u32,
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize)]
#[tsify(from_wasm_abi)]
pub enum BridgeQuery {
    LoadROM {
        #[tsify(type = "Uint8Array")]
        file: Vec<u8>,
        is_zip: bool,
    },
    LoadRAM {
        #[tsify(type = "Uint8Array")]
        ram: Vec<u8>,
    },
    RunCPU {
        millis: f32,
    },
    SaveRAM {},
    SerializeCPU {},
    DeserializeCPU {
        #[tsify(type = "Uint8Array")]
        buffer: Vec<u8>,
    },
    SetPaused {
        paused: bool,
    },
    SetSpeed {
        speed: f32,
    },
    UpdateInput {
        input: String,
        pressed: bool,
    },
    UpdateOptions {
        options: EmulatorOptions,
    },
}

#[wasm_bindgen]
impl ROMInfo {
    #[wasm_bindgen(getter)]
    pub fn title(&self) -> String {
        self.title.clone()
    }
}

pub enum BridgeResponse {
    /// New ROM is loaded,
    /// returns info about newly loaded ROM
    ROMLoaded(ROMInfo),
    /// Returns the current RAM buffer
    RAMSaved(Vec<u8>),
    /// CPU is successfully serialized into a save state,
    /// returns the serialized CPU
    CPUSerialized(Vec<u8>),
}

#[derive(Debug)]
pub struct BridgeRequest {
    resolve: js_sys::Function,
    reject: js_sys::Function,
    pub query: Option<BridgeQuery>,
}

impl BridgeRequest {
    fn full_resolve(&self, response: Option<BridgeResponse>) {
        if let Some(response) = response {
            use BridgeResponse as R;
            match response {
                R::ROMLoaded(info) => self.resolve.call1(&JsValue::NULL, &info.into()),
                R::CPUSerialized(buffer) => self
                    .resolve
                    .call1(&JsValue::NULL, &js_sys::Uint8Array::new_from_slice(&buffer)),
                R::RAMSaved(buffer) => self
                    .resolve
                    .call1(&JsValue::NULL, &js_sys::Uint8Array::new_from_slice(&buffer)),
            }
            .unwrap_throw();
        } else {
            self.resolve.call0(&JsValue::NULL).unwrap_throw();
        }
    }

    pub fn resolve(&self) {
        self.full_resolve(None);
    }

    pub fn respond(&self, response: BridgeResponse) {
        self.full_resolve(Some(response));
    }

    pub fn reject(&self, reason: &str) {
        self.reject
            .call1(&JsValue::NULL, &reason.into())
            .unwrap_throw();
    }
}

#[derive(Debug)]
pub enum UserEvent {
    InitRenderer(Box<Renderer>),
    Query(BridgeRequest),
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

    pub fn query(&self, query: BridgeQuery) -> js_sys::Promise {
        js_sys::Promise::new(&mut |resolve, reject| {
            let request = BridgeRequest {
                resolve,
                reject,
                query: Some(query.clone()),
            };
            self.send(UserEvent::Query(request));
        })
    }
}
