use bevy::prelude::*;

#[derive(Component, Deref, DerefMut)]
pub struct Position(pub Vec2);
#[derive(Component, Deref, DerefMut)]
pub struct Velocity(pub Vec2);
#[derive(Component, Deref, DerefMut)]
pub struct Acceleration(pub Vec2);
#[derive(Component, Deref, DerefMut)]
pub struct Size(pub Vec2);
#[derive(Component)]
pub struct Room;
