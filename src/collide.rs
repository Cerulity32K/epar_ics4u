use macroquad::math::{dvec2, DVec2};

use crate::transform;

pub const CIRCLE_RECTANGLE_COLLISION_MAP: [fn(DVec2, DVec2, DVec2, f64) -> bool; 9] = [
    |posr: DVec2, _sze: DVec2, posc: DVec2, rad: f64| posr.distance_squared(posc) < rad * rad, // top left
    |posr: DVec2, _sze: DVec2, posc: DVec2, rad: f64| posr.y < posc.y + rad, // top center
    |posr: DVec2, size: DVec2, posc: DVec2, rad: f64| {
        (posr + dvec2(size.x, 0.0)).distance_squared(posc) < rad * rad
    }, // top right
    |posr: DVec2, _sze: DVec2, posc: DVec2, rad: f64| posr.x < posc.x + rad, // center left
    |_psr: DVec2, _sze: DVec2, _psc: DVec2, _rd: f64| true,                  // center center
    |posr: DVec2, size: DVec2, posc: DVec2, rad: f64| posr.x + size.x > posc.x - rad, // center right
    |posr: DVec2, size: DVec2, posc: DVec2, rad: f64| {
        (posr + dvec2(0.0, size.y)).distance_squared(posc) < rad * rad
    }, // bottom left
    |posr: DVec2, size: DVec2, posc: DVec2, rad: f64| posr.y + size.y > posc.y - rad, // bottom center
    |posr: DVec2, size: DVec2, posc: DVec2, rad: f64| {
        (posr + size).distance_squared(posc) < rad * rad
    }, // bottom right
];

pub fn circle_circle(
    circle1_pos: DVec2,
    circle1_radius: f64,
    circle2_pos: DVec2,
    circle2_radius: f64,
) -> bool {
    let radius_sum = circle1_radius + circle2_radius;
    (circle1_pos - circle2_pos).length_squared() <= (radius_sum * radius_sum)
}
pub fn circle_aabb(
    circle_pos: DVec2,
    circle_radius: f64,
    aabb_topleft: DVec2,
    aabb_size: DVec2,
) -> bool {
    let mut col_idx = 0;
    if circle_pos.x > aabb_topleft.x {
        col_idx += 1;
        if circle_pos.x >= aabb_topleft.x + aabb_size.x {
            col_idx += 1;
        }
    }
    if circle_pos.y > aabb_topleft.y {
        col_idx += 3;
        if circle_pos.y >= aabb_topleft.y + aabb_size.y {
            col_idx += 3;
        }
    }
    CIRCLE_RECTANGLE_COLLISION_MAP[col_idx](aabb_topleft, aabb_size, circle_pos, circle_radius)
}
pub fn circle_rectangle(
    circle_pos: DVec2,
    circle_radius: f64,
    rectangle_center: DVec2,
    rectangle_size: DVec2,
    rectangle_rotation: f64,
) -> bool {
    let localized_circle_pos =
        transform::rotate_around(circle_pos, rectangle_center, rectangle_rotation);
    circle_aabb(
        localized_circle_pos,
        circle_radius,
        rectangle_center - rectangle_size * 0.5,
        rectangle_size,
    )
}
pub fn circle_line(
    circle_pos: DVec2,
    circle_radius: f64,
    line_start: DVec2,
    line_end: DVec2,
    line_thickness: f64,
) -> bool {
    let (center, size, rot) = transform::rectify_line(line_start, line_end, line_thickness);
    circle_rectangle(circle_pos, circle_radius, center, size, rot)
}
