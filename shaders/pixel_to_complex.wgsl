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
var<storage, read_write> values: array<vec2<f32>>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) pos: vec3<u32>) {
    if any(pos.xy >= uniforms.size) {
        return;
    }
    var uv = (vec2<f32>(pos.xy) - uniforms.offset) * uniforms.pixel_size;
    var c = (uv - vec2(0.5 * uniforms.aspect_ratio, 0.5)) * 3.0;
    var index = pos.x + pos.y * uniforms.size.x;
    values[index] = c;
}
