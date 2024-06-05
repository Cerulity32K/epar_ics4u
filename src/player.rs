use std::f64::NEG_INFINITY;

use macroquad::{
    color::{RED, SKYBLUE, WHITE},
    input::{is_key_down, is_key_pressed, KeyCode},
    math::{dvec2, DVec2, Vec2},
    miniquad::window::screen_size,
    shapes::draw_circle,
    window::{screen_height, screen_width},
};

use crate::{draw::draw_centered_text, level::Level, transform};

#[derive(Clone, Copy)]
pub struct Player {
    pub position: DVec2,
    pub radius: f64,
    pub last_time: f64,
    pub regular_speed: f64,
    pub dash_speed: f64,

    pub last_dash_time: f64,
    pub dash_lifetime: f64,
    pub dash_invincibility_lifetime: f64,

    pub last_hit_time: f64,
    pub stun_lifetime: f64,
    pub stun_velocity: DVec2,
    pub hit_cooldown: f64,
    pub hp: usize,

    single_frame_freeze: bool,
}
impl Player {
    pub fn new() -> Self {
        Self {
            position: Vec2::from(screen_size()).as_dvec2() * dvec2(0.25, 0.5),
            radius: 5.0,
            last_time: 0.0,
            regular_speed: 200.0,
            dash_speed: 1000.0,

            last_dash_time: 0.0,
            dash_lifetime: 0.4,
            dash_invincibility_lifetime: 0.35,

            last_hit_time: NEG_INFINITY,
            stun_lifetime: 0.2,
            stun_velocity: -DVec2::X * 500.0,
            hit_cooldown: 2.0,
            hp: 3,

            single_frame_freeze: true,
        }
    }
    pub fn speed(&mut self, time: f64) -> f64 {
        if self.last_dash_time + self.dash_lifetime > time {
            transform::lerp(
                self.dash_speed,
                self.regular_speed,
                (time - self.last_dash_time) / self.dash_lifetime,
            )
        } else {
            self.regular_speed
        }
    }
    pub fn update(&mut self, time: f64, beat: f64, level: &Level) -> bool {
        let dt = time - self.last_time;
        if self.single_frame_freeze {
            self.last_time = time;
            self.single_frame_freeze = false;
            return false;
        }

        if self.last_hit_time + self.stun_lifetime > time {
            self.position += self.stun_velocity * dt;
        } else {
            let before = self.position;
            if is_key_down(KeyCode::A) {
                self.position.x -= dt * self.speed(time);
            }
            if is_key_down(KeyCode::D) {
                self.position.x += dt * self.speed(time);
            }
            if is_key_down(KeyCode::W) {
                self.position.y -= dt * self.speed(time);
            }
            if is_key_down(KeyCode::S) {
                self.position.y += dt * self.speed(time);
            }
            let after = self.position;
            if after != before {
                self.stun_velocity = (before - after).normalize() * self.dash_speed * 0.5;
            }
            if is_key_pressed(KeyCode::Space) && self.last_dash_time + self.dash_lifetime < time {
                self.last_dash_time = time;
            }
        }

        let is_invincible = self.last_dash_time + self.dash_invincibility_lifetime > time;

        if self.last_hit_time + self.hit_cooldown < time {
            if !is_invincible && level.collide(&self, beat) {
                self.hp = self.hp.saturating_sub(1);
                println!("womp womp {}", self.hp);
                self.last_hit_time = time;
            }
        }

        self.position = self.position.clamp(
            DVec2::new(self.radius + 10.0, self.radius + 10.0),
            DVec2::new(
                screen_width() as f64 - (self.radius + 10.0),
                screen_height() as f64 - (self.radius + 10.0),
            ),
        );

        self.last_time = time;
        self.hp == 0
    }
    pub fn draw(&self, time: f64) {
        let mut color = SKYBLUE;
        if self.last_hit_time + self.hit_cooldown > time && time.rem_euclid(0.1) > 0.05 {
            color = RED;
        }
        draw_circle(
            self.position.x as f32,
            self.position.y as f32,
            self.radius as f32,
            color,
        );
        draw_centered_text(
            &format!("{}", self.hp),
            self.position.x as f32,
            self.position.y as f32,
            24,
            WHITE,
        );
    }
}
