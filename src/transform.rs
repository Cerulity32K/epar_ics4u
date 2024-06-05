use macroquad::math::{dvec2, DVec2, Vec2};

/// Rotates a vector around (0, 0) using radians.
pub fn rotate_d(vec: DVec2, rot: f64) -> DVec2 {
    // literally matrix multiplication
    DVec2 {
        x: rot.cos() * vec.x + rot.sin() * vec.y,
        y: -rot.sin() * vec.x + rot.cos() * vec.y,
    }
}
/// Rotates a vector around (0, 0) using radians.
pub fn rotate(vec: Vec2, rot: f32) -> Vec2 {
    // literally matrix multiplication
    Vec2 {
        x: rot.cos() * vec.x + rot.sin() * vec.y,
        y: -rot.sin() * vec.x + rot.cos() * vec.y,
    }
}
pub fn rotate_around(vec: DVec2, around: DVec2, rot: f64) -> DVec2 {
    rotate_d(vec - around, rot) + around
}
pub fn rectify_line(start: DVec2, end: DVec2, thickness: f64) -> (DVec2, DVec2, f64) {
    let delta = end - start;
    (
        start.lerp(end, 0.5),                  // Position (center)
        dvec2(start.distance(end), thickness), // Size
        delta.y.atan2(delta.x),                // Rotation
    )
}
pub fn lerp(start: f64, end: f64, factor: f64) -> f64 {
    start + (end - start) * factor
}
pub fn time_independent_lerp(start: f64, end: f64, factor_per_second: f64, delta_time: f64) -> f64 {
    let factor = factor_per_second.powf(delta_time);
    lerp(end, start, factor)
}
pub fn time_independent_vec2_lerp(
    start: Vec2,
    end: Vec2,
    factor_per_second: f64,
    delta_time: f64,
) -> Vec2 {
    let factor = factor_per_second.powf(delta_time);
    start.lerp(end, factor as f32)
}
