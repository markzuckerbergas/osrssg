use bevy::prelude::*;
use osrssg::*;

/// Main entry point for the OSRSSG game
/// 
/// This sets up the Bevy app with all necessary systems:
/// - Setup: Initializes the game scene, camera, lighting, and units
/// - Input: Handles mouse clicks for selection and movement commands
/// - Camera: Provides camera movement and zoom controls
/// - Movement: Moves units and manages their animations
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<GameState>()
        .init_resource::<CameraSettings>()
        .init_resource::<MinimapSettings>()
        .add_systems(Startup, (
            setup_scene,
            setup_animations,
            setup_minimap,
        ))
        .add_systems(
            Update,
            (
                // Input handling (first)
                (handle_drag_selection_start, handle_drag_selection_update, handle_drag_selection_complete, handle_movement_command),
                
                // Movement (runs before animations so they can detect component changes)
                move_units,
                
                // Animation setup and animation logic (runs after movement to detect Moving component changes)
                (setup_animation_players, animate_units),
                
                // Camera controls (enhanced with edge scrolling and bounds, zoom disabled)
                (camera_movement, edge_scrolling),
                
                // Minimap updates
                (update_minimap, toggle_minimap_visibility, handle_minimap_click, handle_minimap_viewport_drag_start, handle_minimap_viewport_drag, update_minimap_viewport_appearance),
                
                // Debug systems
                debug_animation_assets,
                debug_moving_components,
            ).chain()  // Run systems in this exact order
        )
        .run();
}
