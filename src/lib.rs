use std::sync::Arc;
use wasm_bindgen::prelude::*;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    keyboard::PhysicalKey,
    window::Window,
};

mod audio;
use audio::*;
mod renderer;
use renderer::*;
mod cpu;
use cpu::*;
mod options;
use options::*;

const CANVAS_ID: &str = "canvas";

#[wasm_bindgen]
pub fn run() -> Result<Proxy, JsValue> {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).unwrap_throw();

    let rom = include_bytes!("../roms/test.gb");

    use winit::platform::web::EventLoopExtWebSys;

    let event_loop = EventLoop::with_user_event().build().unwrap_throw();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
    let app = App::new(&event_loop, rom.to_vec());
    let proxy = event_loop.create_proxy();
    event_loop.spawn_app(app);

    Ok(Proxy { proxy })
}

#[wasm_bindgen]
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

    pub fn run_cpu(&self, millis: f32) {
        self.send(UserEvent::RunCPU(millis));
    }
}

#[derive(Debug)]
pub enum UserEvent {
    InitRenderer(Box<Renderer>),
    RunCPU(f32),
    Test(String),
}

pub struct App {
    proxy: Option<winit::event_loop::EventLoopProxy<UserEvent>>,
    options: Options,
    renderer: Option<Renderer>,
    audio: AudioHandler,
    input_state: InputFlag,
    cpu: CPU,
    last_cpu_frame: u8,
}

impl App {
    pub fn new(event_loop: &EventLoop<UserEvent>, rom: Vec<u8>) -> Self {
        let mut cpu = CPU::new(rom);

        let audio_config = AudioHandler::get_audio_config();
        let audio_consumer = cpu.init_audio_buffer(&audio_config);
        let audio = AudioHandler::init(audio_consumer);

        let options = Options::default();
        Self {
            proxy: Some(event_loop.create_proxy()),
            options,
            renderer: None,
            audio,
            input_state: InputFlag::from_bits_truncate(0xFF),
            cpu,
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
                assert!(proxy
                    .send_event(UserEvent::InitRenderer(Box::new(
                        Renderer::new(window)
                            .await
                            .expect("Unable to create canvas")
                    )))
                    .is_ok())
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
                if self.last_cpu_frame != self.cpu.frame_counter {
                    renderer.update_display(self.cpu.get_display_buffer());
                    self.last_cpu_frame = self.cpu.frame_counter;
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
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => {
                for (input, key) in &self.options.keybinds {
                    if code == *key {
                        self.input_state.set(*input, !key_state.is_pressed());
                    }
                }
            }
            _ => {}
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::InitRenderer(mut renderer) => {
                // This is where proxy.send_event() ends up
                renderer.resize(
                    renderer.window.inner_size().width,
                    renderer.window.inner_size().height,
                );
                self.renderer = Some(*renderer);
            }
            UserEvent::RunCPU(millis) => {
                self.cpu.update_input(&self.input_state);
                self.cpu.run(millis);
            }
            UserEvent::Test(string) => {
                web_sys::console::log_1(&JsValue::from_str(&string));
            }
        }
    }
}
