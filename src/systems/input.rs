use bevy::prelude::*;
use crate::components::*;

/// Handles left-click selection of units with 3D collision detection
/// 
/// This system uses a cylindrical collision volume around each unit that allows
/// clicking anywhere on the character model (legs, torso, head, etc.) rather
/// than just near the ground position. This provides much better user experience
/// for character selection in an isometric view.
pub fn handle_unit_selection(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    units: Query<(Entity, &GlobalTransform), (With<Controllable>, With<SceneRoot>)>,
    selected: Query<Entity, With<Selected>>,
    mut commands: Commands,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };
    let Ok((camera, cam_transform)) = cameras.single() else { return; };
    let Ok(ray) = camera.viewport_to_world(cam_transform, cursor_pos) else { return; };

    // Deselect all current selections
    for entity in selected.iter() {
        commands.entity(entity).remove::<Selected>();
    }

    // 3D selection - check if ray intersects with unit's cylindrical volume
    for (entity, unit_transform) in units.iter() {
        let unit_pos = unit_transform.translation();
        
        // Create a cylindrical selection volume around the unit
        // This allows clicking on any part of the character model:
        // - selection_radius: How wide the clickable area is (left/right of character)
        // - selection_height: How tall the clickable area is (allows clicking on head/torso)
        // Account for the small scale of the model (0.03) - adjust for visual size
        let selection_radius = 1.2; // Horizontal selection radius (generous for clicking)
        let selection_height = 2.0; // Vertical selection height (allows clicking on torso/head)
        
        // Check if the ray passes close to the unit's cylindrical volume
        if ray_intersects_cylinder(ray, unit_pos, selection_radius, selection_height) {
            commands.entity(entity).insert(Selected);
            break; // Only select one unit
        }
    }
}

/// Check if a ray intersects with a cylinder (unit selection volume)
fn ray_intersects_cylinder(ray: Ray3d, cylinder_center: Vec3, radius: f32, height: f32) -> bool {
    let ray_origin = ray.origin;
    let ray_dir = ray.direction.normalize();
    
    // Vector from ray origin to cylinder center
    let to_center = cylinder_center - ray_origin;
    
    // Project this vector onto the ray direction to find the closest point on the ray
    let t = to_center.dot(ray_dir);
    let closest_point_on_ray = ray_origin + ray_dir * t;
    
    // Calculate horizontal distance from cylinder center to the closest point on the ray
    let horizontal_distance = Vec3::new(
        cylinder_center.x - closest_point_on_ray.x,
        0.0, // Ignore Y for horizontal distance
        cylinder_center.z - closest_point_on_ray.z,
    ).length();
    
    // Check if ray passes through the cylinder horizontally
    let horizontal_hit = horizontal_distance <= radius;
    
    // Check if the ray intersects at the right height
    let ray_y_at_intersection = closest_point_on_ray.y;
    let vertical_hit = ray_y_at_intersection >= cylinder_center.y && 
                      ray_y_at_intersection <= (cylinder_center.y + height);
    
    horizontal_hit && vertical_hit
}

/// Handles right-click movement commands
pub fn handle_movement_command(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    selected_units: Query<Entity, With<Selected>>,
    mut commands: Commands,
) {
    if !buttons.just_pressed(MouseButton::Right) {
        return;
    }

    // Only move if we have selected units
    if selected_units.is_empty() {
        return;
    }

    let Ok(window) = windows.single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };
    let Ok((camera, cam_transform)) = cameras.single() else { return; };
    let Ok(ray) = camera.viewport_to_world(cam_transform, cursor_pos) else { return; };

    // Get ground intersection point
    if let Some(destination) = ray_ground_intersection(ray, 0.0) {
        info!("🎯 Movement command to: ({:.2}, {:.2}, {:.2})", destination.x, destination.y, destination.z);
        
        // Set individual destination for each selected unit
        for entity in selected_units.iter() {
            commands.entity(entity).insert((
                crate::components::Destination {
                    target: destination,
                },
                Moving,
            ));
        }
    }
}

/// Helper function to find where a ray intersects the ground plane
fn ray_ground_intersection(ray: Ray3d, ground_y: f32) -> Option<Vec3> {
    let ray_dir = ray.direction.normalize();
    
    // Check if ray is pointing downward
    if ray_dir.y >= 0.0 {
        return None;
    }
    
    // Calculate intersection with ground plane
    let t = (ground_y - ray.origin.y) / ray_dir.y;
    Some(ray.origin + ray_dir * t)
}
