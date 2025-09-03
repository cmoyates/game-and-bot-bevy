// post.rs
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef, ShaderType};
use bevy::sprite::{Material2d, Material2dPlugin};

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct Globals {
    pub burnt_amount: f32,
    pub mask_intensity: f32,
    pub scanline_intensity: f32,
    pub aberration_px: f32,
    pub pixelate_px: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PostMaterial {
    // matches @group(2) @binding(0) and (1)
    #[texture(0)]
    #[sampler(1)]
    pub scene_tex: Handle<Image>,

    // this is the missing piece: matches @group(2) @binding(2)
    #[uniform(2)]
    pub globals: Globals,
}

impl Material2d for PostMaterial {
    fn fragment_shader() -> ShaderRef {
        "shader.wgsl".into()
    }
}

pub struct PostProcessingPlugin;
impl Plugin for PostProcessingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<PostMaterial>::default());
    }
}
