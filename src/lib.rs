pub use std::sync::{Arc, Mutex};
pub use wasm_bindgen::prelude::*;
pub use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

mod audio;
use audio::*;
mod renderer;
use renderer::*;
mod cpu;
use cpu::input::InputFlag;
use cpu::*;
mod options;
use options::*;

const CANVAS_ID: &str = "canvas";
pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

#[wasm_bindgen]
pub fn run() -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).unwrap_throw();

    let rom = include_bytes!("../roms/test.gb");

    let event_loop = EventLoop::with_user_event().build().unwrap_throw();
    let mut app = App::new(&event_loop, rom.to_vec());
    event_loop.run_app(&mut app).unwrap_throw();

    Ok(())
}

pub struct App {
    proxy: Option<winit::event_loop::EventLoopProxy<Renderer>>,
    options: Options,
    renderer: Option<Renderer>,
    input_state: InputFlag,
    cpu: CPU,
    audio: AudioHandler,
}

impl App {
    pub fn new(event_loop: &EventLoop<Renderer>, rom: Vec<u8>) -> Self {
        let options = Options::default();
        let audio = AudioHandler::init();
        let cpu = CPU::new(rom, audio.sample_rate);
        Self {
            proxy: Some(event_loop.create_proxy()),
            options,
            renderer: None,
            audio,
            input_state: InputFlag::from_bits_truncate(0xFF),
            cpu,
        }
    }
}

impl ApplicationHandler<Renderer> for App {
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
                    .send_event(
                        Renderer::new(window)
                            .await
                            .expect("Unable to create canvas")
                    )
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
        let (renderer, audio, cpu) = (
            self.renderer.as_mut().unwrap(),
            &mut self.audio,
            &mut self.cpu,
        );

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => renderer.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                cpu.update_input(&self.input_state);

                loop {
                    if cpu.execute() {
                        break;
                    }
                }

                audio.update_audio(cpu.apu.receive_buffer());
                renderer.update_display(&cpu.ppu.display);

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

    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: Renderer) {
        // This is where proxy.send_event() ends up
        event.window.request_redraw();
        event.resize(
            event.window.inner_size().width,
            event.window.inner_size().height,
        );
        self.renderer = Some(event);
    }
}
