use bevy::prelude::*;
use osrssg::*;

/// Main entry point for the OSRSSG game
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_event::<GatherEvent>()
        .add_event::<SelectionChanged>()
        .add_event::<InventoryChanged>()
        .init_resource::<GameState>()
        .init_resource::<CameraSettings>()
        .init_resource::<MinimapSettings>()
        .init_resource::<MinimapDragState>()
        .init_resource::<GatheringConfig>()
        .init_resource::<SelectedUnits>()
        .add_systems(
            Startup,
            (
                setup_scene,      // Scene, camera, lighting, units, obstacles
                setup_animations, // Animation system
                setup_minimap,    // Minimap UI
                setup_game_ui,    // Main game UI
                spawn_resources,  // Resource nodes (must run after setup_scene for collision avoidance)
            ),
        )
        .add_systems(
            Update,
            (
                // === INPUT PHASE ===
                // Minimap (runs first to get priority)
                (handle_minimap_drag, handle_minimap_click),
                // Input handling
                (
                    handle_double_click_selection, // AoE2-style double-click selection
                    handle_drag_selection_start,
                    handle_drag_selection_update,
                    handle_drag_selection_complete,
                    handle_movement_command,
                    issue_gather_task, // Process gather events right after movement commands
                ),
                
                // === MOVEMENT PHASE ===
                move_units, // Update positions first
                
                // === ECONOMY PHASE ===
                // Process economic activities after movement is complete
                process_gathering_state_machine,
                
                // === UI & VISUALS PHASE ===
                // Animation
                (setup_animation_players, animate_units),
                // UI updates
                update_inventory_ui, // Update inventory UI based on selection/inventory changes
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
