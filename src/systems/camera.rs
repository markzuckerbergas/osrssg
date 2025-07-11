use bevy::prelude::*;
use crate::resources::CameraSettings;
use crate::components::{MainCamera, MinimapCamera};

/// Handles camera movement with arrow keys and applies bounds
pub fn camera_movement(
    keys: Res<ButtonInput<KeyCode>>,
    mut cameras: Query<&mut Transform, (With<Camera>, With<MainCamera>)>,
    settings: Res<CameraSettings>,
    time: Res<Time>,
) {
    for mut transform in cameras.iter_mut() {
        let mut direction = Vec3::ZERO;
        
        if keys.pressed(KeyCode::ArrowLeft) {
            direction -= transform.rotation * Vec3::X;
        }
        if keys.pressed(KeyCode::ArrowRight) {
            direction += transform.rotation * Vec3::X;
        }
        if keys.pressed(KeyCode::ArrowUp) {
            direction += transform.rotation * Vec3::Y;
        }
        if keys.pressed(KeyCode::ArrowDown) {
            direction -= transform.rotation * Vec3::Y;
        }
        
        if direction != Vec3::ZERO {
            let movement = direction.normalize() * settings.move_speed * time.delta_secs();
            let new_position = transform.translation + movement;
            
            // Apply camera bounds
            transform.translation = Vec3::new(
                new_position.x.clamp(settings.bounds_min.x, settings.bounds_max.x),
                new_position.y.clamp(settings.bounds_min.y, settings.bounds_max.y),
                new_position.z.clamp(settings.bounds_min.z, settings.bounds_max.z),
            );
        }
    }
}

/// Handles edge scrolling when mouse is near screen edges
pub fn edge_scrolling(
    windows: Query<&Window>,
    mut cameras: Query<&mut Transform, (With<Camera>, With<MainCamera>)>,
    settings: Res<CameraSettings>,
    time: Res<Time>,
) {
    let Ok(window) = windows.single() else { return; };
    let Some(cursor_position) = window.cursor_position() else { return; };
    
    let window_size = Vec2::new(window.width(), window.height());
    let margin = settings.edge_scroll_margin;
    
    for mut transform in cameras.iter_mut() {
        let mut direction = Vec3::ZERO;
        
        // Check edges and calculate scroll direction
        if cursor_position.x < margin {
            // Left edge
            direction -= transform.rotation * Vec3::X;
        } else if cursor_position.x > window_size.x - margin {
            // Right edge
            direction += transform.rotation * Vec3::X;
        }
        
        if cursor_position.y < margin {
            // Top edge (in screen coordinates, Y=0 is top)
            direction += transform.rotation * Vec3::Y;
        } else if cursor_position.y > window_size.y - margin {
            // Bottom edge
            direction -= transform.rotation * Vec3::Y;
        }
        
        if direction != Vec3::ZERO {
            let movement = direction.normalize() * settings.edge_scroll_speed * time.delta_secs();
            let new_position = transform.translation + movement;
            
            // Apply camera bounds
            transform.translation = Vec3::new(
                new_position.x.clamp(settings.bounds_min.x, settings.bounds_max.x),
                new_position.y.clamp(settings.bounds_min.y, settings.bounds_max.y),
                new_position.z.clamp(settings.bounds_min.z, settings.bounds_max.z),
            );
        }
    }
}

/// Handles camera zoom with mouse wheel
pub fn camera_zoom(
    mut scroll_events: EventReader<bevy::input::mouse::MouseWheel>,
    mut cameras: Query<&mut Transform, (With<Camera>, With<MainCamera>)>,
    settings: Res<CameraSettings>,
) {
    for event in scroll_events.read() {
        for mut transform in cameras.iter_mut() {
            let zoom_delta = 1.0 + (-event.y * settings.zoom_speed);
            let new_scale = transform.scale * zoom_delta;
            
            // Clamp zoom levels
            let scale_factor = new_scale.x.clamp(settings.min_zoom, settings.max_zoom);
            transform.scale = Vec3::splat(scale_factor);
        }
    }
}

/// Updates minimap camera to follow the main camera
pub fn update_minimap_camera(
    main_cameras: Query<&Transform, (With<MainCamera>, Without<MinimapCamera>)>,
    mut minimap_cameras: Query<&mut Transform, (With<MinimapCamera>, Without<MainCamera>)>,
) {
    if let Ok(main_transform) = main_cameras.single() {
        for mut minimap_transform in minimap_cameras.iter_mut() {
            // Position minimap camera above the main camera's position
            minimap_transform.translation = Vec3::new(
                main_transform.translation.x,
                main_transform.translation.y + 20.0, // Higher up for top-down view
                main_transform.translation.z,
            );
            
            // Always look straight down
            minimap_transform.look_at(
                Vec3::new(
                    main_transform.translation.x,
                    0.0,
                    main_transform.translation.z,
                ),
                Vec3::Y,
            );
        }
    }
}

/// Handle minimap clicking to move camera (simplified for now)
pub fn minimap_navigation() {
    // Future: Implement minimap clicking to move main camera
    // This would require UI interaction detection with the minimap viewport
}
