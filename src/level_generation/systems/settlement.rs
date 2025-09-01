use bevy::prelude::*;

use crate::level_generation::components::{Position, Room, Size};

/// Tracks whether we've already reported that all rooms have stopped.
#[derive(Resource, Default)]
pub struct SettlementState {
    pub reported: bool,
}

/// Print once when all rooms have stopped moving (velocities are zero).
pub fn report_when_rooms_stopped(
    mut state: ResMut<SettlementState>,
    rooms: Query<(&Position, &Size), With<Room>>,
) {
    if state.reported {
        return;
    }

    let snapshots: Vec<(Vec2, Vec2)> = rooms.iter().map(|(p, s)| (**p, **s)).collect();
    if snapshots.is_empty() {
        return;
    }

    // Detect if any overlapping pairs exist (AABB on XY plane)
    let mut any_overlap = false;
    for i in 0..snapshots.len() {
        let (pos_a, size_a) = snapshots[i];
        let half_a = 0.5 * size_a;
        for j in (i + 1)..snapshots.len() {
            let (pos_b, size_b) = snapshots[j];
            let half_b = 0.5 * size_b;

            let delta = pos_b - pos_a;
            let overlap_x = (half_a.x + half_b.x) - delta.x.abs();
            let overlap_y = (half_a.y + half_b.y) - delta.y.abs();
            if overlap_x > 0.0 && overlap_y > 0.0 {
                any_overlap = true;
                break;
            }
        }
        if any_overlap {
            break;
        }
    }

    if !any_overlap {
        info!("All rooms have stopped moving.");
        state.reported = true;
    }
}
