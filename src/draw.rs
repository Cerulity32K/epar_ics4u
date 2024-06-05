use macroquad::{
    color::Color,
    math::{vec2, Vec2},
    miniquad::window::screen_size,
    shapes::draw_triangle,
    text::{draw_text, measure_text},
};

use crate::transform;

pub fn rotated_rect(center: Vec2, size: Vec2, rot: f32, color: impl Into<Color>) {
    let clr = color.into();
    let tl = transform::rotate(size * -0.5, rot) + center;
    let tr = transform::rotate(size * vec2(0.5, -0.5), rot) + center;
    let bl = transform::rotate(size * vec2(-0.5, 0.5), rot) + center;
    let br = transform::rotate(size * 0.5, rot) + center;
    draw_triangle(tl, tr, bl, clr);
    draw_triangle(br, tr, bl, clr);
}

pub fn draw_screen_centered_text(
    text: &str,
    x_from_center: f32,
    y_from_center: f32,
    font_size: u16,
    color: Color,
) {
    let center = Vec2::from(screen_size()) * 0.5;
    draw_centered_text(
        text,
        x_from_center + center.x,
        y_from_center + center.y,
        font_size,
        color,
    );
}
pub fn draw_centered_text(text: &str, x: f32, y: f32, font_size: u16, color: Color) {
    let dim = measure_text(text, None, font_size, 1.0);
    draw_text(
        text,
        x - dim.width * 0.5,
        y - dim.height * 0.5,
        font_size as f32,
        color,
    )
}
