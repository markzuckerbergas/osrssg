use bevy::prelude::*;
use crate::components::*;

/// Legacy single-click selection - now handled by drag selection system
/// This function is kept for compatibility but functionality moved to drag selection
pub fn handle_unit_selection(
    // This system is now effectively disabled - drag selection handles both clicks and drags
) {
    // Functionality moved to handle_drag_selection_complete
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

/// Handles starting drag selection
pub fn handle_drag_selection_start(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    minimap_query: Query<&Node, With<MinimapUI>>,
    existing_drag: Query<Entity, With<DragSelection>>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };

    // Check if click is on minimap - if so, don't start drag selection
    if let Ok(_minimap_node) = minimap_query.single() {
        let minimap_rect = Rect::from_corners(
            Vec2::new(window.width() - 200.0, window.height() - 200.0),
            Vec2::new(window.width(), window.height())
        );
        
        if minimap_rect.contains(cursor_pos) {
            return; // Don't start drag selection on minimap
        }
    }

    // Remove any existing drag selection
    for entity in existing_drag.iter() {
        commands.entity(entity).despawn();
    }

    // Start new drag selection
    let _drag_entity = commands.spawn((
        DragSelection {
            start_pos: cursor_pos,
            current_pos: cursor_pos,
        },
        Name::new("DragSelection"),
    )).id();

    // Create the visual drag box
    commands.spawn((
        DragSelectionBox,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(cursor_pos.x),
            top: Val::Px(cursor_pos.y),
            width: Val::Px(0.0),
            height: Val::Px(0.0),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
        BackgroundColor(Color::srgba(0.5, 0.5, 1.0, 0.2)),
        Name::new("DragSelectionBox"),
    ));

    info!("🖱️ Started drag selection at: ({:.2}, {:.2})", cursor_pos.x, cursor_pos.y);
}

/// Updates drag selection while dragging
pub fn handle_drag_selection_update(
    mut drag_query: Query<&mut DragSelection>,
    mut box_query: Query<&mut Node, With<DragSelectionBox>>,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
) {
    if drag_query.is_empty() {
        return;
    }

    let Ok(window) = windows.single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };

    if let Ok(mut drag) = drag_query.single_mut() {
        drag.current_pos = cursor_pos;

        // Update visual box
        if let Ok(mut node) = box_query.single_mut() {
            let min_x = drag.start_pos.x.min(drag.current_pos.x);
            let min_y = drag.start_pos.y.min(drag.current_pos.y);
            let max_x = drag.start_pos.x.max(drag.current_pos.x);
            let max_y = drag.start_pos.y.max(drag.current_pos.y);

            node.left = Val::Px(min_x);
            node.top = Val::Px(min_y);
            node.width = Val::Px(max_x - min_x);
            node.height = Val::Px(max_y - min_y);
        }
    }
}

/// Completes drag selection when mouse is released
pub fn handle_drag_selection_complete(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    drag_query: Query<(Entity, &DragSelection)>,
    box_query: Query<Entity, With<DragSelectionBox>>,
    units: Query<(Entity, &GlobalTransform), (With<Controllable>, With<SceneRoot>)>,
    selected: Query<Entity, With<Selected>>,
) {
    if !buttons.just_released(MouseButton::Left) || drag_query.is_empty() {
        return;
    }

    let Ok(window) = windows.single() else { return; };
    let Ok((camera, cam_transform)) = cameras.single() else { return; };
    let Ok((drag_entity, drag)) = drag_query.single() else { return; };

    // Deselect all current selections
    for entity in selected.iter() {
        commands.entity(entity).remove::<Selected>();
    }

    // Calculate selection rectangle
    let min_x = drag.start_pos.x.min(drag.current_pos.x);
    let min_y = drag.start_pos.y.min(drag.current_pos.y);
    let max_x = drag.start_pos.x.max(drag.current_pos.x);
    let max_y = drag.start_pos.y.max(drag.current_pos.y);
    let selection_rect = Rect::from_corners(
        Vec2::new(min_x, min_y),
        Vec2::new(max_x, max_y)
    );

    let mut selected_count = 0;

    // Check if drag was just a click (small area)
    let drag_area = (max_x - min_x) * (max_y - min_y);
    if drag_area < 25.0 { // Small area threshold (5x5 pixels)
        // Treat as single click selection - use existing logic
        let cursor_pos = drag.current_pos;
        let Ok(ray) = camera.viewport_to_world(cam_transform, cursor_pos) else { 
            cleanup_drag_selection(&mut commands, drag_entity, &box_query);
            return; 
        };

        // Single unit selection with cylinder intersection
        for (entity, unit_transform) in units.iter() {
            let unit_pos = unit_transform.translation();
            let selection_radius = 1.2;
            let selection_height = 2.0;
            
            if ray_intersects_cylinder(ray, unit_pos, selection_radius, selection_height) {
                commands.entity(entity).insert(Selected);
                selected_count += 1;
                break; // Only select one unit for click
            }
        }
    } else {
        // Multi-selection with rectangle
        for (entity, unit_transform) in units.iter() {
            let unit_pos = unit_transform.translation();
            
            // Project 3D position to 2D screen coordinates
            if let Ok(screen_pos) = camera.world_to_viewport(cam_transform, unit_pos) {
                // Check if unit is within selection rectangle
                if selection_rect.contains(screen_pos) {
                    commands.entity(entity).insert(Selected);
                    selected_count += 1;
                }
            }
        }
    }

    info!("✅ Drag selection complete: {} units selected", selected_count);

    // Cleanup drag selection
    cleanup_drag_selection(&mut commands, drag_entity, &box_query);
}

/// Helper function to clean up drag selection entities
fn cleanup_drag_selection(
    commands: &mut Commands,
    drag_entity: Entity,
    box_query: &Query<Entity, With<DragSelectionBox>>,
) {
    // Remove drag selection entity
    commands.entity(drag_entity).despawn();

    // Remove drag selection box
    for entity in box_query.iter() {
        commands.entity(entity).despawn();
    }
}
