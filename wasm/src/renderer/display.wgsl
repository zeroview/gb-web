/// Options for rendering
struct OptionsUniform {
    /// The colored palette to get the final drawn pixel color from
    palette: array<vec4<f32>, 4>,
    /// The scale of pixels
    scale: u32,
    pad: u32,
    /// The origin of the display in pixel space
    origin: vec2<i32>,
}

@group(0) @binding(0)
var<uniform> options: OptionsUniform;

// Contains the display pixel data for rendering.
// Structured as an array of 4D vectors to
// satisfy the memory alignment system.
// 1440 integers in 4D vectors = 360 vectors
struct DisplayUniform {
    buffer: array<vec4<u32>, 360>
};

@group(1) @binding(0)
var<uniform> display: DisplayUniform;

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var square_vertices = array<vec2<f32>, 6>(
        vec2(-1.0, -1.0),
        vec2(1.0, -1.0),
        vec2(-1.0, 1.0),
        vec2(1.0, 1.0),
        vec2(-1.0, 1.0),
        vec2(1.0, -1.0),
    );
    let vertex = square_vertices[in_vertex_index];

    var out: VertexOutput;
    out.pos = vec4f(vertex, 0.0, 1.0);
    return out;
}

fn get_pixel_color(pos: vec2<i32>, origin: vec2<i32>, scale: u32) -> vec4<f32> {
    // Crop out pixels on the top and left sides of display
    if pos.x < origin.x || pos.y < origin.y {
        return vec4f(0.0, 0.0, 0.0, 1.0);
    }
    let pixel = vec2u(pos - origin) / scale;
    // Crop out pixels on the bottom and right sides of display
    if pixel.x >= 160u || pixel.y >= 144u {
        return vec4f(0.0, 0.0, 0.0, 1.0);
    }
    // Calculate index of pixel on display
    let pixel_i = pixel.y * 160u + pixel.x;
    // Calculate index of the two color bits in display buffer
    let bit_i = 2u * pixel_i;
    // Calculate index of vector containing bit
    // (vec4 contains 128 bits of data)
    let vec_i = bit_i / 128u;
    // Calculate index of vector integer containing bit
    let vec_bit_i = bit_i % 128u;
    let int_i = vec_bit_i / 32u;
    // Calculate index of color bits inside integer 
    let int_bit_i = ((vec_bit_i % 32u) / 2u) * 2u;

    // Get integer value and bit mask the color value
    let int = display.buffer[vec_i][int_i];
    let color = (int >> int_bit_i) & 3u;
    // Return color from current palette
    return options.palette[color];
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return get_pixel_color(vec2i(in.pos.xy), options.origin, options.scale);
}

