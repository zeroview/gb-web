/// Options for rendering
struct OptionsUniform {
    /// The colored palette to get the final drawn pixel color from
    palette: array<vec4<f32>, 4>,
    
    /// The brightness or strength of scanlines
    scanline_strength: f32,
    /// The size of the scanline
    scanline_size: f32,
    pad1: vec2<u32>,
  
    /// The origin of the display in pixel space
    origin: vec2<i32>,
    /// The scale of pixels
    scale: u32,
    pad2: u32,
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

fn get_pixel_color(pos: vec2<i32>) -> vec4<f32> {
    let origin = options.origin;
    let scale = options.scale;

    // Crop out pixels on the top and left sides of display
    if pos.x < origin.x || pos.y < origin.y {
        return vec4f(0.0);
    }
    let pixel = vec2u(pos - origin) / scale;
    // Crop out pixels on the bottom and right sides of display
    if pixel.x >= 160u || pixel.y >= 144u {
        return vec4f(0.0);
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

const PI = 3.14159265359;
fn scanline_sin(x: f32, freq: f32) -> f32 {
    // Return 0 when outside of the first period centered at 0
    if x > freq || x < -freq {
        return 0.0;
    }
    return (sin(PI * ((1.0 / freq) * x + (1.0 / 2.0))) + 1.0) / 2.0;
}

fn get_scanline_color(color: vec4<f32>, pos: vec2<i32>) -> vec4<f32> {
    let scale = f32(options.scale);
    let size = options.scanline_size;
    let strength = options.scanline_strength / 10.0;
    // Don't draw scanlines if pixel size is only one
    if scale == 1.0 {
        return color;
    }
    
    // Correct pos so the right and bottom sides show
    // the fully lit scanline
    var corrected_pos = pos;
    if pos.x == 160 * i32(options.scale) - 1 {
        corrected_pos.x += 1;
    }
    if pos.y == 144 * i32(options.scale) - 1 {
        corrected_pos.y += 1;
    }

    let pixel_pos = (vec2f(corrected_pos) % scale) / scale;
    // Calculate scanline light coming from all sides of the pixel
    var value = scanline_sin(pixel_pos.x, size);
    value += scanline_sin(1.0 - pixel_pos.x, size);
    value += scanline_sin(pixel_pos.y, size);
    value += scanline_sin(1.0 - pixel_pos.y, size);
    return color + vec4f(vec3f(value * strength), 0.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pos = vec2i(in.pos.xy);
    let color = get_pixel_color(pos);
    if color.a == 0.0 {
      discard;
    }
    return get_scanline_color(color, pos - options.origin);
}

