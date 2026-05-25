struct Uniforms {
    viewport_offset: vec2<f32>,
    buffer_size: vec2<u32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read> steps: array<u32>;

fn mandelbrot_color(i: u32, iters: u32) -> vec4<f32> {
    var t: f32;
    if i == 0 {
        t = 0;
    } else {
        t = log(f32(i)) / log(f32(iters));
    }
    return (1.0 - t) * vec4(0.0, 0.0, 0.0, 1.0) + t * vec4(1.0, 0.0, 0.0, 1.0);
}

@fragment
fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    var buffer_pos = vec2<u32>(pos.xy - uniforms.viewport_offset);
    var buffer_index = buffer_pos.x + buffer_pos.y * uniforms.buffer_size.x;
    var value = steps[buffer_index];
    return mandelbrot_color(value, 1000);
}
