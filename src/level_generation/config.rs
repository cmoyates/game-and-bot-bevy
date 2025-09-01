use bevy::ecs::resource::Resource;

#[derive(Clone, Copy, Resource)]
pub struct SeparationCfg {
    pub stiffness: f32,
    pub max_force: f32,
    pub drag: f32,
}
impl Default for SeparationCfg {
    fn default() -> Self {
        Self {
            stiffness: 100.0,
            max_force: 300.0,
            // Linear drag coefficient (per second). Small value for gentle damping.
            drag: 3.5,
        }
    }
}
