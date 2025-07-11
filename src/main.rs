use bevy::prelude::*;
use osrssg::*;

/// Main entry point for the OSRSSG game
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<GameState>()
        .init_resource::<CameraSettings>()
        .init_resource::<MinimapSettings>()
        .add_systems(Startup, (setup_scene, setup_animations, setup_minimap))
        .add_systems(
            Update,
            (
                // Input handling
                (
                    handle_drag_selection_start,
                    handle_drag_selection_update,
                    handle_drag_selection_complete,
                    handle_movement_command,
                ),
                // Movement
                move_units,
                // Animation
                (setup_animation_players, animate_units),
                // Camera controls
                (camera_movement, edge_scrolling),
                // Minimap
                (update_minimap, toggle_minimap_visibility, handle_minimap_click),
                // Debug
                debug_entity_spawning,
                debug_animation_assets,
                debug_moving_components,
                // debug_collision_circles, // Uncomment to see collision circles
            )
                .chain(),
        )
        .run();
}
