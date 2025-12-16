use wgpu::util::DeviceExt;

use super::*;
use gb_web_core::{DISPLAY_BUFFER_SIZE, DisplayBuffer};

mod buffers;
use buffers::*;

#[derive(Debug)]
pub struct Renderer {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub is_surface_configured: bool,
    pub window: Arc<Window>,

    display_render_pipeline: wgpu::RenderPipeline,
    blur_render_pipeline: wgpu::RenderPipeline,
    final_render_pipeline: wgpu::RenderPipeline,

    display_texture: Option<Texture>,
    h_blur_texture: Option<Texture>,
    v_blur_texture: Option<Texture>,
    background_texture: Texture,
    texture_bind_group_layout: wgpu::BindGroupLayout,

    background_definition: BackgroundDefinition,
    background_rendered_rect: Rect,
    show_controls: bool,
    display_options: UniformBuffer<DisplayOptionsUniform>,
    display: UniformBuffer<DisplayBufferUniform>,
    blur_options: UniformBuffer<BlurOptionsUniform>,
    glow_iterations: usize,
    glow_radius: f32,
    final_options: UniformBuffer<FinalOptionsUniform>,
}

impl Renderer {
    fn init_render_pipeline(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        shader: &wgpu::ShaderModule,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
    ) -> wgpu::RenderPipeline {
        let display_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts,
                push_constant_ranges: &[],
            });
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&display_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
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
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            cache: None,
            depth_stencil: None,
            multiview: None,
        })
    }

    pub async fn new(
        window: Arc<Window>,
        background_definition: BackgroundDefinition,
    ) -> anyhow::Result<Self> {
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

        let options = UniformBuffer::<DisplayOptionsUniform>::new(&device, "Options");
        let display = UniformBuffer::<DisplayBufferUniform>::new(&device, "Display");
        // Initialize render pipeline for rendering the raw display data
        let display_shader = device.create_shader_module(wgpu::include_wgsl!("display.wgsl"));
        let display_render_pipeline = Self::init_render_pipeline(
            &device,
            &config,
            &display_shader,
            &[&display.bind_group_layout, &options.bind_group_layout],
        );

        let blur_options = UniformBuffer::<BlurOptionsUniform>::new(&device, "Effect Options");
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("Texture Bind Group Layout"),
            });
        // Initialize render pipeline for blurring shader used for glow effect
        let blur_shader = device.create_shader_module(wgpu::include_wgsl!("blur.wgsl"));
        let blur_render_pipeline = Self::init_render_pipeline(
            &device,
            &config,
            &blur_shader,
            &[&texture_bind_group_layout, &blur_options.bind_group_layout],
        );

        let final_options = UniformBuffer::<FinalOptionsUniform>::new(&device, "Final Options");
        // Load background image into a byte array
        let background_png = include_bytes!("../assets/background.png");
        let background_image = image::load_from_memory(background_png).unwrap();
        let background_rgba = background_image.to_rgba8();
        // Initialize background texture
        let background_texture_size = wgpu::Extent3d {
            width: background_rgba.width(),
            height: background_rgba.height(),
            depth_or_array_layers: 1,
        };
        let background_texture = Texture::new(
            &device,
            &texture_bind_group_layout,
            &background_texture_size,
            "Background",
        );
        queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::TexelCopyTextureInfo {
                texture: &background_texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            &background_rgba,
            // The layout of the texture
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * background_rgba.width()),
                rows_per_image: Some(background_rgba.height()),
            },
            background_texture_size,
        );

        // Initialize render pipeline for final composite pass
        let final_shader = device.create_shader_module(wgpu::include_wgsl!("final.wgsl"));
        let final_render_pipeline = Self::init_render_pipeline(
            &device,
            &config,
            &final_shader,
            &[
                // Display texture
                &texture_bind_group_layout,
                // Blur texture
                &texture_bind_group_layout,
                // Background texture
                &texture_bind_group_layout,
                // Options
                &final_options.bind_group_layout,
            ],
        );

        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            window,

            display_render_pipeline,
            blur_render_pipeline,
            final_render_pipeline,

            display_texture: None,
            h_blur_texture: None,
            v_blur_texture: None,
            background_texture,
            texture_bind_group_layout,

            background_definition,
            background_rendered_rect: Rect::default(),
            show_controls: false,
            display_options: options,
            display,
            blur_options,
            final_options,
            glow_iterations: 0,
            glow_radius: 0.0,
        })
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // We can't render unless the surface is configured
        if !self.is_surface_configured {
            return Ok(());
        }

        // Create command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Display Render Encoder"),
            });

        // Render the Game Boy display onto texture
        let mut display_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Display Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.display_texture.as_ref().unwrap().texture_view,
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
        display_render_pass.set_pipeline(&self.display_render_pipeline);
        display_render_pass.set_bind_group(0, &self.display_options.bind_group, &[]);
        display_render_pass.set_bind_group(1, &self.display.bind_group, &[]);
        display_render_pass.draw(0..6, 0..1);
        drop(display_render_pass);

        let (h_blur, v_blur) = (
            self.h_blur_texture.as_mut().unwrap(),
            self.v_blur_texture.as_mut().unwrap(),
        );
        // Don't apply glow if not enabled
        if self.final_options.glow_enabled > 0 {
            // Generate bind groups for uniforms to be used in blur passes'
            let mut blur_uniform_bind_groups = vec![];
            let radius = self.glow_radius / self.glow_iterations as f32;
            for i in 1..=self.glow_iterations {
                let mut buffer = *self.blur_options;
                // Calculate radius for blurring
                let blur_radius = ((self.glow_iterations - i) as f32) * radius;
                // First blur horizontally, then vertically
                buffer.direction = if i.is_multiple_of(2) {
                    [0.0, blur_radius]
                } else {
                    [blur_radius, 0.0]
                };
                // Copy uniform into a new wrapper and push generated bind group into list
                let uniform = UniformBuffer::from(buffer, &self.device, &format!("Blur {i}"));
                blur_uniform_bind_groups.push(uniform.bind_group);
            }

            // Run blur shader for iterations to blur the result of the display render pass onto a
            // texture
            for (i, uniform_bind_group) in blur_uniform_bind_groups.iter().enumerate() {
                // Choose texture view and texture bind group based on iteration count
                let (view, mut bind_group) = if i.is_multiple_of(2) {
                    (&v_blur.texture_view, &h_blur.bind_group)
                } else {
                    (&h_blur.texture_view, &v_blur.bind_group)
                };
                // On the first pass, read from the initial display texture
                if i == 1 {
                    bind_group = &self.display_texture.as_ref().unwrap().bind_group;
                }

                // Execute render pass
                let mut effect_render_pass =
                    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some(&format!("Effect Render Pass {i}")),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view,
                            resolve_target: None,
                            depth_slice: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });
                effect_render_pass.set_pipeline(&self.blur_render_pipeline);
                effect_render_pass.set_bind_group(0, Some(bind_group), &[]);
                effect_render_pass.set_bind_group(1, uniform_bind_group, &[]);
                effect_render_pass.draw(0..6, 0..1);
            }
        }

        // Combine display and blur textures onto a final image
        let output_texture = self.surface.get_current_texture()?;
        let output_view = output_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut final_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Final Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        final_render_pass.set_pipeline(&self.final_render_pipeline);
        final_render_pass.set_bind_group(
            0,
            &self.display_texture.as_ref().unwrap().bind_group,
            &[],
        );
        // Read final blur result from vertically blurred texture
        final_render_pass.set_bind_group(1, &v_blur.bind_group, &[]);
        final_render_pass.set_bind_group(2, &self.background_texture.bind_group, &[]);
        final_render_pass.set_bind_group(3, &self.final_options.bind_group, &[]);
        final_render_pass.draw(0..6, 0..1);
        drop(final_render_pass);
        self.queue.submit(std::iter::once(encoder.finish()));
        output_texture.present();

        self.window.request_redraw();
        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;

            // Update frame textures
            let texture_size = wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            };
            self.display_texture = Some(Texture::new(
                &self.device,
                &self.texture_bind_group_layout,
                &texture_size,
                "Display Texture",
            ));
            self.h_blur_texture = Some(Texture::new(
                &self.device,
                &self.texture_bind_group_layout,
                &texture_size,
                "Horizontal Blur Texture",
            ));
            self.v_blur_texture = Some(Texture::new(
                &self.device,
                &self.texture_bind_group_layout,
                &texture_size,
                "Vertical Blur Texture",
            ));

            let surface_size = Vector::new(Fp::from(width as i16), Fp::from(height as i16));
            let lcd_size = Vector::new(Fp::from(160), Fp::from(144));

            // If the on-screen controls should be shown, display is placed so part of the
            // background image containing the controls is fully visible.
            // Otherwise display is centered and scaled to fit
            // In both cases, the display is scaled so pixels have an even integer scale
            let (display_scale, display_origin, display_size) = if self.show_controls {
                // The rectangles are defined with the background image
                let controls_rect = self.background_definition.controls;
                let display_rect = self.background_definition.display;

                // Fit the rectangle containing controls onto the screen
                let controls_scale = (surface_size.x / controls_rect.size.x)
                    .min(surface_size.y / controls_rect.size.y);
                let fitted_controls_size = controls_rect.size * controls_scale;

                // Calculate size of display in fitted rectangle
                let rect_diff = controls_rect.size / display_rect.size;
                let fitted_display_size = fitted_controls_size / rect_diff;

                // Calculate scale at which the display pixel scale will be an integer
                let display_scale = fitted_display_size / lcd_size;
                let display_scale_rounded = display_scale.x.floor().min(display_scale.y.floor());
                let pixel_scale = display_scale_rounded * lcd_size / display_rect.size;

                // Calculate actual size of control rectangle
                let final_controls_size = display_scale_rounded * lcd_size * rect_diff;
                // Calculate the origin of the display
                let controls_origin = (surface_size - final_controls_size) / 2;
                let rect_pos_diff = display_rect.pos - controls_rect.pos;
                let display_origin = controls_origin + (rect_pos_diff * pixel_scale);

                (
                    display_scale_rounded,
                    display_origin,
                    lcd_size * display_scale_rounded,
                )
            } else {
                // Calculate pixel scale as the possible largest integer scale
                // which still fits display in both dimensions
                let scale = (surface_size.x / lcd_size.x)
                    .min(surface_size.y / lcd_size.y)
                    .floor();
                // Calculate size of the display
                let size = lcd_size * scale;
                // Calculate top-left origin in pixel space for centered canvas
                let origin = (surface_size - size) / 2;

                (scale, origin, size)
            };

            // Scale the background to match the display position and scale
            let background_scale = display_size / self.background_definition.display.size;
            let background_size = self.background_texture.size() * background_scale;
            let background_origin =
                display_origin - (self.background_definition.display.pos * background_scale);

            // Helper functions to convert fixed point vectors into arrays that can be passed into
            // shader
            fn vec_to_buffer_rounded(vector: &Vector) -> [i32; 2] {
                [vector.x.round().into(), vector.y.round().into()]
            }
            fn vec_to_buffer(vector: &Vector) -> [f32; 2] {
                [vector.x.into(), vector.y.into()]
            }

            // Update options
            self.background_rendered_rect = Rect::new(background_origin, background_size);
            self.display_options.scale = i16::from(display_scale) as u32;
            self.display_options.origin = vec_to_buffer_rounded(&display_origin);
            self.display_options.update_buffer(&self.queue);
            self.blur_options.resolution[0] = f32::from(surface_size.x / display_scale);
            self.blur_options.resolution[1] = f32::from(surface_size.y / display_scale);
            self.blur_options.update_buffer(&self.queue);
            self.final_options.display_origin = vec_to_buffer_rounded(&display_origin);
            self.final_options.display_size = vec_to_buffer_rounded(&display_size);
            self.final_options.background_origin = vec_to_buffer(&background_origin);
            self.final_options.background_size = vec_to_buffer(&background_size);
            self.final_options.viewport_size = [width, height];
            self.final_options.update_buffer(&self.queue);
        }
    }

    pub fn get_pos_in_background(&self, pos: Vector) -> Vector {
        let bg_rect = self.background_rendered_rect;
        let tx_size = self.background_texture.size();
        let uv = (pos - bg_rect.pos) / bg_rect.size;
        uv * tx_size
    }

    pub fn update_options(&mut self, options: &EmulatorOptions) {
        if self.show_controls != options.show_controls {
            self.show_controls = options.show_controls;
            self.resize(self.config.width, self.config.height);
        }
        self.display_options.palette = options.palette;
        self.display_options.scanline_strength = options.scanline_strength;
        self.display_options.scanline_size = options.scanline_size;
        self.final_options.glow_strength_display = options.display_glow_strength;
        self.final_options.glow_strength_background = options.background_glow_strength;
        self.final_options.ambient_light = options.ambient_light;
        self.final_options.glow_enabled = if options.glow_enabled { 1 } else { 0 };
        self.glow_iterations = options.glow_iterations;
        self.glow_radius = options.glow_radius;
        self.display_options.update_buffer(&self.queue);
        self.final_options.update_buffer(&self.queue);
    }

    pub fn update_display(&mut self, display: &DisplayBuffer) {
        self.display.buffer = *display;
        self.display.update_buffer(&self.queue);
    }
}
