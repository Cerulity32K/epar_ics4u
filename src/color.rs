use std::f32::consts::TAU;

use macroquad::color::Color;

pub fn sinebow(phase: f32) -> Color {
    Color {
        r: (phase + TAU / 3.0 * 3.0).sin() / 2.0 + 0.5,
        g: (phase + TAU / 3.0 * 2.0).sin() / 2.0 + 0.5,
        b: (phase + TAU / 3.0 * 1.0).sin() / 2.0 + 0.5,
        a: 1.0,
    }
}
