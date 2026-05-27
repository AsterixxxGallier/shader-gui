struct Uniforms {
    offset: vec2<f32>,
    pixel_size: f32,
    aspect_ratio: f32,
    size: vec2<u32>,
    cursor_position: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read> input: array<vec2<f32>>;

@group(0) @binding(2)
var<storage, read_write> output: array<vec2<f32>>;

fn square(z: vec2<f32>) -> vec2<f32> {
    return vec2(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y);
}

fn length_squared(z: vec2<f32>) -> f32 {
    return z.x * z.x + z.y * z.y;
}

fn mandelbrot_(c: vec2<f32>, iters: u32) -> vec2<f32> {
    var z = c;
    for (var i: u32 = 0; i < iters; i += 1) {
        z = square(z) + c;
        if length_squared(z) > 4.0 {
            return z;
        }
    }
    return z;
}

fn mul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}

fn conj(a: vec2<f32>) -> vec2<f32> {
    return vec2(a.x, -a.y);
}

fn recip(a: vec2<f32>) -> vec2<f32> {
    return conj(a) / length_squared(a);
}

fn angle(v: vec2<f32>) -> f32 {
    return atan2(v.y, v.x);
}

fn ln(a: vec2<f32>) -> vec2<f32> {
    return vec2(log(length(a)), angle(a));
}

fn _exp(a: vec2<f32>) -> vec2<f32> {
    return vec2(cos(a.y), sin(a.y)) * exp(a.x);
}

fn pow(base: vec2<f32>, exponent: vec2<f32>) -> vec2<f32> {
    if all(exponent == vec2(0.0, 0.0)) {
        return vec2(1.0, 0.0);
    }
    return _exp(mul(exponent, ln(base)));
}

fn mandelbrot(z0: vec2f, c: vec2f, e: vec2f, iters: u32) -> vec2f {
    var z = z0;
    for (var i = 0u; i < iters; i += 1) {
        z = pow(z, e) + c;
//        if length_squared(z) > 4.0 {
//            return z;
//        }
    }
    return z;
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) pos: vec3<u32>) {
    if any(pos.xy >= uniforms.size) {
        return;
    }
    var uv = (uniforms.cursor_position - uniforms.offset) * uniforms.pixel_size;
    var c = (uv - vec2(0.5 * uniforms.aspect_ratio, 0.5)) * 3.0;
    let index = pos.x + pos.y * uniforms.size.x;
    let z = input[index];
    output[index] = mandelbrot(vec2(0.2, -0.5), c, z, 50);
}
