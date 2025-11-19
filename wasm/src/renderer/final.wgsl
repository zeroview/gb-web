@group(0) @binding(0)
var display_texture: texture_2d<f32>;
@group(0) @binding(1)
var display_sampler: sampler;
@group(1) @binding(0)
var blur_texture: texture_2d<f32>;
@group(1) @binding(1)
var blur_sampler: sampler;
@group(2) @binding(0)
var background_texture: texture_2d<f32>;
@group(2) @binding(1)
var background_sampler: sampler;

struct Options {
    glow_enabled: u32,
    glow_strength_display: f32,
    glow_strength_background: f32,
    ambient_light: f32,
    display_origin: vec2<i32>,
    display_size: vec2<u32>,
    background_display_origin: vec2<u32>,
    background_display_size: vec2<u32>,
    viewport_size: vec2<u32>,
    pad2: vec2<u32>,
}

@group(3) @binding(0)
var<uniform> options: Options;


struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    // List of vertices that form a full clip space quad
    var square_vertices = array<vec2<f32>, 6>(
        vec2(-1.0, -1.0),
        vec2(1.0, -1.0),
        vec2(-1.0, 1.0),
        vec2(1.0, 1.0),
        vec2(-1.0, 1.0),
        vec2(1.0, -1.0),
    );
    let vertex = square_vertices[in_vertex_index];
    var uv = (vertex + 1.0) / 2.0;
    uv.y = 1.0 - uv.y;

    var out: VertexOutput;
    out.uv = uv;
    out.pos = vec4f(vertex, 0.0, 1.0);
    return out;
}

fn sample_background(pos: vec2<u32>) -> vec4<f32> {
    // Calculate offset from display origin
    let offset_from_display = vec2f(pos) - vec2f(options.display_origin);
    // Scale offset to texture pixels
    let offset_in_texture = offset_from_display * (vec2f(options.background_display_size) / vec2f(options.display_size));
    // Calculate pixel position in texture
    let texture_pixel = vec2f(options.background_display_origin) + offset_in_texture;
    // Scale to UV
    let texture_uv = texture_pixel / vec2f(textureDimensions(background_texture));
    // Sample background texture
    return textureSample(background_texture, background_sampler, texture_uv);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pos = vec2u(in.pos.xy);
    // Calculate display bounds
    let display_min = vec2u(max(options.display_origin, vec2i(0)));
    let display_max = display_min + options.display_size - 1u;

    var color = vec4f(0.0);
    var glow_strength = 0.0;
    if pos.x < display_min.x || pos.x > display_max.x || pos.y < display_min.y || pos.y > display_max.y {
        // If nothing needs to be drawn on the background, discard fragment
        if options.ambient_light == 0.0 && options.glow_enabled == 0u {
          discard;
        }
        // Sample background with brightness
        color = sample_background(pos) * options.ambient_light;
        glow_strength = options.glow_strength_background;
    } else {
        // Sample display texture
        color = textureSample(display_texture, display_sampler, in.uv);
        glow_strength = options.glow_strength_display;
    }
    // Apply glow if enabled
    if options.glow_enabled > 0u {
        let glow = textureSample(blur_texture, blur_sampler, in.uv);
        color += (glow * glow_strength);
    }
    return color;
}

