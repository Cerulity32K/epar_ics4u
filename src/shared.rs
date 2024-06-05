use macroquad::{
    color::Color,
    math::{vec2, Vec2},
};

use crate::{obstacle::Obstacle, provider::Provider};

pub struct Shared {
    shake: f64,
    jerk: Vec2,
    new_obstacles: Vec<Obstacle>,
    pub new_background: Option<Box<dyn Provider<Color>>>,
    pub new_foreground: Option<Box<dyn Provider<Color>>>,
}
impl Shared {
    pub fn new() -> Shared {
        Self {
            shake: 0.0,
            jerk: vec2(0.0, 0.0),
            new_obstacles: vec![],
            new_background: None,
            new_foreground: None,
        }
    }
    pub fn add_jerk(&mut self, add: Vec2) -> &mut Self {
        self.jerk += add;
        self
    }
    pub fn add_shake(&mut self, add: f64) -> &mut Self {
        self.shake += add;
        self
    }
    pub fn add_obstacle(&mut self, add: Obstacle) -> &mut Self {
        self.new_obstacles.push(add);
        self
    }
    pub fn set_foreground(&mut self, to: Box<dyn Provider<Color>>) -> &mut Self {
        self.new_foreground = Some(to);
        self
    }
    pub fn set_background(&mut self, to: Box<dyn Provider<Color>>) -> &mut Self {
        self.new_background = Some(to);
        self
    }
    pub fn jerk(&self) -> Vec2 {
        self.jerk
    }
    pub fn shake(&self) -> f64 {
        self.shake
    }
    pub fn consume_for_obstacles(self) -> Vec<Obstacle> {
        self.new_obstacles
    }
}
