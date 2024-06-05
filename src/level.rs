use std::mem;

use jut::extensions::Boxed;
use macroquad::{
    color::{Color, BLACK, RED},
    math::{dvec2, Vec2},
    shapes::draw_rectangle,
    window::{screen_height, screen_width},
};

use crate::{
    ext::ColorExt,
    music::Music,
    obstacle::Obstacle,
    player::Player,
    provider::{FnWrap, Provider},
    shared::Shared,
    transform,
};

pub struct LevelBuilder {
    obstacles: Vec<Obstacle>,
}
impl LevelBuilder {
    pub fn new() -> Self {
        Self { obstacles: vec![] }
    }
    pub fn obstacle(&mut self, add: Obstacle) -> &mut Self {
        self.obstacles.push(add);
        self
    }
    pub fn pop_last_obstacle(&mut self) -> Option<Obstacle> {
        self.obstacles.pop()
    }
    pub fn build(
        self,
        song_data: &'static [u8],
        bpm: f64,
        start_time: f64,
        checkpoints: &'static [f64],
    ) -> Level {
        Level {
            obstacles: self.obstacles,

            shake: 0.0,
            jerk: Vec2::ZERO,

            foreground_color: FnWrap(|_| Color {
                r: 1.0,
                g: 0.0,
                b: 0.5,
                a: 1.0,
            })
            .boxed(),
            background_color: FnWrap(|_| BLACK).boxed(),

            last_beat: 0.0,

            song_data,
            bpm,
            start_time,

            checkpoints,
        }
    }
}

pub struct Level {
    pub shake: f64,
    pub jerk: Vec2,
    obstacles: Vec<Obstacle>,

    foreground_color: Box<dyn Provider<Color>>,
    background_color: Box<dyn Provider<Color>>,

    last_beat: f64,

    pub song_data: &'static [u8],
    pub bpm: f64,
    pub start_time: f64,

    pub checkpoints: &'static [f64],
}
impl Level {
    pub fn update(&mut self, beat: f64) {
        let dt = beat - self.last_beat;
        self.shake = transform::time_independent_lerp(self.shake, 0.0, 0.1, dt);
        self.jerk = transform::time_independent_vec2_lerp(self.jerk, Vec2::ZERO, 0.1, dt);
        let mut shared = Shared::new();
        for i in &mut self.obstacles {
            i.update(&mut shared, beat);
        }
        let mut i = 0;
        while i < self.obstacles.len() {
            if self.obstacles[i].should_kill(beat) {
                self.obstacles[i].kill(&mut shared, beat);
                self.obstacles.swap_remove(i);
            } else {
                i += 1;
            }
        }
        self.shake += shared.shake();
        self.jerk += shared.jerk();
        if let Some(new_background) = mem::take(&mut shared.new_background) {
            self.background_color = new_background;
        }
        if let Some(new_foreground) = mem::take(&mut shared.new_foreground) {
            self.foreground_color = new_foreground;
        }
        for mut i in shared.consume_for_obstacles() {
            i.offset += beat;
            self.obstacles.push(i);
        }
        self.last_beat = beat;
    }
    pub fn collide(&self, player: &Player, beat: f64) -> bool {
        for i in &self.obstacles {
            if i.collides(beat, player.position, player.radius) {
                return true;
            }
        }
        false
    }
    pub fn background_color(&self, beat: f64) -> Color {
        self.background_color.get(beat)
    }
    pub fn draw(&mut self, beat: f64) {
        for i in &self.obstacles {
            i.draw(self.foreground_color.get(beat), beat);
        }
    }
    #[allow(dead_code)]
    pub fn shade_collisions(&mut self, player: &Player, beat: f64) {
        let mut player = player.clone();

        let horizontal_resolution = 100;
        let vertical_resolution = 100;

        let fragment_width = screen_width() / horizontal_resolution as f32;
        let fragment_height = screen_height() / vertical_resolution as f32;

        for xi in 0..horizontal_resolution {
            let x = xi as f32 * fragment_width;
            for yi in 0..vertical_resolution {
                let y = yi as f32 * fragment_height;
                player.position = dvec2(
                    (x + fragment_width * 0.5) as f64,
                    (y + fragment_height * 0.5) as f64,
                );
                if self.collide(&player, beat) {
                    draw_rectangle(x, y, fragment_width, fragment_height, RED.faded());
                }
            }
        }
    }
    /// Updates and kills objects as needed. Killed objects can only change colors.
    pub fn update_to(&mut self, music: &Music) {
        let mut shared = Shared::new();
        let beat = music.beat();
        for i in &mut self.obstacles {
            i.update(&mut shared, beat);
        }
        let mut i = 0;
        while i < self.obstacles.len() {
            if self.obstacles[i].should_kill(beat) {
                self.obstacles[i].kill(&mut shared, beat);
                self.obstacles.swap_remove(i);
            } else {
                i += 1;
            }
        }
        if let Some(bg) = mem::take(&mut shared.new_background) {
            self.background_color = bg;
        }
        if let Some(fg) = mem::take(&mut shared.new_foreground) {
            self.foreground_color = fg;
        }
    }
}
