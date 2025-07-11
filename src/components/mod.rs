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

/// Tracks drag selection state
#[derive(Component)]
pub struct DragSelection {
    pub start_pos: Vec2,
    pub current_pos: Vec2,
}

/// Marks the drag selection box UI element
#[derive(Component)]
pub struct DragSelectionBox;

/// Links an AnimationPlayer to its parent controllable unit
#[derive(Component)]
pub struct UnitAnimationPlayer {
    pub unit_entity: Entity,
}

/// Collision radius for entities
#[derive(Component)]
pub struct CollisionRadius {
    pub radius: f32,
}

impl Default for CollisionRadius {
    fn default() -> Self {
        Self { radius: 0.3 } // Default radius for characters
    }
}

/// Tracks how long a unit has been stuck (not making progress)
#[derive(Component)]
pub struct StuckTimer {
    pub timer: f32,
    pub last_position: Vec3,
    pub stuck_threshold: f32, // How long before considering stuck
}

impl Default for StuckTimer {
    fn default() -> Self {
        Self {
            timer: 0.0,
            last_position: Vec3::ZERO,
            stuck_threshold: 1.5, // 1.5 seconds of no movement = stuck (increased for better navigation)
        }
    }
}
