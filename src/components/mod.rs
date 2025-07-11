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

/// Individual destination for each unit
#[derive(Component)]
pub struct Destination {
    pub target: Vec3,
}

/// Marks the main game camera
#[derive(Component)]
pub struct MainCamera;

/// Marks the minimap camera
#[derive(Component)]
pub struct MinimapCamera;

/// Marks the minimap UI container
#[derive(Component)]
pub struct MinimapUI;

/// Marks a player dot on the minimap
#[derive(Component)]
pub struct MinimapPlayerDot;

/// Marks the camera viewport indicator on the minimap
#[derive(Component)]
pub struct MinimapCameraViewport;

/// Tracks dragging state for the minimap viewport
#[derive(Component)]
pub struct MinimapViewportDragging {
    pub start_cursor_pos: Vec2,
    pub start_camera_pos: Vec3,
}
