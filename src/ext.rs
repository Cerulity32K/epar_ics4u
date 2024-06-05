use macroquad::color::Color;

use crate::transform;

pub trait ColorExt {
    fn faded(&self) -> Self;
    fn mix(&self, other: Self, factor: f64) -> Self;
}
pub const FADE_FACTOR: f32 = 0.25;
impl ColorExt for Color {
    fn faded(&self) -> Self {
        Self {
            a: self.a * FADE_FACTOR,
            ..*self
        }
    }
    fn mix(&self, other: Self, factor: f64) -> Self {
        Self {
            r: transform::lerp(self.r as f64, other.r as f64, factor) as f32,
            g: transform::lerp(self.g as f64, other.g as f64, factor) as f32,
            b: transform::lerp(self.b as f64, other.b as f64, factor) as f32,
            a: transform::lerp(self.a as f64, other.a as f64, factor) as f32,
        }
    }
}
