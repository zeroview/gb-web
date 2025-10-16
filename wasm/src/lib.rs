use dmg_2025_core::*;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::Window,
};

mod audio;
use audio::*;
mod renderer;
use renderer::*;
mod options;
use options::*;

const CANVAS_ID: &str = "canvas";

#[wasm_bindgen]
pub fn spawn_event_loop() -> Result<Proxy, JsValue> {
    // Initialize debugging tools
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).unwrap_throw();

    // Create event loop and a proxy to communicate with it from the frontend
    let event_loop = EventLoop::with_user_event().build().unwrap_throw();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
    let app = App::new(&event_loop);
    let proxy = event_loop.create_proxy();

    use winit::platform::web::EventLoopExtWebSys;
    event_loop.spawn_app(app);
    Ok(Proxy { proxy })
}

#[wasm_bindgen]
// A proxy to communicate with the event loop from frontend
pub struct Proxy {
    proxy: EventLoopProxy<UserEvent>,
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
}

#[derive(Debug)]
pub enum UserEvent {
    InitRenderer(Box<Renderer>),
    LoadRom(Vec<u8>),
    RunCPU(f32),
    UpdateInput(String, bool),
    Test(String),
}

pub struct App {
    proxy: Option<winit::event_loop::EventLoopProxy<UserEvent>>,
    renderer: Option<Renderer>,
    audio: Option<AudioHandler>,
    input_state: InputFlag,
    cpu: Option<CPU>,
    last_cpu_frame: u8,
}

impl App {
    pub fn new(event_loop: &EventLoop<UserEvent>) -> Self {
        Self {
            proxy: Some(event_loop.create_proxy()),
            renderer: None,
            audio: None,
            input_state: InputFlag::from_bits_truncate(0xFF),
            cpu: None,
            last_cpu_frame: 0,
        }
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();

        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowAttributesExtWebSys;

        let window = wgpu::web_sys::window().unwrap_throw();
        let document = window.document().unwrap_throw();
        let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
        let html_canvas_element = canvas.unchecked_into();
        window_attributes = window_attributes.with_canvas(Some(html_canvas_element));

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        // Run the future asynchronously and use the
        // proxy to send the results to the event loop
        if let Some(proxy) = self.proxy.take() {
            wasm_bindgen_futures::spawn_local(async move {
                assert!(
                    proxy
                        .send_event(UserEvent::InitRenderer(Box::new(
                            Renderer::new(window)
                                .await
                                .expect("Unable to create canvas")
                        )))
                        .is_ok()
                )
            });
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if self.renderer.is_none() {
            return;
        }
        let renderer = self.renderer.as_mut().unwrap();

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => renderer.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                if let Some(cpu) = &self.cpu {
                    // Update buffer only when there is new frame available
                    if self.last_cpu_frame != cpu.frame_counter {
                        renderer.update_display(cpu.get_display_buffer());
                        self.last_cpu_frame = cpu.frame_counter;
                    }

                    match renderer.render() {
                        Ok(_) => {}
                        // Reconfigure the surface if it's lost or outdated
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            let size = renderer.window.inner_size();
                            renderer.resize(size.width, size.height);
                        }
                        Err(e) => {
                            log::error!("Unable to render {}", e);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::InitRenderer(mut renderer) => {
                renderer.window.request_redraw();
                // This is where proxy.send_event() ends up
                renderer.resize(
                    renderer.window.inner_size().width,
                    renderer.window.inner_size().height,
                );
                self.renderer = Some(*renderer);
            }
            UserEvent::LoadRom(rom) => {
                let mut cpu = CPU::new(rom);
                let audio_config = AudioHandler::get_audio_config();
                let audio_consumer = cpu.init_audio_buffer(&audio_config);
                let audio = AudioHandler::init(audio_consumer);

                self.cpu = Some(cpu);
                self.audio = Some(audio);
            }
            UserEvent::RunCPU(millis) => {
                if let Some(cpu) = &mut self.cpu {
                    cpu.run(millis);
                }
            }
            UserEvent::UpdateInput(input_str, pressed) => {
                let input_option = match input_str.as_str() {
                    "Right" => Some(InputFlag::RIGHT),
                    "Left" => Some(InputFlag::LEFT),
                    "Up" => Some(InputFlag::UP),
                    "Down" => Some(InputFlag::DOWN),
                    "A" => Some(InputFlag::A),
                    "B" => Some(InputFlag::B),
                    "Select" => Some(InputFlag::SELECT),
                    "Start" => Some(InputFlag::START),
                    _ => None,
                };
                if let Some(input_flag) = input_option {
                    self.input_state.set(input_flag, !pressed);

                    if let Some(cpu) = &mut self.cpu {
                        cpu.update_input(&self.input_state);
                    }
                }
            }
            UserEvent::Test(string) => {
                log::info!("Test from JS: {}", &string);
            }
        }
    }
}
