use crate::level_generation::components::*;
use bevy::prelude::*;

#[derive(Bundle)]
pub struct RoomBundle {
    pub room: Room,
    pub position: Position,
    pub velocity: Velocity,
    pub acceleration: Acceleration,
    pub size: Size,
    pub sprite: Sprite,
    pub transform: Transform,
}

impl RoomBundle {
    pub fn new(
        position: Position,
        velocity: Velocity,
        acceleration: Acceleration,
        size: Size,
        sprite_color: Color,
    ) -> Self {
        let mut sprite = Sprite::sized(size.0);
        sprite.color = sprite_color;
        Self {
            room: Room,
            position,
            velocity,
            acceleration,
            size,
            sprite,
            transform: Transform::from_translation(Vec3::ZERO),
        }
    }
}
