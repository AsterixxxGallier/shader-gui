struct Uniforms {
    offset: vec2<f32>,
    pixel_size: vec2<f32>,
    size: vec2<u32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read_write> steps: array<u32>;

fn square(z: vec2<f32>) -> vec2<f32> {
    return vec2(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y);
}

fn abs_sqr(z: vec2<f32>) -> f32 {
    return z.x * z.x + z.y * z.y;
}

fn mandelbrot(c: vec2<f32>, iters: u32) -> u32 {
    var z = c;
    for (var i: u32 = 0; i < iters; i += 1) {
        z = square(z) + c;
        if abs_sqr(z) > 4.0 {
            return i;
        }
    }
    return iters;
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) pos: vec3<u32>) {
    if any(pos.xy >= uniforms.size) {
        return;
    }
    var uv = (vec2<f32>(pos.xy) - uniforms.offset) * uniforms.pixel_size;
    var c = (uv - vec2(1.0, 0.5)) * 3.0;
    var index = pos.x + pos.y * uniforms.size.x;
    steps[index] = mandelbrot(c, 1000);
}
