use bevy::prelude::*;

/// Global game state and destination for movement
#[derive(Resource, Default)]
pub struct GameState {
    pub move_destination: Option<Vec3>,
}

/// Camera settings for isometric view
#[derive(Resource)]
pub struct CameraSettings {
    pub move_speed: f32,
    pub zoom_speed: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            move_speed: 5.0,
            zoom_speed: 0.1,
            min_zoom: 1.0,
            max_zoom: 10.0,
        }
    }
}

/// Animation system using AnimationGraph approach for Bevy 0.16
#[derive(Resource)]
pub struct UnitAnimations {
    pub walk_node: AnimationNodeIndex,
    pub idle_node: AnimationNodeIndex,
    pub animation_graph: Handle<AnimationGraph>,
}
