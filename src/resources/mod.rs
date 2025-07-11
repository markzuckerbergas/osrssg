use bevy::prelude::*;

/// Global game state
#[derive(Resource, Default)]
pub struct GameState {
    // Future: Add game-wide state like score, level, etc.
}

/// Camera settings for isometric view
#[derive(Resource)]
pub struct CameraSettings {
    pub move_speed: f32,
    pub zoom_speed: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
    // Edge scrolling settings
    pub edge_scroll_margin: f32,
    pub edge_scroll_speed: f32,
    // Camera bounds
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            move_speed: 8.0,          // Matched with edge_scroll_speed for consistency
            zoom_speed: 0.0,          // Disabled zoom for now
            min_zoom: 1.0,            // Standard zoom level (fixed)
            max_zoom: 1.0,            // Standard zoom level (fixed)
            edge_scroll_margin: 50.0, // Pixels from edge to trigger scrolling
            edge_scroll_speed: 8.0,   // Speed multiplier for edge scrolling
            bounds_min: Vec3::new(-50.0, 0.0, -50.0), // Expanded camera bounds
            bounds_max: Vec3::new(50.0, 15.0, 50.0), // Expanded camera bounds
        }
    }
}

/// Minimap settings and state
#[derive(Resource)]
pub struct MinimapSettings {
    pub size: Vec2,
    pub position: Vec2,   // Screen position (0,0 = top-left, 1,1 = bottom-right)
    pub world_size: Vec2, // Area of world to show
    pub zoom: f32,
}

impl Default for MinimapSettings {
    fn default() -> Self {
        Self {
            size: Vec2::new(200.0, 200.0),
            position: Vec2::new(0.98, 0.98), // Bottom-right corner with small margin
            world_size: Vec2::new(100.0, 100.0), // 100x100 world units (matches expanded bounds)
            zoom: 0.1,
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

/// Resource to track minimap drag state
#[derive(Resource, Default)]
pub struct MinimapDragState {
    pub is_dragging: bool,
    pub last_mouse_pos: Option<Vec2>,
}
