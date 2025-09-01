use bevy::window::{Window, WindowResizeConstraints, WindowResolution};
use bevy::{color, prelude::*};

fn main() {
    App::new()
        .insert_resource(ClearColor(color::palettes::basic::BLACK.into()))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Resizable Window".into(),
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
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    let size = Vec2::new(100.0, 100.0);
    let color = color::palettes::basic::RED;
    let sprite = Sprite::from_color(color, size);

    commands.spawn(sprite);
}

fn on_window_resized(mut evr: EventReader<bevy::window::WindowResized>) {
    for _e in evr.read() {
        // info!("Window {} -> {} x {}", e.window, e.width, e.height);
    }
}
