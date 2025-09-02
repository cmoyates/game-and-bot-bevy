use crate::post::PostMaterial;
use bevy::math::primitives::Rectangle;
use bevy::prelude::*;
use bevy::render::camera::ClearColorConfig;
use bevy::render::{
    render_asset::RenderAssetUsages,
    render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
};
use bevy::sprite::MeshMaterial2d; // <-- so we can spawn with your material

pub struct RenderToTexturePlugin;

impl Plugin for RenderToTexturePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_cameras)
            .add_systems(Update, resize_fullscreen_quad);
    }
}

fn make_scene_target(images: &mut Assets<Image>, width: u32, height: u32) -> Handle<Image> {
    let mut image = Image::new_fill(
        Extent3d {
            width,
            height,
            ..default()
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
    images.add(image)
}

#[derive(Component)]
struct MainSceneCam;
#[derive(Component)]
struct PostCam;

fn setup_cameras(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut post_materials: ResMut<Assets<PostMaterial>>,
    windows: Query<&Window>,
) {
    // Default in case no window exists yet
    let (mut w, mut h) = (1280, 720);
    if let Some(win) = windows.iter().next() {
        let size = win.physical_size();
        w = size.x;
        h = size.y;
    }

    // 1) Scene camera renders the world into an Image
    let scene_rt = make_scene_target(&mut images, w, h);
    commands.spawn((
        Camera3d::default(),
        Camera {
            target: scene_rt.clone().into(),
            ..default()
        },
        MainSceneCam,
    ));

    // 2) Overlay camera renders to the window
    commands.spawn((
        Camera2d,
        Camera {
            order: 999, // render after MainSceneCam
            clear_color: ClearColorConfig::None,
            ..default()
        },
        PostCam,
    ));

    // 3) Store the handle somewhere you can reuse (e.g., resource). Here we’ll
    // just spawn the fullscreen quad immediately below and pass it in.
    spawn_fullscreen_quad(commands, &mut meshes, &mut post_materials, scene_rt);
}

#[derive(Component)]
struct FullscreenQuad;

fn spawn_fullscreen_quad(
    mut commands: Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<PostMaterial>>,
    scene_rt: Handle<Image>,
) {
    // Start with a 1x1 rect; a resize system will stretch it to the window size.
    let rect_mesh = meshes.add(Mesh::from(Rectangle::new(1.0, 1.0)));
    let material = materials.add(PostMaterial {
        scene_tex: scene_rt,
        globals: crate::post::Globals { burnt_amount: 1.0 },
    });

    commands.spawn((
        Mesh2d(rect_mesh),
        MeshMaterial2d(material),
        Transform::from_xyz(0.0, 0.0, 0.0), // overlay camera sees this
        FullscreenQuad,
    ));
}

fn resize_fullscreen_quad(
    windows: Query<&Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    q: Query<&Mesh2d, With<FullscreenQuad>>,
) {
    if let Some(win) = windows.iter().next() {
        let size = Vec2::new(win.width(), win.height());
        for m in &q {
            // Update the existing mesh asset to match the current window size
            if let Some(mesh) = meshes.get_mut(&m.0) {
                *mesh = Mesh::from(Rectangle::new(size.x, size.y));
            }
        }
    }
}
