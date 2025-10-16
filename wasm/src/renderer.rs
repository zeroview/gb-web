use wgpu::util::DeviceExt;

use super::*;
use dmg_2025_core::{DISPLAY_BUFFER_SIZE, DisplayBuffer};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct OptionsUniform {
    palette: Palette,
    display_width: u32,
    display_height: u32,
    canvas_width: u32,
    canvas_height: u32,
}

impl OptionsUniform {
    fn new(options: &Options) -> Self {
        Self {
            palette: options.palette,
            display_width: 160,
            display_height: 144,
            canvas_width: 0,
            canvas_height: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
// Uniform of display data
struct DisplayUniform {
    pub pixels: DisplayBuffer,
}

impl DisplayUniform {
    fn new() -> Self {
        Self {
            pixels: [0; DISPLAY_BUFFER_SIZE],
        }
    }
}

#[derive(Debug)]
pub struct Renderer {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub is_surface_configured: bool,
    pub render_pipeline: wgpu::RenderPipeline,
    pub window: Arc<Window>,

    options_uniform: OptionsUniform,
    options_buffer: wgpu::Buffer,
    options_bind_group: wgpu::BindGroup,

    display_uniform: DisplayUniform,
    display_buffer: wgpu::Buffer,
    display_bind_group: wgpu::BindGroup,
}

impl Renderer {
    fn init_uniform_buffer<U>(
        device: &wgpu::Device,
        uniform: U,
        name: &str,
    ) -> (wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup)
    where
        U: bytemuck::NoUninit,
    {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{name} Buffer")),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("{name} Bind Group Layout")),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{name} Bind Group")),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        (buffer, bind_group_layout, bind_group)
    }

    pub async fn new(window: Arc<Window>, options: &Options) -> anyhow::Result<Self> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let mut limits = wgpu::Limits::downlevel_webgl2_defaults();
        // Increase max texture size so website can be ran on bigger screens
        limits.max_texture_dimension_2d = 4096;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_features: wgpu::Features::empty(),
                required_limits: limits,
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        // Initialize options uniform for the GPU
        let options_uniform = OptionsUniform::new(options);
        let (options_buffer, options_bind_group_layout, options_bind_group) =
            Self::init_uniform_buffer(&device, options_uniform, "Options");

        // Initialize display data for the GPU
        let display_uniform = DisplayUniform::new();
        let (display_buffer, display_bind_group_layout, display_bind_group) =
            Self::init_uniform_buffer(&device, display_uniform, "Display");

        // Initialize render pipeline
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&options_bind_group_layout, &display_bind_group_layout],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            render_pipeline,
            window,

            options_uniform,
            options_buffer,
            options_bind_group,

            display_uniform,
            display_buffer,
            display_bind_group,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;

            self.options_uniform.canvas_width = width;
            self.options_uniform.canvas_height = height;
            self.queue.write_buffer(
                &self.options_buffer,
                0,
                bytemuck::cast_slice(&[self.options_uniform]),
            );
        }
    }

    pub fn update_options(&mut self, options: &Options) {
        self.options_uniform.palette = options.palette;
        self.queue.write_buffer(
            &self.options_buffer,
            0,
            bytemuck::cast_slice(&[self.options_uniform]),
        );
    }

    pub fn update_display(&mut self, display: &DisplayBuffer) {
        self.display_uniform.pixels = *display;
        self.queue.write_buffer(
            &self.display_buffer,
            0,
            bytemuck::cast_slice(&[self.display_uniform]),
        );
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // We can't render unless the surface is configured
        if !self.is_surface_configured {
            return Ok(());
        }

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_bind_group(0, &self.options_bind_group, &[]);
        render_pass.set_bind_group(1, &self.display_bind_group, &[]);
        render_pass.draw(0..6, 0..1);
        drop(render_pass);

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        self.window.request_redraw();
        Ok(())
    }
}
