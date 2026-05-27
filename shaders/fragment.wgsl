struct Uniforms {
    viewport_offset: vec2<f32>,
    buffer_size: vec2<u32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read> colors: array<vec3<f32>>;

fn color_at(pos: vec2<f32>) -> vec3<f32> {
    let buffer_pos = vec2<u32>(pos);
    let buffer_index = buffer_pos.x + buffer_pos.y * uniforms.buffer_size.x;
    return colors[buffer_index];
}

@fragment
fn main(@builtin(position) screen_pos: vec4<f32>) -> @location(0) vec4<f32> {
    let pos = screen_pos.xy - uniforms.viewport_offset;
    let color = color_at(pos);
//    var color = vec3(0.0);
//    if pos.y % 50.0 < 25.0 {
//        color = vec3(1.0, 0.4, 0.4);
//    } else {
//        color = vec3(0.4, 0.4, 1.0);
//    }
    return vec4(color, 1.0);
}
