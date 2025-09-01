use crate::level_generation::{
    components::{Acceleration, Position, Room, Size, Velocity},
    config::SeparationCfg,
};

use bevy::prelude::*;
use std::collections::HashMap;

/// Apply separation forces so each room accelerates away from the centroid of rooms it overlaps.
/// Rooms with no overlaps will have their acceleration and velocity set to zero (they stop).
pub fn separation_forces(
    config: Res<SeparationCfg>,
    mut rooms_query: Query<(Entity, &Position, &Size, &mut Acceleration), With<Room>>,
    mut velocity_query: Query<&mut Velocity, With<Room>>,
) {
    // Reset all accelerations at the start of the step.
    for (_, _, _, mut acceleration) in rooms_query.iter_mut() {
        **acceleration = Vec2::ZERO;
    }

    // Snapshot immutable data to compute overlaps without holding mutable borrows.
    let room_snapshots: Vec<(Entity, Vec2, Vec2)> = rooms_query
        .iter()
        .map(|(entity, position, size, _)| (entity, **position, **size))
        .collect();

    // For each room, accumulate the centroid of overlapping neighbors and the total penetration.
    let mut overlap_accumulators: HashMap<
        Entity,
        (
            Vec2,  /* sum of neighbor positions */
            usize, /* neighbor count */
            f32,   /* sum penetration */
        ),
    > = HashMap::new();

    for index_a in 0..room_snapshots.len() {
        let (entity_a, position_a, size_a) = room_snapshots[index_a];
        let half_extents_a = 0.5 * size_a;
        for index_b in (index_a + 1)..room_snapshots.len() {
            let (entity_b, position_b, size_b) = room_snapshots[index_b];
            let half_extents_b = 0.5 * size_b;

            let delta_ab = position_b - position_a; // from A to B
            let overlap_x_extent = (half_extents_a.x + half_extents_b.x) - delta_ab.x.abs();
            let overlap_y_extent = (half_extents_a.y + half_extents_b.y) - delta_ab.y.abs();

            if overlap_x_extent > 0.0 && overlap_y_extent > 0.0 {
                let penetration = overlap_x_extent.min(overlap_y_extent);
                overlap_accumulators
                    .entry(entity_a)
                    .and_modify(
                        |(neighbor_positions_sum, neighbor_count, penetration_sum)| {
                            *neighbor_positions_sum += position_b;
                            *neighbor_count += 1;
                            *penetration_sum += penetration;
                        },
                    )
                    .or_insert((position_b, 1, penetration));
                overlap_accumulators
                    .entry(entity_b)
                    .and_modify(
                        |(neighbor_positions_sum, neighbor_count, penetration_sum)| {
                            *neighbor_positions_sum += position_a;
                            *neighbor_count += 1;
                            *penetration_sum += penetration;
                        },
                    )
                    .or_insert((position_a, 1, penetration));
            }
        }
    }

    // Apply forces away from the local-overlap centroid; stop non-overlapping rooms.
    for (entity, position, _, mut acceleration) in rooms_query.iter_mut() {
        if let Some((neighbor_positions_sum, neighbor_count, penetration_sum)) =
            overlap_accumulators.get(&entity)
        {
            let overlap_centroid = *neighbor_positions_sum / (*neighbor_count as f32);
            let away_direction = (**position - overlap_centroid).normalize_or_zero();
            let force_magnitude = config.stiffness * (*penetration_sum / *neighbor_count as f32);
            **acceleration = away_direction * force_magnitude;
        } else {
            // No overlaps: halt movement
            **acceleration = Vec2::ZERO;
            if let Ok(mut velocity) = velocity_query.get_mut(entity) {
                **velocity = Vec2::ZERO;
            }
        }
    }
}

/// Clamp acceleration magnitude to the configured maximum force per room.
pub fn clamp_steering(config: Res<SeparationCfg>, mut query: Query<&mut Acceleration, With<Room>>) {
    for mut acceleration in &mut query {
        let magnitude = acceleration.length();
        if magnitude > config.max_force {
            **acceleration = **acceleration * (config.max_force / magnitude);
        }
    }
}
