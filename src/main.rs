use bevy::prelude::*;
use osrssg::*;

/// Main entry point for the OSRSSG game
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<GameState>()
        .init_resource::<CameraSettings>()
        .init_resource::<MinimapSettings>()
        .init_resource::<MinimapDragState>()
        .add_systems(Startup, (setup_scene, setup_animations, setup_minimap, setup_game_ui))
        .add_systems(
            Update,
            (
                // Minimap (runs first to get priority)
                (handle_minimap_drag, handle_minimap_click),
                // Input handling
                (
                    handle_double_click_selection, // AoE2-style double-click selection
                    handle_drag_selection_start,
                    handle_drag_selection_update,
                    handle_drag_selection_complete,
                    handle_movement_command,
                ),
                // UI interactions
                handle_spawn_button,
                // Movement
                move_units,
                // Animation
                (setup_animation_players, animate_units),
                // Camera controls
                (camera_movement, edge_scrolling),
                // Minimap updates
                (update_minimap, toggle_minimap_visibility),
                // Debug
                // debug_entity_spawning,
                // debug_animation_assets,
                // debug_moving_components,
                // debug_collision_circles, // Uncomment to see collision circles
            )
                .chain(),
        )
        .run();
}
