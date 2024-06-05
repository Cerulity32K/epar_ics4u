#![allow(incomplete_features)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(generic_const_exprs)]
#![feature(const_fn_floating_point_arithmetic)]
#![feature(never_type)]

use std::error::Error;

use draw::draw_screen_centered_text;
use macroquad::{
    camera::{set_camera, set_default_camera, Camera2D},
    color::{BLACK, GREEN, WHITE},
    math::Vec2,
    miniquad::window::screen_size,
    text::draw_text,
    time::get_time,
    window::{clear_background, next_frame, request_new_screen_size},
};
use music::Music;
use player::Player;
use rand::{thread_rng, Rng};
use res::sfx;
use soloud::{AudioExt, LoadExt, Wav};

mod collide;
mod draw;
mod ext;
mod level;
mod levels;
mod macros;
mod music;
mod obstacle;
mod player;
mod polygon;
mod provider;
mod res;
mod shared;
mod transform;

#[macroquad::main("Exclusively Polygons Alongside Rhythms")]
async fn main() -> Result<(), Box<dyn Error>> {
    // notification of removal for in class
    // blocks further action
    // loop {
    //     use macroquad::color::RED;

    //     clear_background(BLACK);
    //     draw_screen_centered_text(":(", 0.0, -32.0, 64, RED);
    //     draw_screen_centered_text(
    //         "Removal of this game has been requested. Sorry!",
    //         0.0,
    //         0.0,
    //         32,
    //         WHITE,
    //     );
    //     draw_screen_centered_text("github.com/cerulity32k", 0.0, 32.0, 32, WHITE);
    //     draw_screen_centered_text("youtube.com/@cerulity32k", 0.0, 64.0, 32, WHITE);
    //     next_frame().await;
    // }

    // the actual game
    let mut level = levels::dpm::build();
    let mut player = Player::new();
    let mut music = Music::new()?;

    let mut music_track = Wav::default();
    music_track.load_mem(&level.song_data)?;
    music.play(&music_track, level.bpm, level.start_time);
    music.seek(0.0)?;
    level.update_to(&music);

    let mut checkpoint_sound = Wav::default();
    checkpoint_sound.load_mem(sfx::CHECKPOINT)?;
    checkpoint_sound.set_volume(3.0);
    let mut death_sound = Wav::default();
    death_sound.load_mem(sfx::DIE)?;
    death_sound.set_volume(3.0);

    let mut camera = Camera2D::default();
    let mut current_checkpoint = -(level.start_time - 0.01) * level.bpm / 60.0;
    let mut next_checkpoint_index = 0;

    loop {
        request_new_screen_size(800.0, 600.0);
        if music.finished() {
            set_default_camera();
            clear_background(BLACK);

            draw_screen_centered_text("Level complete!", 0.0, -90.0, 50, GREEN);
            draw_screen_centered_text("Song by Jane Remover (Leroy)", 0.0, -50.0, 25, WHITE);
            draw_screen_centered_text("`...during pride month?`", 0.0, -30.0, 25, WHITE);
            draw_screen_centered_text("Made by Cerulity32K", 0.0, 10.0, 25, WHITE);
            draw_screen_centered_text("github.com/cerulity32k", 0.0, 30.0, 25, WHITE);
            draw_screen_centered_text("youtube.com/@cerulity32k", 0.0, 50.0, 25, WHITE);
            draw_screen_centered_text("Inspired by Just Shapes and Beats", 0.0, 90.0, 25, WHITE);
            next_frame().await;
        } else {
            let screen_size: Vec2 = screen_size().into();
            camera.zoom = 2.0 / screen_size;
            level.shake = level.shake.abs();
            let shake_x = thread_rng().gen_range(-level.shake..=level.shake);
            let shake_y = thread_rng().gen_range(-level.shake..=level.shake);
            camera.target =
                screen_size / 2.0 + Vec2::new(shake_x as f32, shake_y as f32) + level.jerk;
            set_camera(&camera);
            let time = get_time();
            let beat = music.beat();
            clear_background(level.background_color(beat));
            level.update(beat);
            if player.update(time, beat, &level) {
                level = levels::dpm::build();
                player = Player::new();
                player.last_hit_time = time;
                music.play(&music_track, level.bpm, level.start_time);
                music.seek(current_checkpoint + level.start_time * level.bpm / 60.0)?;
                music.soloud.play(&death_sound);
                level.update_to(&music);
                next_frame().await;
                continue;
            }
            if let Some(&next_checkpoint) = level.checkpoints.get(next_checkpoint_index) {
                if beat > next_checkpoint {
                    current_checkpoint = next_checkpoint;
                    next_checkpoint_index += 1;
                    println!("{next_checkpoint} {beat}");
                    music.soloud.play(&checkpoint_sound);
                }
            }
            level.draw(beat);
            // level.shade_collisions(&player, beat);
            player.draw(time);
            set_default_camera();
            if next_checkpoint_index != 0 && current_checkpoint + 2.0 > beat {
                draw_text("Checkpoint!", 0.0, 32.0, 32.0, WHITE);
            }
            next_frame().await;
        }
    }
}
