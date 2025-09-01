use crate::level_generation::components::{Position, Room};
use bevy::prelude::*;

/// Keep the Transform translation in sync with the Position component for rooms.
pub fn sync_transform_with_position(
    mut rooms_transforms_query: Query<(&Position, &mut Transform), With<Room>>,
) {
    for (position, mut transform) in &mut rooms_transforms_query {
        transform.translation.x = position.x;
        transform.translation.y = position.y;
    }
}
