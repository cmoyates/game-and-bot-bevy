use crate::level_generation::components::{Acceleration, Position, Velocity};
use crate::level_generation::config::SeparationCfg;
use bevy::prelude::*;

/// Integrate acceleration into velocity: v += a * dt
pub fn integrate_acceleration_into_velocity(
    mut velocities_query: Query<(&mut Velocity, &Acceleration)>,
    time: Res<Time>,
    config: Res<SeparationCfg>,
) {
    let dt = time.delta_secs();
    let drag = config.drag;
    for (mut velocity, acceleration) in &mut velocities_query {
        // v += a * dt
        **velocity += **acceleration * dt;
        // Apply simple linear drag: v *= (1 - drag * dt), clamped to non-negative factor
        let drag_factor = (1.0 - drag * dt).max(0.0);
        **velocity *= drag_factor;
    }
}

/// Integrate velocity into position: p += v * dt
pub fn integrate_velocity_into_position(
    mut positions_query: Query<(&mut Position, &Velocity)>,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();
    for (mut position, velocity) in &mut positions_query {
        **position += **velocity * delta_time;
    }
}
