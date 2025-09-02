pub mod bundles;
pub mod components;
pub mod config;
mod systems;

use crate::config::{ROOM_COUNT, ROOM_MAX_SIDE_LENGTH, ROOM_MIN_SIDE_LENGTH, ROOM_SPAWN_RADIUS};
use bevy::prelude::*;
use config::SeparationCfg;
use rand::prelude::*;
use systems::{integration::*, render_sync::*, separation::*, settlement::*};

pub struct RoomGenPlugin;

impl Plugin for RoomGenPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SeparationCfg::default())
            .init_resource::<SettlementState>()
            .add_systems(
                FixedUpdate,
                (
                    separation_forces,
                    clamp_steering,
                    integrate_acceleration_into_velocity,
                    integrate_velocity_into_position,
                    sync_transform_with_position,
                    report_when_rooms_stopped,
                ),
            )
            .add_systems(Startup, setup_rooms);
    }
}

fn setup_rooms(mut commands: Commands) {
    use bundles::RoomBundle;
    use components::{Acceleration, Position, Size, Velocity};

    // Camera
    // commands.spawn((
    //     Camera2d,
    //     Projection::from(OrthographicProjection {
    //         scale: 0.5,
    //         ..OrthographicProjection::default_2d()
    //     }),
    // ));

    // Randomly spawn rooms within a disk, with random sizes and colors
    let mut rng = rand::rng();
    let disk_center = Vec2::ZERO;

    for _ in 0..ROOM_COUNT {
        let size = Vec2::new(
            rng.random_range(ROOM_MIN_SIDE_LENGTH..=ROOM_MAX_SIDE_LENGTH) as f32,
            rng.random_range(ROOM_MIN_SIDE_LENGTH..=ROOM_MAX_SIDE_LENGTH) as f32,
        );
        let color = random_color(&mut rng);
        let position = random_point_in_disk(&mut rng, disk_center, ROOM_SPAWN_RADIUS);

        commands.spawn(RoomBundle::new(
            Position(position),
            Velocity(Vec2::ZERO),
            Acceleration(Vec2::ZERO),
            Size(size),
            color,
        ));
    }
}

// Color and placement helpers (copied from previous main, adapted here)
fn random_color(rng: &mut impl rand::Rng) -> Color {
    let red_green = rng.random_range(-1.0..=1.0);
    let blue_yellow = rng.random_range(-1.0..=1.0);
    Color::oklab(1.0, red_green, blue_yellow)
}

fn random_point_in_disk(rng: &mut impl rand::Rng, center: Vec2, radius: f32) -> Vec2 {
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let distance = radius * rng.random::<f32>().sqrt();
    center + Vec2::from_angle(angle) * distance
}
