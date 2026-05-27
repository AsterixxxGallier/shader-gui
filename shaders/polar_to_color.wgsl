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
var<storage, read> polar: array<vec2<f32>>;

@group(0) @binding(2)
var<storage, read_write> colors: array<vec3<f32>>;

fn length_squared(z: vec2<f32>) -> f32 {
    return z.x * z.x + z.y * z.y;
}

fn sigmoid(value: f32) -> f32 {
    return 1.0 - exp(-value);
}

// Value ranges:
// - hue: from 0.0 to 12.0 (12.0 is equivalent to 360° or 2 pi)
// - saturation: from 0.0 to 1.0
// - lightness: from 0.0 to 1.0
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> vec3<f32> {
    let a = s * min(l, 1.0 - l);

    const n = vec3(0.0, 8.0, 4.0);
    let x = n + vec3(h);
    let k = x - 12.0 * floor(x * (1.0 / 12.0));
    let rgb = vec3(l) - a * clamp(min(k - 3.0, 9.0 - k), vec3(-1.0), vec3(1.0));
    return rgb;
}

const PI = 3.14159265358979323846264338327950288;

fn polar_color(z: vec2<f32>) -> vec3<f32> {
    const NAN_COLOR = vec3(1.0, 0.0, 1.0);
    const INFINITY_COLOR = vec3(1.0, 0.0, 0.0);

    if z.x != z.x || z.y != z.y {
        return NAN_COLOR;
    }

    let norm = z.x;
    let hue = z.y / PI * -6.0;
    let lightness = sigmoid(norm);
    let base_color = hsl_to_rgb(hue, 0.5, lightness);

     let t = min(log(norm) / log(100.0), 1.0);
     return t * INFINITY_COLOR + (1.0 - t) * base_color;
//    return base_color;
}

fn to_srgb(c: vec3<f32>) -> vec3<f32> {
    return select(pow((c + 0.055) * (1.0 / 1.055), vec3(2.4)), c * (1.0 / 12.92), c <= vec3(0.04045));
}

fn from_srgb(c: vec3<f32>) -> vec3<f32> {
    return select(pow(c, vec3(1.0 / 2.4)) * 1.055 - vec3(0.055), c * 12.92, c <= vec3(0.0031308));
}

fn from_angle(angle: f32) -> vec2<f32> {
    return vec2(cos(angle), sin(angle));
}

fn angle(v: vec2<f32>) -> f32 {
    return atan2(v.y, v.x);
}

fn rect_min(rect_size: vec2<f32>, pos: vec2<f32>) -> vec2<f32> {
    return floor(pos / rect_size) * rect_size;
}

fn needle(
    rect_size: vec2<f32>,
    needle_length: f32,
    needle_thickness: f32,
    pos: vec2<f32>,
    direction: f32,
) -> vec4<f32> {
    let rect_min = rect_min(rect_size, pos);
    let rect_center = rect_min + rect_size * 0.5;
    let within_radius = length_squared(rect_center - pos) <= needle_length * needle_length;
    if within_radius {
        let needle_delta = from_angle(direction) * needle_length;

        let needle_start = rect_center - needle_delta * 0.5;
        let start_to_point = pos - needle_start;

        let projection_t = dot(start_to_point, needle_delta) / length_squared(needle_delta);
        let projection = needle_start + projection_t * needle_delta;

        let distance_to_needle = distance(projection, pos);

        let effective_needle_thickness = mix(needle_thickness * 1.5, needle_thickness * 0.5, projection_t) * 2.0;

        if distance_to_needle <= effective_needle_thickness {
            let ratio = distance_to_needle / effective_needle_thickness;
            return vec4(0.0, 0.0, 0.0, (1.0 - ratio * ratio) * 0.5);
        } else {
            return vec4(0.0, 0.0, 0.0, 0.0);
        }
    } else {
        return vec4(0.0, 0.0, 0.0, 0.0);
    }
}

fn polar_at(pos: vec2<f32>) -> vec2<f32> {
    let buffer_pos = vec2<u32>(pos);
    let buffer_index = buffer_pos.x + buffer_pos.y * uniforms.size.x;
    return polar[buffer_index];
}

fn overlay(below: vec3<f32>, above: vec4<f32>) -> vec3<f32> {
    return below * (1.0 - above.w) + above.xyz * above.w;
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) index_pos: vec3<u32>) {
    if any(index_pos.xy >= uniforms.size) {
        return;
    }

    let pos = vec2<f32>(index_pos.xy);

    let rect_size = vec2(50.0);
    let needle_length = 10.0;
    let needle_thickness = 1.0;
    let rect_min = rect_min(rect_size, pos);
    let rect_center = rect_min + rect_size * 0.5;
    let center_value = polar_at(rect_center);
    let needle_color = needle(rect_size, needle_length, needle_thickness, pos, center_value.y);

    let index = index_pos.x + index_pos.y * uniforms.size.x;
    let value = polar[index];
//    colors[index] = from_srgb(overlay(polar_color(value), needle_color));
    colors[index] = from_srgb(polar_color(value));
}
