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
var<storage, read> complex: array<vec2<f32>>;

@group(0) @binding(2)
var<storage, read_write> polar: array<vec2<f32>>;

fn angle(v: vec2<f32>) -> f32 {
    return atan2(v.y, v.x);
}

fn to_polar(v: vec2<f32>) -> vec2<f32> {
    return vec2(length(v), angle(v));
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) pos: vec3<u32>) {
    if any(pos.xy >= uniforms.size) {
        return;
    }
    var index = pos.x + pos.y * uniforms.size.x;
    polar[index] = to_polar(complex[index]);
}
