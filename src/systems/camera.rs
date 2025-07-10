use bevy::prelude::*;
use crate::resources::CameraSettings;

/// Handles camera movement with arrow keys
pub fn camera_movement(
    keys: Res<ButtonInput<KeyCode>>,
    mut cameras: Query<&mut Transform, With<Camera>>,
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
            transform.translation += direction.normalize() * settings.move_speed * time.delta_secs();
        }
    }
}

/// Handles camera zoom with mouse wheel
pub fn camera_zoom(
    mut scroll_events: EventReader<bevy::input::mouse::MouseWheel>,
    mut cameras: Query<&mut Transform, With<Camera>>,
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
