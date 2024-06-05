//! Level construction for Leroy - ...during pride month?
//!
//! WTF IS A LEVEL EDITOR!!!!!!!!!!!

use std::f64::consts::{E, FRAC_PI_2, FRAC_PI_4, TAU};

use jut::extensions::Boxed;
use macroquad::{
    color::{Color, BLACK},
    math::{dvec2, DVec2},
    window::{screen_height, screen_width},
};
use obstacles::{PolygonPayload, TheShapes};
use providers::Homosexuality;
use rand::{seq::SliceRandom, thread_rng, Rng};

use crate::{
    level::{Level, LevelBuilder},
    obstacle::{
        lasers::{SlamLaser, WidenLaser},
        Bomb, Obstacle, ObstacleBehaviour, Rectangle, RectangleGenerator, SetBackground,
        SetForeground, Shake,
    },
    polygon::{
        self,
        presets::{generate_heart, generate_polygon, generate_spokes},
        Polygon,
    },
    provider::{Constant, FnWrap, Provider, ProviderOffset},
    res::songs,
};

mod providers {
    use std::cell::Cell;

    use jut::extensions::Boxed;
    use macroquad::color::{Color, BLACK, WHITE};

    use crate::{ext::ColorExt, provider::Provider};

    #[rustfmt::skip]
    pub const GAY_COLORS: [Color; 10] = [
        Color::new(1.0, 0.0, 0.0, 1.0),
        Color::new(1.0, 0.5, 0.0, 1.0),
        Color::new(1.0, 1.0, 0.0, 1.0),
        Color::new(0.0, 1.0, 0.0, 1.0),
        Color::new(0.0, 0.0, 1.0, 1.0),
        Color::new(0.5, 0.0, 1.0, 1.0),
        //
        Color::new(1.0, 0.5, 0.8, 1.0),
        Color::new(0.5, 0.8, 1.0, 1.0),
        Color::new(0.8, 0.8, 0.8, 1.0),
        //
        Color::new(0.5, 0.25, 0.0, 1.0),
    ];
    #[derive(Clone)]
    pub struct Homosexuality {
        pub timings: Vec<f64>,
        pub current_index: Cell<usize>,
        pub is_background: bool,
    }
    impl Homosexuality {
        pub fn new(mut timings: Vec<f64>, is_background: bool) -> Self {
            timings.insert(0, 0.0);
            Self {
                timings,
                is_background,
                current_index: Cell::new(1),
            }
        }
    }
    impl Provider<Color> for Homosexuality {
        fn box_clone(&self) -> Box<dyn Provider<Color>> {
            self.clone().boxed()
        }
        fn get(&self, beat: f64) -> Color {
            while self.current_index.get() < self.timings.len() {
                if beat > self.timings[self.current_index.get()] {
                    self.current_index.set(self.current_index.get() + 1);
                } else {
                    break;
                }
            }
            let designator_color = if self.is_background { BLACK } else { WHITE };
            let base_color = GAY_COLORS[(self.current_index.get() - 1) % GAY_COLORS.len()]
                .mix(designator_color, 0.5);
            let flash_color = designator_color.mix(WHITE, 0.5);
            let timing = if self.timings.is_empty() {
                0.0
            } else {
                self.timings[self
                    .current_index
                    .get()
                    .min(self.timings.len())
                    .saturating_sub(1)]
            };
            flash_color.mix(base_color, ((beat - timing) * 2.0).min(1.0))
        }
    }
}

mod obstacles {
    use std::f64::consts::TAU;

    use jut::extensions::Boxed;
    use macroquad::{
        color::Color,
        math::{DVec2, Vec2},
    };

    use crate::{
        collide,
        ext::ColorExt,
        obstacle::{Circle, DynObstacleBehaviour, Obstacle, ObstacleBehaviour},
        polygon::Polygon,
        provider::Velocity,
        shared::Shared,
    };

    #[derive(Clone)]
    pub struct PolygonPayload {
        polygon: Polygon,
        time: f64,
        shake: f64,
        projectile_count: usize,
    }
    impl PolygonPayload {
        pub fn new(polygon: Polygon, time: f64, shake: f64, projectile_count: usize) -> Self {
            Self {
                polygon,
                time,
                shake,
                projectile_count,
            }
        }
    }

    #[derive(Clone)]
    pub struct TheShapes {
        pub polygons: Vec<PolygonPayload>,
        pub lifetime: f64,
        pub current_index: usize,
        pub has_updated_once: bool,
        pub position: Vec2,
        pub scale: f64,
        pub warn_time: f64,
    }
    impl TheShapes {
        pub fn new(
            polygons: Vec<PolygonPayload>,
            lifetime: f64,
            position: Vec2,
            scale: f64,
            warn_time: f64,
        ) -> Self {
            Self {
                polygons,
                lifetime,
                position,
                scale,
                current_index: 0,
                has_updated_once: false,
                warn_time,
            }
        }
    }
    impl ObstacleBehaviour for TheShapes {
        fn box_clone(&self) -> DynObstacleBehaviour {
            self.clone().boxed()
        }
        fn collides(&self, beat: f64, circle_pos: DVec2, circle_radius: f64) -> bool {
            beat > 0.0
                && collide::circle_circle(
                    circle_pos,
                    circle_radius,
                    self.position.as_dvec2(),
                    self.scale,
                )
        }
        fn draw(&self, mut main_color: Color, beat: f64) {
            if beat < 0.0 {
                main_color = main_color.faded();
                main_color.a *= (beat / self.warn_time) as f32 + 1.0;
            }
            self.polygons.get(self.current_index).map(|payload| {
                payload.polygon.draw(
                    self.position,
                    0.0,
                    Vec2::new(self.scale as f32, self.scale as f32),
                    main_color,
                )
            });
        }
        fn update(&mut self, shared: &mut Shared, beat: f64) {
            if beat < 0.0 {
                return;
            }
            if !self.has_updated_once {
                self.has_updated_once = true;
                if let Some(first_payload) = self.polygons.get(0) {
                    shared.add_shake(first_payload.shake);
                    for i in 0..first_payload.projectile_count {
                        let period = i as f64 / first_payload.projectile_count as f64 * TAU;
                        let dir = DVec2::new(period.sin(), period.cos());
                        shared.add_obstacle(Obstacle::new(
                            0.0,
                            Circle::new(
                                Velocity::new(self.position.as_dvec2() + dir * 50.0, dir * 100.0),
                                10.0,
                                None,
                            )
                            .boxed(),
                        ));
                    }
                }
            }
            while let Some(payload) = self.polygons.get(self.current_index + 1) {
                if beat > payload.time {
                    shared.add_shake(payload.shake);
                    for i in 0..payload.projectile_count {
                        let period = i as f64 / payload.projectile_count as f64 * TAU;
                        let dir = DVec2::new(period.sin(), period.cos());
                        shared.add_obstacle(Obstacle::new(
                            0.0,
                            Circle::new(
                                Velocity::new(self.position.as_dvec2() + dir * 50.0, dir * 100.0),
                                10.0,
                                None,
                            )
                            .boxed(),
                        ));
                    }
                    self.current_index += 1;
                } else {
                    break;
                }
            }
        }
        fn should_enable(&self, beat: f64) -> bool {
            beat > -self.warn_time
        }
        fn should_kill(&self, beat: f64) -> bool {
            beat > self.lifetime
        }
    }
}

pub fn build() -> Level {
    let mut level_builder = LevelBuilder::new();
    // intro
    let scr_width = screen_width() as f64;
    let scr_height = screen_height() as f64;
    let scr_size = DVec2::new(scr_width, scr_height);
    let chords = 4;
    for (offset, lifetime) in [(0.0, 2.0), (2.0, 1.5), (3.5, 1.0), (4.5, 1.5), (6.0, 2.0)] {
        for i in 0..4 {
            for _ in 0..chords {
                level_builder.obstacle(Obstacle::new(
                    offset + i as f64 * 8.0,
                    WidenLaser::default()
                        .warn_time(4.0)
                        .thickness(50.0)
                        .lifetime(lifetime)
                        .start_pos(DVec2::new(
                            thread_rng().gen_range(-scr_width..scr_width * 2.0),
                            -50.0,
                        ))
                        .end_pos(DVec2::new(
                            thread_rng().gen_range(-scr_width..scr_width * 2.0),
                            scr_height as f64 + 50.0,
                        ))
                        .boxed(),
                ));
            }
        }
    }
    for _ in 0..chords {
        level_builder.pop_last_obstacle();
    }

    #[rustfmt::skip]
    let cardinal_slam_data = [
        (32.0, false), (33.0, false), (34.0, true), (34.5, true), (35.5, false),
        (36.5, true), (37.0, false), (38.0, true),

        (40.0, false), (41.0, false), (42.0, true), (42.5, false), (43.5, true),
        (44.5, true), (45.0, false), (46.0, false), (46.5, true), (47.0, true),

        (48.0, false), (51.5, false), (52.5, false), (53.0, true), (54.0, true),
        
        (56.0, false), (57.0, false), (58.0, true), (58.5, false), (59.5, true),
        (60.5, true), (61.0, false), (62.0, false), (62.5, true), (63.0, true),
    ];
    // the little drop
    for (idx, (time, horizontal)) in cardinal_slam_data.into_iter().enumerate() {
        let [start, end] = if horizontal {
            let mut x = thread_rng().gen_range(50.0..screen_width() as f64 * 0.5);
            if idx % 2 == 0 {
                x += screen_width() as f64 * 0.5 - 100.0;
            }
            let mut out = [
                DVec2::new(x, -50.0),
                DVec2::new(x, screen_height() as f64 + 50.0),
            ];
            out.shuffle(&mut thread_rng());
            out
        } else {
            let mut y = thread_rng().gen_range(0.0..screen_height() as f64 * 0.5);
            if idx % 2 == 0 {
                y += screen_height() as f64 * 0.5;
            }
            let mut out = [
                DVec2::new(-50.0, y),
                DVec2::new(screen_width() as f64 + 50.0, y),
            ];
            out.shuffle(&mut thread_rng());
            out
        };
        level_builder.obstacle(Obstacle::new(
            time,
            SlamLaser::default()
                .warn_time(4.0)
                .lifetime(2.0)
                .start_pos(start)
                .end_pos(end)
                .thickness(100.0)
                .shake(20.0)
                .boxed(),
        ));
    }

    for i in 0..14 {
        if i == 4 || i == 12 {
            continue;
        }
        let y = thread_rng().gen_range(50.0f64..screen_height() as f64 - 50.0f64);
        let y_start_offset = thread_rng().gen_range(-20.0f64..20.0f64);
        level_builder.obstacle(Obstacle::new(
            i as f64 * 4.0 + 33.0,
            Bomb {
                start_position: DVec2::new(screen_width() as f64, y + y_start_offset),
                end_position: DVec2::new(screen_width() as f64 - 150.0, y),
                lifetime: 1.0,
                radius_per_beat: 10.0,
                projectile_count: 12,
                projectile_radius: 5.0,
                projectile_speed: 200.0,
            }
            .boxed(),
        ));
    }

    #[rustfmt::skip]
    let uncardinal_slam_data = [
        (34.0, true), (34.5, true), (35.5, false), (36.5, true), (37.0, false), (38.0, true),

        (40.0, false), (41.0, false), (42.0, true), (42.5, false), (43.5, true),
        (44.5, true), (45.0, false), (46.0, false), (46.5, true), (47.0, true),

        (48.0, false), (51.5, false), (52.5, false), (53.0, true), (54.0, true),
    ];
    for (idx, (time, horizontal)) in uncardinal_slam_data.into_iter().enumerate() {
        let time = time + 32.0;
        let [start, end] = if horizontal {
            let mut x = thread_rng().gen_range(50.0..screen_width() as f64 * 0.5);
            let mut x2 = thread_rng().gen_range(50.0..screen_width() as f64 * 0.5);
            if idx % 2 == 0 {
                x += screen_width() as f64 * 0.5 - 100.0;
                x2 += screen_width() as f64 * 0.5 - 100.0;
            }
            let mut out = [
                DVec2::new(x, -50.0),
                DVec2::new(x2, screen_height() as f64 + 50.0),
            ];
            out.shuffle(&mut thread_rng());
            out
        } else {
            let mut y = thread_rng().gen_range(50.0..screen_height() as f64 * 0.5);
            let mut y2 = thread_rng().gen_range(50.0..screen_height() as f64 * 0.5);
            if idx % 2 == 0 {
                y += screen_height() as f64 * 0.5 - 100.0;
                y2 += screen_height() as f64 * 0.5 - 100.0;
            }
            let mut out = [
                DVec2::new(-50.0, y),
                DVec2::new(screen_width() as f64 + 50.0, y2),
            ];
            out.shuffle(&mut thread_rng());
            out
        };
        level_builder.obstacle(Obstacle::new(
            time,
            SlamLaser::default()
                .warn_time(4.0)
                .lifetime(1.0)
                .start_pos(start)
                .end_pos(end)
                .thickness(100.0)
                .shake(20.0)
                .boxed(),
        ));
    }
    let laser = SlamLaser::default().warn_time(4.0).leave_time(2.0);

    level_builder.obstacle(Obstacle::new(
        88.0,
        laser
            .clone()
            .start_pos(DVec2::new(scr_width * 0.5, -50.0))
            .end_pos(DVec2::new(scr_width * 0.5, scr_height + 50.0))
            .thickness(200.0)
            .shake(50.0)
            .boxed(),
    ));
    level_builder.obstacle(Obstacle::new(
        90.0,
        laser
            .clone()
            .end_pos(DVec2::new(scr_width * 0.2, -50.0))
            .start_pos(DVec2::new(scr_width * 0.2, scr_height + 50.0))
            .thickness(100.0)
            .shake(25.0)
            .boxed(),
    ));
    level_builder.obstacle(Obstacle::new(
        90.0,
        laser
            .clone()
            .end_pos(DVec2::new(scr_width * 0.8, -50.0))
            .start_pos(DVec2::new(scr_width * 0.8, scr_height + 50.0))
            .thickness(100.0)
            .shake(25.0)
            .boxed(),
    ));
    level_builder.obstacle(Obstacle::new(
        92.0,
        laser
            .clone()
            .end_pos(DVec2::new(scr_width + 50.0, scr_height * 0.75))
            .start_pos(DVec2::new(-50.0, scr_height * 0.75))
            .thickness(100.0)
            .shake(25.0)
            .boxed(),
    ));
    level_builder.obstacle(Obstacle::new(
        93.0,
        laser
            .clone()
            .end_pos(DVec2::new(-50.0, scr_height * 0.25))
            .start_pos(DVec2::new(scr_width + 50.0, scr_height * 0.25))
            .thickness(100.0)
            .shake(25.0)
            .boxed(),
    ));
    level_builder.obstacle(Obstacle::new(
        30.0,
        Rectangle {
            center: Constant::<DVec2>(scr_size * 0.5).boxed(),
            size: Constant::<DVec2>(dvec2(150.0, 150.0)).boxed(),
            rotation: FnWrap(|beat| -(beat - 1.0).max(0.0) * TAU * 2.0).boxed(),
            lifetime: 2.0,
            warn_time: 4.0,
            leave_time: 0.1,
        }
        .boxed(),
    ));

    #[rustfmt::skip]
    let timings = [
        0.0, 1.0, 2.0, 2.75, 3.5,
        4.0, 5.0, 6.0, 6.75, 7.5,
        8.0, 9.0, 10.0, 10.75, 11.5,
        12.0, 13.0, 14.0,
    ];
    // the kick lasers
    for timing in timings {
        for i in [96.0, 336.0, 352.0, 368.0] {
            let x = thread_rng().gen_range(0.0..scr_width);
            level_builder.obstacle(Obstacle::new(
                timing + i,
                WidenLaser::default()
                    .start_pos(dvec2(x, -50.0))
                    .end_pos(dvec2(x, scr_height + 50.0))
                    .thickness(50.0)
                    .shake(10.0)
                    .grow_time(0.01)
                    .lifetime(0.5)
                    .warn_time(4.0)
                    .boxed(),
            ));
            let x = thread_rng().gen_range(0.0..scr_width);
            level_builder.obstacle(Obstacle::new(
                timing + i,
                WidenLaser::default()
                    .start_pos(dvec2(x, -50.0))
                    .end_pos(dvec2(x, scr_height + 50.0))
                    .thickness(50.0)
                    .shake(10.0)
                    .grow_time(0.01)
                    .lifetime(0.5)
                    .warn_time(4.0)
                    .boxed(),
            ));
        }
        for i in [112.0, 384.0] {
            let x = thread_rng().gen_range(0.0..scr_width);
            let x2 = thread_rng().gen_range(0.0..scr_width);
            level_builder.obstacle(Obstacle::new(
                timing + i,
                WidenLaser::default()
                    .start_pos(dvec2(x, -50.0))
                    .end_pos(dvec2(x2, scr_height + 50.0))
                    .thickness(75.0)
                    .grow_time(0.125)
                    .shrink_time(0.125)
                    .lifetime(1.0)
                    .shake(5.0)
                    .warn_time(4.0)
                    .boxed(),
            ));
            let x = thread_rng().gen_range(0.0..scr_width);
            let x2 = thread_rng().gen_range(0.0..scr_width);
            level_builder.obstacle(Obstacle::new(
                timing + i,
                WidenLaser::default()
                    .start_pos(dvec2(x, -50.0))
                    .end_pos(dvec2(x2, scr_height + 50.0))
                    .thickness(75.0)
                    .grow_time(0.125)
                    .shrink_time(0.125)
                    .lifetime(1.0)
                    .shake(5.0)
                    .warn_time(4.0)
                    .boxed(),
            ));
        }
    }
    #[rustfmt::skip]
    let bomb_timings = [
        0.0, 4.0, 8.0, 10.0, 12.0
    ];
    for coarse in [320.0, 336.0, 352.0] {
        for fine in bomb_timings {
            for projectiles in 8..=10 {
                let y = thread_rng().gen_range(50.0..scr_height - 50.0);
                level_builder.obstacle(Obstacle::new(
                    coarse + fine - 2.0,
                    Bomb {
                        start_position: dvec2(scr_width, y),
                        end_position: dvec2(scr_width - 50.0, y),
                        lifetime: 2.0,
                        radius_per_beat: 10.0,
                        projectile_count: projectiles,
                        projectile_radius: 5.0,
                        projectile_speed: 100.0,
                    }
                    .boxed(),
                ));
            }
        }
    }
    // the boingy lasers
    for i in 0..31 {
        if i == 15 {
            continue;
        }
        level_builder.obstacle(Obstacle::new(
            i as f64 * 1.0 + 352.0,
            WidenLaser::default()
                .start_pos(dvec2(-50.0, 0.0))
                .end_pos(dvec2(scr_width + 50.0, 0.0))
                .thickness(125.0)
                .grow_time(0.125)
                .shrink_time(0.125)
                .lifetime(0.5)
                .flash_time(0.0)
                .warn_time(if i == 0 { 4.0 } else { 0.0 })
                .boxed(),
        ));
        level_builder.obstacle(Obstacle::new(
            i as f64 * 1.0 + 352.5,
            WidenLaser::default()
                .start_pos(dvec2(-50.0, scr_height))
                .end_pos(dvec2(scr_width + 50.0, scr_height))
                .thickness(125.0)
                .grow_time(0.125)
                .shrink_time(0.125)
                .lifetime(0.5)
                .flash_time(0.0)
                .warn_time(if i == 0 { 4.0 } else { 0.0 })
                .boxed(),
        ));
    }

    // the breakdown
    // the part with no words
    let generator = RectangleGenerator {
        interval: 0.25,
        lifetime: 8.0,
        spawned: 0,
        spawned_center: FnWrap(|_| {
            DVec2::new(
                (thread_rng().gen_range(0.0..screen_width()) as f64 / 50.0).round() * 50.0,
                (thread_rng().gen_range(0.0..screen_height()) as f64 / 50.0).round() * 50.0,
            )
        })
        .boxed(),
        spawned_size: Constant(dvec2(50.0, 50.0)).boxed(),
        spawned_rotation: Constant(0.0).boxed(),
        spawned_lifetime: 1.0,
        spawned_warn_time: 4.0,
    };

    let mut wide_generator = generator.clone();
    wide_generator.spawned_size = Constant(dvec2(150.0, 50.0)).boxed();
    let mut tall_generator = generator.clone();
    tall_generator.spawned_size = Constant(dvec2(50.0, 150.0)).boxed();

    for _ in 0..3 {
        tall_generator.lifetime = 8.0;
        level_builder.obstacle(Obstacle::new(128.0, wide_generator.box_clone()));
        level_builder.obstacle(Obstacle::new(136.0, tall_generator.box_clone()));
        level_builder.obstacle(Obstacle::new(144.0, wide_generator.box_clone()));
        tall_generator.lifetime = 6.0;
        level_builder.obstacle(Obstacle::new(152.0, tall_generator.box_clone()));
    }

    let gen = FnWrap(|beat| (beat * 4.0).round() / 16.0 * TAU).boxed();
    wide_generator.spawned_rotation = gen.box_clone();
    tall_generator.spawned_rotation = gen.box_clone();

    for _ in 0..3 {
        tall_generator.lifetime = 8.0;
        level_builder.obstacle(Obstacle::new(160.0, wide_generator.box_clone()));
        level_builder.obstacle(Obstacle::new(168.0, tall_generator.box_clone()));
        level_builder.obstacle(Obstacle::new(176.0, wide_generator.box_clone()));
        tall_generator.lifetime = 6.0;
        level_builder.obstacle(Obstacle::new(184.0, tall_generator.box_clone()));
    }

    // face is a blur
    level_builder.obstacle(Obstacle::new(
        192.0,
        Rectangle {
            center: FnWrap(move |beat| {
                DVec2::new((beat * 0.6).sin(), (beat * 0.6).cos()) * beat.max(0.0) * 25.0
                    + scr_size / 2.0
            })
            .boxed(),
            size: FnWrap(|beat| {
                let factor = if beat < 0.0 {
                    1.0
                } else {
                    (-E.powf(-beat) - E.powf(beat - 16.0) + 1.0).max(0.0)
                };
                DVec2::splat(factor * 300.0)
            })
            .boxed(),
            rotation: FnWrap(|beat| -beat * 0.5).boxed(),
            lifetime: 16.0,
            warn_time: 4.0,
            leave_time: 0.01,
        }
        .boxed(),
    ));
    level_builder.obstacle(Obstacle::new(
        192.0,
        Rectangle {
            center: FnWrap(move |beat| {
                DVec2::new((beat * 0.6).sin(), (beat * 0.6).cos()) * beat.max(0.0) * -25.0
                    + scr_size / 2.0
            })
            .boxed(),
            size: FnWrap(|beat| {
                let factor = if beat < 0.0 {
                    1.0
                } else {
                    (-E.powf(-beat) - E.powf(beat - 16.0) + 1.0).max(0.0)
                };
                DVec2::splat(factor * 300.0)
            })
            .boxed(),
            rotation: FnWrap(|beat| -beat * 0.5).boxed(),
            lifetime: 16.0,
            warn_time: 4.0,
            leave_time: 0.01,
        }
        .boxed(),
    ));

    // oop, big drop time, load up all the polygons now
    let the_big_drop_time = 224.0;
    #[rustfmt::skip]
    let polies = vec![
        PolygonPayload::new(Polygon::from_arrays(generate_polygon::<50>(0.0)), 0.0, 20.0, 25,),
        PolygonPayload::new(Polygon::from_arrays(generate_polygon::<4>(0.0)), 1.0, 20.0, 20,),
        PolygonPayload::new(Polygon::from_arrays(generate_spokes::<4>(0.4, 0.4, 0.0)), 2.0, 20.0, 20,),
        PolygonPayload::new(Polygon::from_arrays(generate_polygon::<3>(FRAC_PI_2)), 2.5, 10.0, 10,),
        PolygonPayload::new(Polygon::from_arrays(generate_polygon::<3>(0.0)), 3.5, 20.0, 20,),
        PolygonPayload::new(Polygon::from_arrays(generate_polygon::<4>(FRAC_PI_4)), 4.5, 10.0, 7,),
        PolygonPayload::new(Polygon::from_arrays(generate_polygon::<6>(0.0)), 5.0, 20.0, 15,),
        PolygonPayload::new(Polygon::from_arrays(generate_spokes::<2>(0.2, 0.2, 0.0)), 6.0, 30.0, 20,),
        PolygonPayload::new(Polygon::from_arrays(generate_spokes::<3>(0.2, 0.2, FRAC_PI_2 as f32)), 6.5, 0.0, 0,),
        PolygonPayload::new(Polygon::from_arrays(generate_spokes::<4>(0.2, 0.2, FRAC_PI_2 as f32 * 2.0)), 7.0, 0.0, 0,),
        PolygonPayload::new(Polygon::from_arrays(generate_spokes::<5>(0.2, 0.2, FRAC_PI_2 as f32 * 3.0)), 7.5, 0.0, 0,),

        PolygonPayload::new(Polygon::from_arrays(generate_heart::<50>()), 8.0, 30.0, 25),
        PolygonPayload::new(Polygon::from_arrays(generate_polygon::<5>(FRAC_PI_2 * 1.5)), 9.0, 20.0, 15,),
        PolygonPayload::new(Polygon::from_arrays(generate_polygon::<5>(FRAC_PI_2 * -1.5)), 10.0, 20.0, 20,),
        PolygonPayload::new(Polygon::from_arrays(generate_spokes::<3>(0.0, 0.4, 0.0)), 10.5, 10.0, 10,),
        PolygonPayload::new(Polygon::from_arrays(generate_spokes::<5>(0.0, 1.0, FRAC_PI_2 as f32 * -1.5)), 11.5, 20.0, 20,),
        PolygonPayload::new(Polygon::from_arrays(generate_spokes::<4>(0.0, 0.2, FRAC_PI_2 as f32 * 0.5)), 12.5, 10.0, 10,),
        PolygonPayload::new(Polygon::from_arrays(generate_spokes::<5>(0.0, 0.5, FRAC_PI_2 as f32 * 1.5)), 13.0, 20.0, 15,),
        PolygonPayload::new(Polygon::from_arrays(generate_polygon::<6>(0.0)), 14.0, 20.0, 20,),
        PolygonPayload::new(Polygon::from_arrays(generate_polygon::<8>(0.0)), 14.5, 10.0, 10,),
        PolygonPayload::new(Polygon::from_arrays(generate_polygon::<10>(0.0)), 15.0, 20.0, 15,),
        PolygonPayload::new(Polygon::from_arrays(generate_polygon::<12>(0.0)), 15.5, 20.0, 25,),

        PolygonPayload::new(Polygon::from_arrays(polygon::presets::none()), 16.0, 20.0, 5,),
        PolygonPayload::new(Polygon::from_arrays(generate_polygon::<3>(0.0)), 19.5, 20.0, 20,),
        PolygonPayload::new(Polygon::from_arrays(generate_polygon::<4>(FRAC_PI_4)), 20.5, 10.0, 7,),
        PolygonPayload::new(Polygon::from_arrays(generate_polygon::<6>(0.0)), 21.0, 20.0, 15,),
        PolygonPayload::new(Polygon::from_arrays(generate_spokes::<2>(0.2, 0.2, 0.0)), 22.0, 30.0, 20,),
        PolygonPayload::new(Polygon::from_arrays(generate_spokes::<3>(0.2, 0.2, FRAC_PI_2 as f32)), 22.5, 0.0, 0,),
        PolygonPayload::new(Polygon::from_arrays(generate_spokes::<4>(0.2, 0.2, FRAC_PI_2 as f32 * 2.0)), 23.0, 0.0, 0,),
        PolygonPayload::new(Polygon::from_arrays(generate_spokes::<5>(0.2, 0.2, FRAC_PI_2 as f32 * 3.0)), 23.5, 0.0, 0,),

        PolygonPayload::new(Polygon::from_arrays(generate_heart::<50>()), 24.0, 30.0, 25),
        PolygonPayload::new(Polygon::from_arrays(generate_polygon::<5>(FRAC_PI_2 * 1.5)), 25.0, 20.0, 15,),
        PolygonPayload::new(Polygon::from_arrays(generate_polygon::<5>(FRAC_PI_2 * -1.5)), 26.0, 20.0, 20,),
        PolygonPayload::new(Polygon::from_arrays(generate_spokes::<3>(0.0, 0.4, 0.0)), 26.5, 10.0, 10,),
        PolygonPayload::new(Polygon::from_arrays(generate_spokes::<5>(0.0, 1.0, FRAC_PI_2 as f32 * -1.5)), 27.5, 20.0, 20,),
        PolygonPayload::new(Polygon::from_arrays(generate_spokes::<4>(0.0, 0.2, FRAC_PI_2 as f32 * 0.5)), 28.5, 10.0, 10,),
        PolygonPayload::new(Polygon::from_arrays(generate_spokes::<5>(0.0, 0.5, FRAC_PI_2 as f32 * 1.5)), 29.0, 20.0, 15,),
        PolygonPayload::new(Polygon::from_arrays(generate_polygon::<6>(0.0)), 30.0, 20.0, 20,),
        PolygonPayload::new(Polygon::from_arrays(generate_polygon::<8>(0.0)), 30.5, 10.0, 10,),
        PolygonPayload::new(Polygon::from_arrays(generate_polygon::<10>(0.0)), 31.0, 20.0, 15,),
        PolygonPayload::new(Polygon::from_arrays(generate_polygon::<12>(0.0)), 31.5, 20.0, 25,),
    ];
    level_builder.obstacle(Obstacle::new(
        the_big_drop_time,
        TheShapes::new(polies, 32.0, scr_size.as_vec2() * 0.5, 100.0, 8.0).boxed(),
    ));
    #[rustfmt::skip]
    let gay_foreground = Homosexuality::new(
        vec![
            1.0, 2.0, 2.5, 3.5, 4.5, 5.0, 6.0,
            8.0, 9.0, 10.0, 10.5, 11.5, 12.5, 13.0, 14.0, 14.5, 15.0, 15.5,
            16.0, 19.5, 20.5, 21.0, 22.0,
            24.0, 25.0, 26.0, 26.5, 27.5, 28.5, 29.0, 30.0, 30.5, 31.0, 31.5,
            31.6, 31.65, 31.7, 31.75, 31.8, 31.85, 31.9, 31.95,

            32.0, 33.0, 34.0, 34.5, 35.5, 36.5, 37.0, 38.0, 38.5, 39.0, 39.5,
            40.0, 41.0, 42.0, 42.5, 43.5, 44.5, 45.0, 46.0, 46.5, 47.0, 47.5,
            48.0, 51.5, 52.5, 53.0, 54.0, 54.5, 55.0, 55.5,
            56.0, 58.0, 60.0, 61.0,
        ],
        false,
    );
    let mut gay_background = gay_foreground.clone();
    gay_background.is_background = true;
    level_builder.obstacle(Obstacle::new(
        the_big_drop_time,
        SetForeground(ProviderOffset(gay_foreground.boxed(), the_big_drop_time).boxed()).boxed(),
    ));
    level_builder.obstacle(Obstacle::new(
        the_big_drop_time,
        SetBackground(ProviderOffset(gay_background.boxed(), the_big_drop_time).boxed()).boxed(),
    ));
    for coarse in [0, 1, 2, 5, 6] {
        let coarse_time = coarse as f64 * 4.0 + 1.0 + the_big_drop_time;
        for fine in 0..3 {
            let fine_time = fine as f64 * 0.125;
            for _ in 0..3 {
                let x = thread_rng().gen_range(0.0..scr_width);
                level_builder.obstacle(Obstacle::new(
                    coarse_time + fine_time,
                    SlamLaser::default()
                        .start_pos(DVec2::new(x + thread_rng().gen_range(-50.0..50.0), -50.0))
                        .end_pos(DVec2::new(
                            x + thread_rng().gen_range(-50.0..50.0),
                            scr_height + 50.0,
                        ))
                        .warn_time(4.0)
                        .thickness(25.0)
                        .lifetime(1.0)
                        .boxed(),
                ));
            }
        }
    }

    let ring_count = 20;
    for rect in 0..ring_count {
        let period = TAU * rect as f64 / ring_count as f64;
        level_builder.obstacle(Obstacle::new(
            258.0,
            Rectangle {
                center: FnWrap(move |beat| {
                    DVec2::new((period + beat).sin(), (period + beat).cos()) * scr_width * 0.4
                        - DVec2::new((beat * 0.25).sin(), (beat * 0.4).cos()) * 250.0
                        + scr_size / 2.0
                })
                .boxed(),
                size: Constant(dvec2(50.0, 50.0)).boxed(),
                rotation: FnWrap(move |beat| period - beat * 2.0).boxed(),
                lifetime: 22.0,
                warn_time: 4.0,
                leave_time: 0.25,
            }
            .boxed(),
        ));
        level_builder.obstacle(Obstacle::new(
            258.0,
            Rectangle {
                center: FnWrap(move |beat| {
                    DVec2::new((period - beat).sin(), (period - beat).cos()) * scr_width * 0.5
                        - DVec2::new((beat * 0.25).sin(), (beat * 0.4).cos()) * 250.0
                        + scr_size / 2.0
                })
                .boxed(),
                size: Constant(dvec2(50.0, 50.0)).boxed(),
                rotation: FnWrap(move |beat| period - beat * 2.0).boxed(),
                lifetime: 22.0,
                warn_time: 4.0,
                leave_time: 0.25,
            }
            .boxed(),
        ));
    }
    let bomb_steps = 5;
    let bomb_timing_interval = 0.5 / bomb_steps as f64;
    let bomb_vertical_interval = (scr_height - 100.0) / bomb_steps as f64;
    let first_bomb_position = DVec2::new(scr_width - 50.0, 50.0);
    for offset in [256.0, 260.0, 264.0, 276.0] {
        for step in 0..bomb_steps {
            level_builder.obstacle(Obstacle::new(
                offset + step as f64 * bomb_timing_interval,
                Bomb {
                    start_position: first_bomb_position
                        + DVec2::new(50.0, 0.0)
                        + DVec2::new(0.0, step as f64 * bomb_vertical_interval),
                    end_position: first_bomb_position
                        + DVec2::new(0.0, step as f64 * bomb_vertical_interval),
                    lifetime: 1.0,
                    radius_per_beat: 25.0,
                    projectile_count: 16,
                    projectile_radius: 5.0,
                    projectile_speed: 300.0,
                }
                .boxed(),
            ));
        }
    }

    #[rustfmt::skip]
    let laser_timings = [
        34.0, 34.5, 35.5, 36.5, 37.0, 38.0,
        40.0, 41.0, 42.0, 42.5, 43.5, 44.5, 45.0, 46.0, 46.5, 47.0,
        48.0, 51.5, 52.5, 53.0, 54.0, 55.0,

        56.0, 58.0, 60.0, 61.0, 56.0, 58.0, 60.0, 61.0, // doubled up lasers
    ];
    for i in laser_timings {
        let is_horizontal = thread_rng().gen::<bool>();
        let swap_sides = thread_rng().gen::<bool>();
        let max_size = if is_horizontal { scr_width } else { scr_height };
        let opposite_size = if is_horizontal { scr_height } else { scr_width };
        let position = thread_rng().gen_range(0.0..max_size);

        let (mut start_x, mut end_x) = (-50.0, opposite_size + 50.0);
        let (mut start_y, mut end_y) = (position, position);

        if is_horizontal {
            (start_x, end_x, start_y, end_y) = (start_y, end_y, start_x, end_x);
        }
        if swap_sides {
            (start_x, end_x, start_y, end_y) = (end_x, start_x, end_y, start_y);
        }
        level_builder.obstacle(Obstacle::new(
            i + the_big_drop_time,
            SlamLaser::default()
                .thickness(50.0)
                .start_pos(dvec2(start_x, start_y))
                .end_pos(dvec2(end_x, end_y))
                .warn_time(4.0)
                .lifetime(1.0)
                .leave_time(1.0)
                .shake(10.0)
                .boxed(),
        ));
    }
    level_builder.obstacle(Obstacle::new(
        286.0,
        SetForeground(Constant(Color::new(1.0, 0.0, 0.5, 1.0)).boxed()).boxed(),
    ));
    level_builder.obstacle(Obstacle::new(
        286.0,
        SetBackground(Constant(BLACK).boxed()).boxed(),
    ));
    for i in 0..121 {
        let offset = i as f64 / 4.0 + 288.0;
        let is_horizontal = thread_rng().gen::<bool>();
        let swap_sides = thread_rng().gen::<bool>();
        let max_size = if is_horizontal { scr_width } else { scr_height };
        let opposite_size = if is_horizontal { scr_height } else { scr_width };
        let position = thread_rng().gen_range(0.0..max_size);

        let (mut start_x, mut end_x) = (-50.0, opposite_size + 50.0);
        let (mut start_y, mut end_y) = (position, position);

        if is_horizontal {
            (start_x, end_x, start_y, end_y) = (start_y, end_y, start_x, end_x);
        }
        if swap_sides {
            (start_x, end_x, start_y, end_y) = (end_x, start_x, end_y, start_y);
        }
        level_builder.obstacle(Obstacle::new(
            offset,
            SlamLaser::default()
                .thickness(25.0)
                .start_pos(dvec2(start_x, start_y))
                .end_pos(dvec2(end_x, end_y))
                .warn_time(4.0)
                .lifetime(1.0)
                .leave_time(1.0)
                .shake(2.0)
                .boxed(),
        ));
    }

    // toothpaste <3<3<3
    for i in 1..=10 {
        level_builder.obstacle(Obstacle::new(
            396.0,
            Bomb {
                start_position: dvec2(scr_width, scr_height * 0.5),
                end_position: dvec2(scr_width - 100.0, scr_height * 0.5),
                lifetime: 4.0,
                radius_per_beat: 10.0,
                projectile_count: i * 3,
                projectile_radius: 10.0,
                projectile_speed: i as f64 * 10.0,
            }
            .boxed(),
        ));
    }
    level_builder.obstacle(Obstacle::new(400.0, Shake(250.0).boxed()));
    level_builder.obstacle(Obstacle::new(
        400.0,
        Rectangle {
            center: Constant(scr_size * 0.5).boxed(),
            size: FnWrap(|beat| {
                DVec2::splat(beat * (beat - 27.0).max(0.0).powf(6.0)) * 0.005 + 100.0
            })
            .boxed(),
            rotation: FnWrap(|beat| beat * 1.25f64.powf(beat - 27.0).max(1.0)).boxed(),
            lifetime: 32.0,
            warn_time: 4.0,
            leave_time: 0.01,
        }
        .boxed(),
    ));

    level_builder.build(
        songs::DURING_PRIDE_MONTH,
        193.0,
        2.4963,
        &[94.0, 216.0, 318.0, 396.0],
    )
}
