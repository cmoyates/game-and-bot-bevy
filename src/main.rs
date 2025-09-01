use bevy::window::{Window, WindowResizeConstraints, WindowResolution};
use bevy::{color, prelude::*};

use rand::prelude::*;

use crate::config::{ROOM_MAX_SIDE_LENGTH, ROOM_MIN_SIDE_LENGTH, ROOM_SPAWN_RADIUS};

mod config;

fn main() {
    App::new()
        .insert_resource(ClearColor(color::palettes::basic::BLACK.into()))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Game and Bot".into(),
                // Start size (logical pixels). You can omit this if you don’t care.
                resolution: WindowResolution::new(1280.0, 720.0),
                // Make sure the window can be resized by the user:
                resizable: true,
                // (Optional) put some boundaries on how small/large it can go:
                resize_constraints: WindowResizeConstraints {
                    min_width: 640.0,
                    min_height: 360.0,
                    max_width: 3840.0,
                    max_height: 2160.0,
                },
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        // (Optional) log whenever the window is resized
        .add_systems(Update, on_window_resized)
        .add_systems(Update, exit_on_key)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn a 2D camera with a customized orthographic projection (zoomed in ~2x)
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scale: 0.5,
            ..OrthographicProjection::default_2d()
        }),
    ));

    let mut rng = rand::rng();

    let center = Vec2::ZERO;

    for _ in 0..config::ROOM_COUNT {
        let size = Vec2::new(
            rng.random_range(ROOM_MIN_SIDE_LENGTH..=ROOM_MAX_SIDE_LENGTH) as f32,
            rng.random_range(ROOM_MIN_SIDE_LENGTH..=ROOM_MAX_SIDE_LENGTH) as f32,
        );

        let color = random_color(&mut rng);

        let position = random_point_in_disk(&mut rng, center, ROOM_SPAWN_RADIUS);

        print!("Room at {position:?} of size {size:?}\n");

        commands.spawn((
            Sprite::from_color(color, size),
            Transform::from_xyz(position.x, position.y, 0.),
        ));
    }
}

// https://youtu.be/fv-wlo8yVhk
fn random_color(rng: &mut impl rand::Rng) -> Color {
    let red_green = rng.random_range(-1.0..=1.0);
    let blue_yellow = rng.random_range(-1.0..=1.0);
    Color::oklab(0.5, red_green, blue_yellow)
}

pub fn random_point_in_disk(rng: &mut impl rand::Rng, center: Vec2, radius: f32) -> Vec2 {
    // Uniform angle in [0, 2π)
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    // Critical bit for *uniform area*: r = R * sqrt(u)
    let r = radius * rng.random::<f32>().sqrt();
    center + Vec2::from_angle(angle) * r
}

fn on_window_resized(mut evr: EventReader<bevy::window::WindowResized>) {
    for _e in evr.read() {
        // info!("Window {} -> {} x {}", e.window, e.width, e.height);
    }
}

fn exit_on_key(keys: Res<ButtonInput<KeyCode>>, mut exit_events: EventWriter<AppExit>) {
    // If either the Escape key or Q is just pressed
    if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyQ) {
        exit_events.write(AppExit::Success);
    }
}
