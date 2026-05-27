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
var<storage, read_write> colors: array<vec3<f32>>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) index_pos: vec3<u32>) {
    if any(index_pos.xy >= uniforms.size) {
        return;
    }

    let index = index_pos.x + index_pos.y * uniforms.size.x;
    colors[index] = vec3(0.5, 0.8, 0.2);
}
