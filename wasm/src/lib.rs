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
mod proxy;
use proxy::*;

const CANVAS_ID: &str = "canvas";
const EVENT_LISTENER_ID: &str = "eventListener";

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

pub struct App {
    window: web_sys::Window,
    proxy: Option<winit::event_loop::EventLoopProxy<UserEvent>>,
    event_target: Option<web_sys::Element>,
    renderer: Option<Renderer>,
    options: EmulatorOptions,
    audio: AudioHandler,
    input_state: InputFlag,
    cpu: Option<CPU>,
    last_cpu_frame: u8,
}

impl App {
    pub fn new(event_loop: &EventLoop<UserEvent>) -> Self {
        Self {
            window: web_sys::window().unwrap(),
            proxy: Some(event_loop.create_proxy()),
            event_target: None,
            renderer: None,
            options: EmulatorOptions::default(),
            audio: AudioHandler::new(),
            input_state: InputFlag::from_bits_truncate(0xFF),
            cpu: None,
            last_cpu_frame: 0,
        }
    }

    fn get_document() -> web_sys::Document {
        web_sys::window().unwrap_throw().document().unwrap_throw()
    }

    fn send_event(&self, event_name: &str, detail: JsValue) -> Result<(), JsValue> {
        if let Some(target) = &self.event_target {
            let init = web_sys::CustomEventInit::new();
            init.set_detail(&detail);
            let event = web_sys::CustomEvent::new_with_event_init_dict(event_name, &init)?;
            target.dispatch_event(&event.into())?;
        }
        Ok(())
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();

        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowAttributesExtWebSys;

        let document = Self::get_document();
        self.event_target = document.get_element_by_id(EVENT_LISTENER_ID);
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
                log::info!("Renderer initialized");
                renderer.window.request_redraw();
                renderer.resize(
                    renderer.window.inner_size().width,
                    renderer.window.inner_size().height,
                );
                renderer.update_options(&self.options);
                self.renderer = Some(*renderer);
            }
            UserEvent::LoadRom(file, is_zip) => {
                // If loaded rom file is zip, try to unzip it into a ROM File
                let rom = if is_zip {
                    use std::io::{BufReader, Cursor, Read, Result};
                    use std::path::Path;

                    let mut rom_option = None;
                    if let Ok(mut archive) = zip::ZipArchive::new(Cursor::new(&file[..])) {
                        // Loop through files in zip to find ROM
                        for i in 0..archive.len() {
                            if let Ok(archive_file) = archive.by_index(i) {
                                // Choose first file inside zip that either has no extension or .gb
                                if Path::new(archive_file.name())
                                    .extension()
                                    .is_none_or(|ext| ext == "gb")
                                {
                                    let buf = BufReader::new(archive_file);
                                    let rom_result: Result<Vec<u8>> = buf.bytes().collect();
                                    if let Ok(deflated_rom) = rom_result {
                                        rom_option = Some(deflated_rom);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    rom_option
                } else {
                    Some(file)
                };

                if let Some(rom) = rom {
                    match CPU::new(rom) {
                        Ok(mut cpu) => {
                            let _ = self
                                .send_event("romloaded", JsValue::from(cpu.get_cartridge_title()));
                            // Initialize audio playback
                            cpu.set_audio_sample_rate(self.audio.sample_rate);
                            let audio_consumer = cpu
                                .init_audio_buffer(self.audio.sample_capacity, self.audio.channels);
                            self.audio.init_playback(audio_consumer);
                            self.cpu = Some(cpu);
                        }
                        Err(e) => {
                            let _ = self.send_event("romloadfailed", JsValue::from(e.to_string()));
                        }
                    }
                } else {
                    let _ =
                        self.send_event("romloadfailed", JsValue::from("Zip archive is invalid"));
                }
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
            UserEvent::UpdateOptions(options) => {
                // Update renderer options
                if let Some(renderer) = &mut self.renderer {
                    renderer.update_options(&options);
                }
                // Update audio sample speed
                if let Some(cpu) = &mut self.cpu {
                    let new_sample_rate = if options.speed == 1.0 {
                        self.audio.sample_rate
                    } else {
                        ((self.audio.sample_rate as f32) / options.speed) as u32
                    };
                    cpu.set_audio_sample_rate(new_sample_rate);
                }
                // Update audio volume
                *self.audio.volume.write().unwrap() = options.volume;

                self.options = options;
            }
            UserEvent::Test(string) => {
                log::info!("Test from JS: {}", &string);
            }
        }
    }
}
