@group(0) @binding(0)
var frame_texture: texture_2d<f32>;
@group(0) @binding(1)
var frame_sampler: sampler;

struct Options {
    direction: vec2<f32>,
    resolution: vec2<f32>
}

@group(1) @binding(0)
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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = vec4(0.0);
    let off1 = vec2f(1.3846153846) * options.direction;
    let off2 = vec2f(3.2307692308) * options.direction;
    // Blur the color by sampling texture at different points with weights
    color += textureSample(frame_texture, frame_sampler, in.uv) * 0.2270270270;
    color += textureSample(frame_texture, frame_sampler, in.uv + (off1 / options.resolution)) * 0.3162162162;
    color += textureSample(frame_texture, frame_sampler, in.uv - (off1 / options.resolution)) * 0.3162162162;
    color += textureSample(frame_texture, frame_sampler, in.uv + (off2 / options.resolution)) * 0.0702702703;
    color += textureSample(frame_texture, frame_sampler, in.uv - (off2 / options.resolution)) * 0.0702702703;

    return color;
}
