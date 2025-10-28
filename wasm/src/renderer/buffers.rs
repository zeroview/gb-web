use super::*;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DisplayOptionsUniform {
    pub palette: Palette,
    pub scale: u32,
    _pad: u32,
    pub origin: [i32; 2],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
// Uniform of display data
pub struct DisplayBufferUniform {
    pub buffer: DisplayBuffer,
}

impl Default for DisplayBufferUniform {
    fn default() -> Self {
        Self {
            buffer: [0; DISPLAY_BUFFER_SIZE],
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlurOptionsUniform {
    pub direction: [f32; 2],
    pub resolution: [f32; 2],
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FinalOptionsUniform {
    pub glow_strength_display: f32,
    pub glow_strength_background: f32,
    pub ambient_light: f32,
    _pad: u32,
    pub display_origin: [i32; 2],
    pub display_size: [u32; 2],
    pub background_display_origin: [u32; 2],
    pub background_display_size: [u32; 2],
    pub viewport_size: [u32; 2],
    _pad2: [u32; 2],
}

#[derive(Debug)]
pub struct UniformBuffer<U> {
    uniform: U,
    buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl<U> UniformBuffer<U>
where
    U: bytemuck::NoUninit + Default,
{
    pub fn new(device: &wgpu::Device, name: &str) -> Self {
        Self::from(U::default(), device, name)
    }

    pub fn from(uniform: U, device: &wgpu::Device, name: &str) -> Self {
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
        Self {
            uniform,
            buffer,
            bind_group,
            bind_group_layout,
        }
    }

    pub fn update_buffer(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }
}

impl<U> std::ops::Deref for UniformBuffer<U> {
    type Target = U;

    fn deref(&self) -> &Self::Target {
        &self.uniform
    }
}

impl<U> std::ops::DerefMut for UniformBuffer<U> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.uniform
    }
}

#[derive(Debug)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
}

impl Texture {
    pub fn new(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        size: &wgpu::Extent3d,
        name: &str,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("{name} Texture")),
            size: *size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some(&format!("{name} Bind Group")),
        });

        Self {
            texture,
            texture_view,
            bind_group,
        }
    }
}
