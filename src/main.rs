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
        .add_systems(Startup, setup_scene)
        .add_systems(
            Update,
            (
                // Input handling
                handle_unit_selection,
                handle_movement_command,
                
                // Camera controls
                camera_movement,
                camera_zoom,
                
                // Movement and animation
                move_units,
                update_movement_animations,
                setup_initial_animations,
            )
        )
        .run();
}
