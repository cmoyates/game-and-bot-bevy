use bevy::window::{Window, WindowResizeConstraints, WindowResolution};
use bevy::{color, prelude::*};

mod config;
mod level_generation;
mod post;
mod render;

fn main() {
    App::new()
        .insert_resource(ClearColor(color::palettes::basic::WHITE.into()))
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
        .add_plugins(post::PostProcessingPlugin)
        .add_plugins(render::RenderToTexturePlugin) // spawns cameras + fullscreen quad
        // Use the level generation plugin for camera + room spawning + systems
        .add_plugins(level_generation::RoomGenPlugin)
        // (Optional) log whenever the window is resized
        .add_systems(Update, on_window_resized)
        .add_systems(Update, exit_on_key)
        .run();
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
