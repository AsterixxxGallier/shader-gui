#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ValueType {
    // WGSL `f32`, unbounded
    Real,
    // WGSL `vec2<f32>`, unbounded, `x` = real component, `y` = imaginary component
    Complex,
    // WGSL `vec2<f32>`, `x` = magnitude (unbounded), `y` = angle (same bounds as Angle)
    Polar,
    // WGSL `f32`, between zero (inclusive) and tau (exclusive)
    Angle,
    // WGSL `f32`, between zero and one (both inclusive)
    Proportion,
    // WGSL `vec3<f32>`, `x` = red, `y` = green, `z` = blue,
    //                   each channel between zero and one (both inclusive),
    //                   linear color space (passed as-is to egui's render pass)
    RgbColor,
    // WGSL `vec4<f32>`, `x` = red, `y` = green, `z` = blue, `w` = alpha,
    //                   each channel between zero and one (both inclusive),
    //                   linear color space (passed as-is to egui's render pass)
    RgbaColor,
}

pub enum Value {
    Real(f32),
    Complex(f32, f32),
    Polar(f32, f32),
    Angle(f32),
    Proportion(f32),
    RgbColor(f32, f32, f32),
    RgbaColor(f32, f32, f32, f32),
}

impl ValueType {
    pub fn size(self) -> usize {
        match self {
            ValueType::Real => 4,
            ValueType::Complex => 8,
            ValueType::Polar => 8,
            ValueType::Angle => 4,
            ValueType::Proportion => 4,
            ValueType::RgbColor => 12,
            ValueType::RgbaColor => 16,
        }
    }

    pub fn align(self) -> usize {
        match self {
            ValueType::Real => 4,
            ValueType::Complex => 8,
            ValueType::Polar => 8,
            ValueType::Angle => 4,
            ValueType::Proportion => 4,
            ValueType::RgbColor => 16,
            ValueType::RgbaColor => 16,
        }
    }

    pub fn default_value(self) -> Value {
        match self {
            ValueType::Real => Value::Real(0.0),
            ValueType::Complex => Value::Complex(0.0, 0.0),
            ValueType::Polar => Value::Polar(0.0, 0.0),
            ValueType::Angle => Value::Angle(0.0),
            ValueType::Proportion => Value::Proportion(0.0),
            ValueType::RgbColor => Value::RgbColor(0.0, 0.0, 0.0),
            ValueType::RgbaColor => Value::RgbaColor(0.0, 0.0, 0.0, 0.0),
        }
    }
}