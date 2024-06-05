use std::f64::consts::TAU as TAU64;

use jut::extensions::Boxed;
use macroquad::{
    color::{Color, WHITE},
    math::{dvec2, DVec2},
    shapes::draw_circle,
    window::{screen_height, screen_width},
};

use crate::{
    collide, draw,
    ext::ColorExt,
    provider::{Constant, Provider, Velocity},
    shared::Shared,
};

pub struct Obstacle {
    pub offset: f64,
    pub behaviour: DynObstacleBehaviour,
}
impl Obstacle {
    pub fn new(offset: f64, behaviour: DynObstacleBehaviour) -> Self {
        Self { offset, behaviour }
    }
    pub fn update(&mut self, shared: &mut Shared, beat: f64) {
        if self.should_enable(beat) {
            self.behaviour.update(shared, beat - self.offset);
        }
    }
    pub fn should_enable(&self, beat: f64) -> bool {
        self.behaviour.should_enable(beat - self.offset)
    }
    pub fn draw(&self, main_color: Color, beat: f64) {
        if self.should_enable(beat) {
            self.behaviour.draw(main_color, beat - self.offset);
        }
    }
    pub fn collides(&self, beat: f64, player_position: DVec2, player_radius: f64) -> bool {
        self.should_enable(beat)
            && self
                .behaviour
                .collides(beat - self.offset, player_position, player_radius)
    }
    pub fn should_kill(&self, beat: f64) -> bool {
        self.behaviour.should_kill(beat - self.offset)
    }
    pub fn kill(&mut self, shared: &mut Shared, beat: f64) {
        self.behaviour.kill(shared, beat - self.offset);
    }
}
impl Clone for Obstacle {
    fn clone(&self) -> Self {
        Self {
            behaviour: self.behaviour.box_clone(),
            ..*self
        }
    }
}

pub type DynObstacleBehaviour = Box<dyn ObstacleBehaviour + 'static>;
/// The behaviour of an Obstacle.
#[allow(unused_variables)]
pub trait ObstacleBehaviour {
    fn update(&mut self, shared: &mut Shared, beat: f64) {}
    fn draw(&self, main_color: Color, beat: f64);
    fn collides(&self, beat: f64, circle_pos: DVec2, circle_radius: f64) -> bool {
        false
    }
    fn box_clone(&self) -> DynObstacleBehaviour;
    fn should_enable(&self, beat: f64) -> bool {
        beat > 0.0
    }
    fn should_kill(&self, beat: f64) -> bool;
    fn kill(&mut self, shared: &mut Shared, beat: f64) {}
}
macro_rules! builder {
    ($ident:tt: $type:ty) => {
        #[allow(dead_code)]
        pub fn $ident(mut self, new_val: $type) -> Self {
            self.$ident = new_val;
            return self;
        }
    };
}
pub mod lasers {
    use std::f64::consts::PI;

    use macroquad::{
        color::{Color, WHITE},
        math::{DVec2, Vec2},
        shapes::draw_line,
    };

    use crate::{collide, ext::ColorExt, shared::Shared};

    use super::{DynObstacleBehaviour, ObstacleBehaviour};

    #[derive(Clone, Copy)]
    pub struct SlamLaser {
        pub start_pos: DVec2,
        pub end_pos: DVec2,

        pub warn_time: f64,
        pub lifetime: f64,
        pub leave_time: f64,
        pub thickness: f64,
        pub flash_time: f64,

        pub shake: f64,
        pub jerk: Vec2,
        pub slam_done: bool,
    }
    impl Default for SlamLaser {
        fn default() -> Self {
            Self {
                start_pos: DVec2::ZERO,
                end_pos: DVec2::ZERO,
                warn_time: 2.0,
                lifetime: 2.0,
                leave_time: 1.0,
                thickness: 50.0,
                flash_time: 0.5,
                shake: 0.0,
                jerk: Vec2::ZERO,
                slam_done: false,
            }
        }
    }
    impl SlamLaser {
        builder!(start_pos: DVec2);
        builder!(end_pos: DVec2);
        builder!(warn_time: f64);
        builder!(lifetime: f64);
        builder!(leave_time: f64);
        builder!(thickness: f64);
        builder!(flash_time: f64);
        builder!(shake: f64);
        builder!(jerk: Vec2);

        pub fn lerp_factor(&self, beat: f64) -> f64 {
            if beat < 0.0 {
                (beat / self.warn_time + 1.0) * 0.2
            } else if beat < self.lifetime {
                1.0
            } else {
                let normalized = (beat - self.lifetime) / self.leave_time;
                1.0 - normalized * normalized
            }
        }
    }
    impl ObstacleBehaviour for SlamLaser {
        fn box_clone(&self) -> DynObstacleBehaviour {
            Box::new(*self)
        }
        fn draw(&self, main_color: Color, beat: f64) {
            let start = self.start_pos.as_vec2();
            let end = self.end_pos.as_vec2();
            let lerp = start.lerp(end, self.lerp_factor(beat) as f32);

            let mut back_color = main_color.mix(WHITE, (beat * PI * 2.0).sin() * 0.5 + 0.5);
            if beat > 0.0 {
                back_color.a = 0.0;
            } else {
                back_color = back_color.faded();
                back_color.a *= (beat / self.warn_time + 1.0).min(1.0) as f32 * 1.5;
            }

            let factor = if 0.0 < beat && beat < self.flash_time {
                1.0 - beat / self.flash_time
            } else {
                0.0
            };
            let front_color = main_color.mix(WHITE, factor);

            draw_line(
                start.x,
                start.y,
                end.x,
                end.y,
                self.thickness as f32,
                back_color,
            );
            draw_line(
                start.x,
                start.y,
                lerp.x,
                lerp.y,
                self.thickness as f32,
                front_color,
            );
        }
        fn update(&mut self, shared: &mut Shared, beat: f64) {
            if beat > 0.0 && !self.slam_done {
                self.slam_done = true;
                shared.add_jerk(self.jerk);
                shared.add_shake(self.shake);
            }
        }
        fn should_enable(&self, beat: f64) -> bool {
            -self.warn_time < beat && beat < self.lifetime + self.leave_time
        }
        fn should_kill(&self, beat: f64) -> bool {
            beat > self.lifetime + self.leave_time
        }
        fn collides(&self, beat: f64, circle_pos: DVec2, circle_radius: f64) -> bool {
            collide::circle_line(
                circle_pos,
                circle_radius,
                self.start_pos,
                self.start_pos.lerp(self.end_pos, self.lerp_factor(beat)),
                self.thickness,
            )
        }
    }

    #[derive(Clone, Copy)]
    pub struct WidenLaser {
        pub start_pos: DVec2,
        pub end_pos: DVec2,

        pub warn_time: f64,
        pub grow_time: f64,
        pub lifetime: f64,
        pub shrink_time: f64,
        pub thickness: f64,
        pub flash_time: f64,

        pub shake: f64,
        pub jerk: Vec2,
        pub started_growing: bool,
    }
    impl Default for WidenLaser {
        fn default() -> Self {
            Self {
                start_pos: DVec2::ZERO,
                end_pos: DVec2::ZERO,
                warn_time: 2.0,
                grow_time: 0.25,
                lifetime: 2.0,
                shrink_time: 0.25,
                thickness: 50.0,
                flash_time: 0.5,
                shake: 0.0,
                jerk: Vec2::ZERO,
                started_growing: false,
            }
        }
    }
    impl WidenLaser {
        builder!(start_pos: DVec2);
        builder!(end_pos: DVec2);
        builder!(warn_time: f64);
        builder!(grow_time: f64);
        builder!(lifetime: f64);
        builder!(shrink_time: f64);
        builder!(thickness: f64);
        builder!(flash_time: f64);
        builder!(shake: f64);
        builder!(jerk: Vec2);

        pub fn thickness_factor(&self, beat: f64) -> f64 {
            if beat < 0.0 {
                0.0
            } else if beat < self.grow_time {
                beat / self.grow_time
            } else if beat < (self.lifetime - self.shrink_time) {
                1.0
            } else if beat < self.lifetime {
                (self.lifetime - beat) / self.shrink_time
            } else {
                0.0
            }
        }
    }
    impl ObstacleBehaviour for WidenLaser {
        fn box_clone(&self) -> DynObstacleBehaviour {
            Box::new(*self)
        }
        fn draw(&self, main_color: Color, beat: f64) {
            let start = self.start_pos.as_vec2();
            let end = self.end_pos.as_vec2();

            let mut back_color = main_color.mix(WHITE, (beat * PI * 2.0).sin() * 0.5 + 0.5);
            if beat > 0.0 {
                back_color.a = 0.0;
            } else {
                back_color = back_color.faded();
                back_color.a *= (beat / self.warn_time + 1.0).min(1.0) as f32 * 1.5;
            }

            let factor = if 0.0 < beat && beat < self.flash_time {
                1.0 - beat / self.flash_time
            } else {
                0.0
            };
            let front_color = main_color.mix(WHITE, factor);

            draw_line(
                start.x,
                start.y,
                end.x,
                end.y,
                self.thickness as f32,
                back_color,
            );
            draw_line(
                start.x,
                start.y,
                end.x,
                end.y,
                (self.thickness * self.thickness_factor(beat)) as f32,
                front_color,
            );
        }
        fn update(&mut self, shared: &mut Shared, beat: f64) {
            if beat > 0.0 && !self.started_growing {
                self.started_growing = true;
                shared.add_jerk(self.jerk);
                shared.add_shake(self.shake);
            }
        }
        fn should_enable(&self, beat: f64) -> bool {
            -self.warn_time < beat && beat < self.lifetime
        }
        fn should_kill(&self, beat: f64) -> bool {
            beat > self.lifetime
        }
        fn collides(&self, beat: f64, circle_pos: DVec2, circle_radius: f64) -> bool {
            beat > 0.0
                && collide::circle_line(
                    circle_pos,
                    circle_radius,
                    self.start_pos,
                    self.end_pos,
                    self.thickness * self.thickness_factor(beat),
                )
        }
    }
}

#[derive(Clone, Copy)]
pub struct Bomb {
    pub start_position: DVec2,
    pub end_position: DVec2,
    pub lifetime: f64,
    pub radius_per_beat: f64,
    pub projectile_count: usize,
    pub projectile_radius: f64,
    pub projectile_speed: f64,
}
impl Bomb {
    pub fn pos(&self, beat: f64) -> DVec2 {
        self.end_position
            .lerp(self.start_position, 1.0 / (beat + 1.0))
    }
}
impl ObstacleBehaviour for Bomb {
    fn box_clone(&self) -> DynObstacleBehaviour {
        self.clone().boxed()
    }
    fn collides(&self, beat: f64, circle_pos: DVec2, circle_radius: f64) -> bool {
        collide::circle_circle(
            circle_pos,
            circle_radius,
            self.pos(beat),
            self.radius_per_beat * beat,
        )
    }
    fn draw(&self, main_color: Color, beat: f64) {
        let color = if (beat * beat) / self.lifetime % 0.5 > 0.25 {
            main_color
        } else {
            WHITE
        };
        let pos = self.pos(beat);
        draw_circle(
            pos.x as f32,
            pos.y as f32,
            (self.radius_per_beat * beat) as f32,
            color,
        );
    }
    fn should_enable(&self, beat: f64) -> bool {
        beat > 0.0 && beat < self.lifetime
    }
    fn should_kill(&self, beat: f64) -> bool {
        beat >= self.lifetime
    }
    fn kill(&mut self, shared: &mut Shared, beat: f64) {
        let pos = self.pos(beat);
        for i in 0..self.projectile_count {
            let frac = i as f64 / self.projectile_count as f64 * TAU64;
            let proj = Circle::pellet(
                self.projectile_radius,
                pos,
                dvec2(frac.sin(), frac.cos()) * self.projectile_speed,
            );
            shared.add_obstacle(Obstacle::new(0.0, proj.boxed()));
        }
    }
}
#[derive(Clone)]
pub struct Circle<F: Provider<DVec2>> {
    position: F,
    radius: f64,
    /// `None` signifies that it should die when going offscreen.
    lifetime: Option<f64>,
}
impl<F: Provider<DVec2>> Circle<F> {
    pub fn new(position: F, radius: f64, lifetime: Option<f64>) -> Self {
        Self {
            position,
            radius,
            lifetime,
        }
    }
}
impl Circle<Velocity> {
    pub fn pellet(radius: f64, base_position: DVec2, velocity: DVec2) -> Self {
        Self {
            position: Velocity {
                start: base_position,
                velocity,
            },
            radius,
            lifetime: None,
        }
    }
}
impl<F: Provider<DVec2> + Clone + 'static> ObstacleBehaviour for Circle<F> {
    fn box_clone(&self) -> DynObstacleBehaviour {
        self.clone().boxed()
    }
    fn collides(&self, beat: f64, circle_pos: DVec2, circle_radius: f64) -> bool {
        collide::circle_circle(
            circle_pos,
            circle_radius,
            self.position.get(beat),
            self.radius,
        )
    }
    fn draw(&self, main_color: Color, beat: f64) {
        let pos = self.position.get(beat);
        draw_circle(pos.x as f32, pos.y as f32, self.radius as f32, main_color);
    }
    fn should_enable(&self, _beat: f64) -> bool {
        true
    }
    fn should_kill(&self, beat: f64) -> bool {
        match self.lifetime {
            Some(l) => beat > l,
            None => {
                let pos = self.position.get(beat);
                pos.x < -self.radius
                    || pos.y < -self.radius
                    || pos.x > screen_width() as f64 + self.radius
                    || pos.y > screen_height() as f64 + self.radius
            }
        }
    }
}

pub struct SetForeground(pub Box<dyn Provider<Color>>);
impl ObstacleBehaviour for SetForeground {
    fn box_clone(&self) -> DynObstacleBehaviour {
        Self(self.0.box_clone()).boxed()
    }
    fn draw(&self, _main_color: Color, _beat: f64) {}
    fn should_kill(&self, beat: f64) -> bool {
        beat > 0.0
    }
    fn update(&mut self, shared: &mut Shared, beat: f64) {
        if beat > 0.0 {
            shared.set_foreground(self.0.box_clone());
        }
    }
}
pub struct SetBackground(pub Box<dyn Provider<Color>>);
impl ObstacleBehaviour for SetBackground {
    fn box_clone(&self) -> DynObstacleBehaviour {
        Self(self.0.box_clone()).boxed()
    }
    fn draw(&self, _main_color: Color, _beat: f64) {}
    fn should_kill(&self, beat: f64) -> bool {
        beat > 0.0
    }
    fn update(&mut self, shared: &mut Shared, beat: f64) {
        if beat > 0.0 {
            shared.set_background(self.0.box_clone());
        }
    }
}
pub struct Shake(pub f64);
impl ObstacleBehaviour for Shake {
    fn box_clone(&self) -> DynObstacleBehaviour {
        Self(self.0).boxed()
    }
    fn draw(&self, _main_color: Color, _beat: f64) {}
    fn should_kill(&self, beat: f64) -> bool {
        beat > 0.0
    }
    fn update(&mut self, shared: &mut Shared, _beat: f64) {
        println!("{}", self.0);
        shared.add_shake(self.0);
    }
}
pub struct Rectangle {
    pub center: Box<dyn Provider<DVec2>>,
    pub size: Box<dyn Provider<DVec2>>,
    pub rotation: Box<dyn Provider<f64>>,
    pub lifetime: f64,
    pub warn_time: f64,
    pub leave_time: f64,
}
impl Rectangle {
    pub fn size_factor(&self, beat: f64) -> f64 {
        if (0.0..0.5).contains(&beat) {
            return 1.25 - beat * 0.5;
        } else if (-self.leave_time..0.0).contains(&(beat - self.lifetime)) {
            return (self.lifetime - beat) / self.leave_time;
        } else {
            return 1.0;
        }
    }
    pub fn color_mix_factor(&self, beat: f64) -> f64 {
        if (0.0..0.5).contains(&beat) {
            return 1.0 - beat * 2.0;
        } else if beat < 0.0 {
            return (beat * TAU64).sin() * 0.5 + 0.5;
        } else {
            return 0.0;
        }
    }
}
impl ObstacleBehaviour for Rectangle {
    fn box_clone(&self) -> DynObstacleBehaviour {
        Self {
            center: self.center.box_clone(),
            size: self.size.box_clone(),
            rotation: self.rotation.box_clone(),
            lifetime: self.lifetime,
            warn_time: self.warn_time,
            leave_time: self.leave_time,
        }
        .boxed()
    }
    fn collides(&self, beat: f64, circle_pos: DVec2, circle_radius: f64) -> bool {
        beat > 0.0
            && collide::circle_rectangle(
                circle_pos,
                circle_radius,
                self.center.get(beat),
                self.size.get(beat) * self.size_factor(beat).min(1.0),
                -self.rotation.get(beat),
            )
    }
    fn should_enable(&self, beat: f64) -> bool {
        beat > -self.warn_time
    }
    fn draw(&self, main_color: Color, beat: f64) {
        let size_factor = self.size_factor(beat);
        let color_factor = self.color_mix_factor(beat);
        let mut color = main_color.mix(WHITE, color_factor);
        if beat < 0.0 {
            color = color.faded();
            color.a *= ((beat + self.warn_time) / self.warn_time) as f32 * 1.5;
        }
        draw::rotated_rect(
            self.center.get(beat).as_vec2(),
            (self.size.get(beat) * size_factor).as_vec2(),
            self.rotation.get(beat) as f32,
            color,
        );
    }
    fn should_kill(&self, beat: f64) -> bool {
        beat > self.lifetime
    }
}
pub struct RectangleGenerator {
    pub interval: f64,
    pub lifetime: f64,
    pub spawned: usize,
    pub spawned_center: Box<dyn Provider<DVec2>>,
    pub spawned_size: Box<dyn Provider<DVec2>>,
    pub spawned_rotation: Box<dyn Provider<f64>>,
    pub spawned_lifetime: f64,
    pub spawned_warn_time: f64,
}
impl Clone for RectangleGenerator {
    fn clone(&self) -> Self {
        Self {
            spawned_center: self.spawned_center.box_clone(),
            spawned_size: self.spawned_size.box_clone(),
            spawned_rotation: self.spawned_rotation.box_clone(),
            ..*self
        }
    }
}
impl ObstacleBehaviour for RectangleGenerator {
    fn box_clone(&self) -> DynObstacleBehaviour {
        self.clone().boxed()
    }
    fn should_kill(&self, beat: f64) -> bool {
        beat > self.lifetime
    }
    fn should_enable(&self, beat: f64) -> bool {
        beat > -self.spawned_warn_time
    }
    fn update(&mut self, shared: &mut Shared, beat: f64) {
        while beat + self.spawned_warn_time > self.spawned as f64 * self.interval {
            if self.spawned as f64 >= self.lifetime / self.interval {
                break;
            }
            shared.add_obstacle(Obstacle::new(
                self.spawned_warn_time,
                Rectangle {
                    center: Constant(self.spawned_center.get(beat)).boxed(),
                    size: Constant(self.spawned_size.get(beat)).boxed(),
                    rotation: Constant(self.spawned_rotation.get(beat)).boxed(),
                    lifetime: self.spawned_lifetime,
                    warn_time: self.spawned_warn_time,
                    leave_time: 0.25,
                }
                .boxed(),
            ));
            self.spawned += 1;
        }
    }
    fn draw(&self, _main_color: Color, _beat: f64) {}
}
