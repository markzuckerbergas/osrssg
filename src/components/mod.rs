use bevy::prelude::*;

/// Marks an entity as selected by the player
#[derive(Component)]
pub struct Selected;

/// Marks an entity as currently moving to a destination
#[derive(Component)]
pub struct Moving;

/// Component for entities that can be controlled by the player
#[derive(Component)]
pub struct Controllable;
