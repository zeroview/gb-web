// Contains the display pixel data for rendering.
// Structured as an array of 4D vectors to
// satisfy the memory alignment system
struct PixelUniform {
  pixels: array<vec4<u32>, 360>
};

@group(0) @binding(0)
var<uniform> pixels: PixelUniform;

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> @builtin(position) vec4f {
    const pos = array(
      vec2(-1.0, -1.0),
      vec2( 1.0, -1.0),
      vec2(-1.0,  1.0),
      vec2( 1.0,  1.0),
      vec2(-1.0,  1.0),
      vec2( 1.0, -1.0),
    );
    return vec4f(pos[in_vertex_index], 0, 1);
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    const width: u32 = 160;
    const height: u32 = 144;
    const scale: f32 = 4.0;

    // Scale pixel coordinate down to make display bigger
    var x = u32(pos.x / scale);
    var y = u32(pos.y / scale);
    // Make outside pixels black
    if x > width || y > height {
      return vec4f(0.0, 0.0, 0.0, 1.0);
    }
    
    var pixel_i = y * width + x;
    var bit_i = 2 * pixel_i;
    var vec_i = bit_i / 128;
    var vec_bit_i = bit_i % 128;
    var int_i = vec_bit_i / 32;

    var int = pixels.pixels[vec_i][int_i];
    var int_col_i = ((vec_bit_i % 32) / 2) * 2;
    
    var color = int >> int_col_i & 3;
    if color == 0 {
      return vec4f(1.0, 1.0, 1.0, 1.0);
    }
    else {
      return vec4f(0.0, 0.0, 0.0, 1.0);
    }
}

