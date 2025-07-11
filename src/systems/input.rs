use crate::components::*;
use crate::resources::*;
use bevy::prelude::*;

/// Legacy single-click selection - now handled by drag selection system
/// This function is kept for compatibility but functionality moved to drag selection
pub fn handle_unit_selection(// This system is now effectively disabled - drag selection handles both clicks and drags
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
    )
    .length();

    // Check if ray passes through the cylinder horizontally
    let horizontal_hit = horizontal_distance <= radius;

    // Check if the ray intersects at the right height
    let ray_y_at_intersection = closest_point_on_ray.y;
    let vertical_hit = ray_y_at_intersection >= cylinder_center.y
        && ray_y_at_intersection <= (cylinder_center.y + height);

    horizontal_hit && vertical_hit
}

/// Checks if a ray intersects with a box (obstacle)
fn ray_box_intersection(ray: Ray3d, box_center: Vec3, box_size: Vec3) -> bool {
    let ray_origin = ray.origin;
    let ray_dir = ray.direction.normalize();

    // Calculate the box bounds
    let min_bounds = box_center - box_size * 0.5;
    let max_bounds = box_center + box_size * 0.5;

    // Calculate intersection with each axis
    let t_min_x = (min_bounds.x - ray_origin.x) / ray_dir.x;
    let t_max_x = (max_bounds.x - ray_origin.x) / ray_dir.x;
    let t_min_y = (min_bounds.y - ray_origin.y) / ray_dir.y;
    let t_max_y = (max_bounds.y - ray_origin.y) / ray_dir.y;
    let t_min_z = (min_bounds.z - ray_origin.z) / ray_dir.z;
    let t_max_z = (max_bounds.z - ray_origin.z) / ray_dir.z;

    // Ensure min < max for each axis
    let (t_min_x, t_max_x) = if t_min_x > t_max_x {
        (t_max_x, t_min_x)
    } else {
        (t_min_x, t_max_x)
    };
    let (t_min_y, t_max_y) = if t_min_y > t_max_y {
        (t_max_y, t_min_y)
    } else {
        (t_min_y, t_max_y)
    };
    let (t_min_z, t_max_z) = if t_min_z > t_max_z {
        (t_max_z, t_min_z)
    } else {
        (t_min_z, t_max_z)
    };

    // Find the intersection range
    let t_min = t_min_x.max(t_min_y).max(t_min_z);
    let t_max = t_max_x.min(t_max_y).min(t_max_z);

    // Check if intersection exists and is in front of the camera
    t_max >= 0.0 && t_min <= t_max
}

/// Finds the closest adjacent grid tile to a box (for OSRS-style object interaction)
fn find_closest_adjacent_tile(box_center: Vec3, from_position: Vec3) -> Vec3 {
    // Grid-snap the box center
    let box_grid = snap_to_grid(box_center);

    // Possible adjacent tiles (4 cardinal directions)
    let adjacent_tiles = [
        Vec3::new(box_grid.x + 1.0, box_center.y, box_grid.z), // East
        Vec3::new(box_grid.x - 1.0, box_center.y, box_grid.z), // West
        Vec3::new(box_grid.x, box_center.y, box_grid.z + 1.0), // North
        Vec3::new(box_grid.x, box_center.y, box_grid.z - 1.0), // South
    ];

    // Find the closest adjacent tile to the click position
    let mut closest_tile = adjacent_tiles[0];
    let mut closest_distance = from_position.distance(closest_tile);

    for tile in &adjacent_tiles[1..] {
        let distance = from_position.distance(*tile);
        if distance < closest_distance {
            closest_distance = distance;
            closest_tile = *tile;
        }
    }

    closest_tile
}

/// Handles right-click movement commands and gathering commands
pub fn handle_movement_command(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    selected_units: Query<Entity, (With<Selected>, With<Controllable>)>,
    static_obstacles: Query<&Transform, With<StaticObstacle>>,
    resource_nodes: Query<(Entity, &Transform, &ResourceNode), With<ResourceNode>>,
    mut gather_events: EventWriter<GatherEvent>,
    mut commands: Commands,
) {
    if !buttons.just_pressed(MouseButton::Right) {
        return;
    }

    // Only move if we have selected units
    if selected_units.is_empty() {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok((camera, cam_transform)) = cameras.single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(cam_transform, cursor_pos) else {
        return;
    };

    // First check if the ray hits any resource nodes
    let mut clicked_resource: Option<Entity> = None;
    for (resource_entity, resource_transform, resource_node) in resource_nodes.iter() {
        // Use larger collision boxes to make resource nodes easier to click
        let resource_size = Vec3::new(1.2, 1.8, 1.2); // Generous size for all resource types
        if ray_box_intersection(ray, resource_transform.translation, resource_size) {
            info!("🎯 Clicked on {} at ({:.1}, {:.1})", 
                resource_node.kind.display_name(), 
                resource_transform.translation.x, 
                resource_transform.translation.z);
            clicked_resource = Some(resource_entity);
            break; // Take the first hit
        }
    }

    // If we clicked on a resource node and have controllable units selected, emit gather events
    if let Some(resource_entity) = clicked_resource {
        let controllable_units: Vec<Entity> = selected_units.iter().collect();
        if !controllable_units.is_empty() {
            info!("⛏️ Gather command: {} units targeting resource entity {:?}", controllable_units.len(), resource_entity);
            
            // Emit gather events for all selected controllable units
            for unit_entity in controllable_units {
                gather_events.write(GatherEvent {
                    unit: unit_entity,
                    resource: resource_entity,
                });
            }
        }
        return; // Don't continue with movement logic if we're gathering
    }

    // Then check if the ray hits any obstacles (boxes)
    let mut clicked_box: Option<Vec3> = None;
    for obstacle_transform in static_obstacles.iter() {
        let box_size = Vec3::new(0.8, 0.5, 0.8); // Size of our boxes
        if ray_box_intersection(ray, obstacle_transform.translation, box_size) {
            clicked_box = Some(obstacle_transform.translation);
            break; // Take the first hit
        }
    }

    let base_destination = if let Some(box_center) = clicked_box {
        // Clicked on a box - find the closest adjacent tile
        let raw_ground_pos = ray_ground_intersection(ray, 0.0).unwrap_or(box_center);
        let adjacent_tile = find_closest_adjacent_tile(box_center, raw_ground_pos);

        info!(
            "📦 Clicked on box at ({:.0}, {:.0}) -> moving to adjacent tile ({:.0}, {:.0})",
            box_center.x, box_center.z, adjacent_tile.x, adjacent_tile.z
        );

        adjacent_tile
    } else {
        // Normal ground click - snap to grid
        if let Some(raw_destination) = ray_ground_intersection(ray, 0.0) {
            let snapped = snap_to_grid(raw_destination);
            info!(
                "🎯 Grid movement command: raw({:.2}, {:.2}) -> grid({:.0}, {:.0})",
                raw_destination.x, raw_destination.z, snapped.x, snapped.z
            );
            snapped
        } else {
            return; // No valid ground intersection
        }
    };

    let selected_entities: Vec<Entity> = selected_units.iter().collect();
    let num_selected = selected_entities.len();

    // Set individual destinations for each selected unit
    for (index, entity) in selected_entities.iter().enumerate() {
        let individual_destination = if num_selected == 1 {
            // Single unit goes to the destination (either adjacent tile or grid-snapped)
            base_destination
        } else {
            // Multiple units get spread out destinations (also grid-snapped)
            snap_to_grid(calculate_formation_position(
                base_destination,
                index,
                num_selected,
            ))
        };

        let mut entity_commands = commands.entity(*entity);
        entity_commands.insert((
            crate::components::Destination {
                target: individual_destination,
            },
            Moving,
        ));

        // Mark the first unit as primary target for priority handling
        if num_selected > 1 && index == 0 {
            entity_commands.insert(crate::components::PrimaryTarget);
        } else {
            // Remove PrimaryTarget if it exists from previous commands
            entity_commands.remove::<crate::components::PrimaryTarget>();
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
    _cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    minimap_query: Query<&Node, With<MinimapUI>>,
    existing_drag: Query<Entity, With<DragSelection>>,
    minimap_drag_state: Res<MinimapDragState>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    // Don't start drag selection if minimap is being dragged
    if minimap_drag_state.is_dragging {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    // Check if click is on minimap - if so, don't start drag selection (minimap has priority)
    if let Ok(_minimap_node) = minimap_query.single() {
        let minimap_rect = Rect::from_corners(
            Vec2::new(window.width() - 200.0, window.height() - 200.0),
            Vec2::new(window.width(), window.height()),
        );

        if minimap_rect.contains(cursor_pos) {
            return; // Don't start drag selection on minimap - let minimap handle it
        }
    }

    // Remove any existing drag selection
    for entity in existing_drag.iter() {
        commands.entity(entity).despawn();
    }

    // Start new drag selection
    let _drag_entity = commands
        .spawn((
            DragSelection {
                start_pos: cursor_pos,
                current_pos: cursor_pos,
            },
            Name::new("DragSelection"),
        ))
        .id();

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

    info!(
        "🖱️ Started drag selection at: ({:.2}, {:.2})",
        cursor_pos.x, cursor_pos.y
    );
}

/// Updates drag selection while dragging
pub fn handle_drag_selection_update(
    mut drag_query: Query<&mut DragSelection>,
    mut box_query: Query<&mut Node, With<DragSelectionBox>>,
    _buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
) {
    if drag_query.is_empty() {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

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
    mut selection_events: EventWriter<SelectionChanged>,
) {
    if !buttons.just_released(MouseButton::Left) || drag_query.is_empty() {
        return;
    }

    let Ok(_window) = windows.single() else {
        return;
    };
    let Ok((camera, cam_transform)) = cameras.single() else {
        return;
    };
    let Ok((drag_entity, drag)) = drag_query.single() else {
        return;
    };

    // Deselect all current selections
    for entity in selected.iter() {
        commands.entity(entity).remove::<Selected>();
    }

    // Calculate selection rectangle
    let min_x = drag.start_pos.x.min(drag.current_pos.x);
    let min_y = drag.start_pos.y.min(drag.current_pos.y);
    let max_x = drag.start_pos.x.max(drag.current_pos.x);
    let max_y = drag.start_pos.y.max(drag.current_pos.y);

    let mut selected_count = 0;
    let mut newly_selected_units: Vec<Entity> = Vec::new();

    // Check if drag was just a click (small area)
    let drag_area = (max_x - min_x) * (max_y - min_y);
    if drag_area < 25.0 {
        // Small area threshold (5x5 pixels) - treat as single click
        let cursor_pos = drag.current_pos;
        let Ok(ray) = camera.viewport_to_world(cam_transform, cursor_pos) else {
            cleanup_drag_selection(&mut commands, drag_entity, &box_query);
            return;
        };

        // Single unit selection with cylinder intersection (existing logic)
        for (entity, unit_transform) in units.iter() {
            let unit_pos = unit_transform.translation();
            let selection_radius = 1.2;
            let selection_height = 2.0;

            if ray_intersects_cylinder(ray, unit_pos, selection_radius, selection_height) {
                commands.entity(entity).insert(Selected);
                newly_selected_units.push(entity);
                selected_count += 1;
                break; // Only select one unit for click
            }
        }
    } else {
        // Drag selection with forgiving buffer
        info!(
            "🎯 Drag selection: ({:.1}, {:.1}) to ({:.1}, {:.1})",
            min_x, min_y, max_x, max_y
        );

        // Slightly enlarge the marquee (±2px) for forgiving selection
        const SELECTION_TOLERANCE: f32 = 2.0; // pixels
        let expanded_rect = Rect::from_corners(
            Vec2::new(min_x - SELECTION_TOLERANCE, min_y - SELECTION_TOLERANCE),
            Vec2::new(max_x + SELECTION_TOLERANCE, max_y + SELECTION_TOLERANCE)
        );

        info!(
            "📏 Expanded selection area: ({:.1}, {:.1}) to ({:.1}, {:.1}) (+{}px buffer)",
            min_x - SELECTION_TOLERANCE, min_y - SELECTION_TOLERANCE,
            max_x + SELECTION_TOLERANCE, max_y + SELECTION_TOLERANCE,
            SELECTION_TOLERANCE
        );

        // Collect units with their screen positions for ordering
        let mut selectable_units = Vec::new();

        for (entity, unit_transform) in units.iter() {
            let unit_world_pos = unit_transform.translation();

            // Convert unit position to screen space for forgiving selection
            if let Ok(screen_pos) = camera.world_to_viewport(cam_transform, unit_world_pos) {
                // Use the expanded rectangle for forgiving selection
                if expanded_rect.contains(screen_pos) {
                    selectable_units.push((entity, screen_pos, unit_world_pos));
                    info!(
                        "✅ Unit at screen ({:.1}, {:.1}) is inside selection area",
                        screen_pos.x, screen_pos.y
                    );
                } else {
                    info!(
                        "❌ Unit at screen ({:.1}, {:.1}) is outside selection area",
                        screen_pos.x, screen_pos.y
                    );
                }
            }
        }

        // Sort by screen Y position (top to bottom priority)
        selectable_units.sort_by(|a, b| {
            a.1.y
                .partial_cmp(&b.1.y)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Selection limit (40 units maximum)
        const MAX_SELECTION: usize = 40;
        let units_to_select = selectable_units.into_iter().take(MAX_SELECTION);

        for (entity, _screen_pos, unit_world_pos) in units_to_select {
            commands.entity(entity).insert(Selected);
            newly_selected_units.push(entity);
            selected_count += 1;
            info!(
                "✅ Selected unit at world pos ({:.1}, {:.1})",
                unit_world_pos.x, unit_world_pos.z
            );
        }
    }

    info!(
        "✅ Drag selection complete: {} units selected",
        selected_count
    );

    // Emit SelectionChanged event
    selection_events.write(SelectionChanged {
        selected_units: newly_selected_units,
    });

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

/// Calculates individual formation positions for multiple selected units
fn calculate_formation_position(
    base_destination: Vec3,
    unit_index: usize,
    total_units: usize,
) -> Vec3 {
    if total_units <= 1 {
        return base_destination;
    }

    // First unit (index 0) ALWAYS gets the exact clicked destination
    if unit_index == 0 {
        return base_destination;
    }

    // All other units get adjacent grid positions
    // Define a tight formation pattern for adjacent tiles
    let adjacent_offsets = vec![
        Vec3::new(1.0, 0.0, 0.0),   // Right
        Vec3::new(-1.0, 0.0, 0.0),  // Left
        Vec3::new(0.0, 0.0, 1.0),   // Forward
        Vec3::new(0.0, 0.0, -1.0),  // Back
        Vec3::new(1.0, 0.0, 1.0),   // Right-Forward
        Vec3::new(-1.0, 0.0, 1.0),  // Left-Forward
        Vec3::new(1.0, 0.0, -1.0),  // Right-Back
        Vec3::new(-1.0, 0.0, -1.0), // Left-Back
        // Second ring if needed
        Vec3::new(2.0, 0.0, 0.0),  // Right 2
        Vec3::new(-2.0, 0.0, 0.0), // Left 2
        Vec3::new(0.0, 0.0, 2.0),  // Forward 2
        Vec3::new(0.0, 0.0, -2.0), // Back 2
        Vec3::new(2.0, 0.0, 1.0),  // Right-Forward 2
        Vec3::new(-2.0, 0.0, 1.0), // Left-Forward 2
        Vec3::new(1.0, 0.0, 2.0),  // Forward-Right 2
        Vec3::new(-1.0, 0.0, 2.0), // Forward-Left 2
    ];

    // Get the offset for this unit (unit_index - 1 because first unit gets exact position)
    let offset_index = (unit_index - 1) % adjacent_offsets.len();
    let offset = adjacent_offsets[offset_index];

    Vec3::new(
        base_destination.x + offset.x,
        base_destination.y,
        base_destination.z + offset.z,
    )
}

// Grid-based movement constants (OSRS style)
const GRID_SIZE: f32 = 1.0; // Size of each grid square

/// Snaps a world position to the nearest grid center (OSRS-style movement)
fn snap_to_grid(position: Vec3) -> Vec3 {
    Vec3::new(
        (position.x / GRID_SIZE).round() * GRID_SIZE,
        position.y, // Keep original Y height
        (position.z / GRID_SIZE).round() * GRID_SIZE,
    )
}

/// Double-click selection - selects all visible units of the same type
pub fn handle_double_click_selection(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    units: Query<(Entity, &GlobalTransform), (With<Controllable>, With<SceneRoot>)>,
    selected: Query<Entity, With<Selected>>,
    mut last_click_time: Local<f32>,
    mut last_click_pos: Local<Vec2>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok((camera, cam_transform)) = cameras.single() else {
        return;
    };

    let current_time = time.elapsed_secs();
    let double_click_threshold = 0.3; // seconds
    let double_click_distance = 20.0; // pixels

    // Check if this is a double-click
    let is_double_click = current_time - *last_click_time < double_click_threshold
        && cursor_pos.distance(*last_click_pos) < double_click_distance;

    *last_click_time = current_time;
    *last_click_pos = cursor_pos;

    if !is_double_click {
        return;
    }

    // Find the unit under the cursor
    let Ok(ray) = camera.viewport_to_world(cam_transform, cursor_pos) else {
        return;
    };

    let mut clicked_unit: Option<Entity> = None;
    for (entity, unit_transform) in units.iter() {
        let unit_pos = unit_transform.translation();
        let selection_radius = 1.2;
        let selection_height = 2.0;

        if ray_intersects_cylinder(ray, unit_pos, selection_radius, selection_height) {
            clicked_unit = Some(entity);
            break;
        }
    }    if let Some(_clicked_entity) = clicked_unit {
        // Select all visible units of the same type
        // For now, we'll select all controllable units on screen (since we don't have unit types yet)
        
        // Deselect current selection
        for entity in selected.iter() {
            commands.entity(entity).remove::<Selected>();
        }
        
        let mut selected_count = 0;
        const MAX_SELECTION: usize = 40; // Selection limit

        for (entity, unit_transform) in units.iter().take(MAX_SELECTION) {
            let unit_pos = unit_transform.translation();
            // Check if unit is visible on screen
            if let Ok(_screen_pos) = camera.world_to_viewport(cam_transform, unit_pos) {
                commands.entity(entity).insert(Selected);
                selected_count += 1;
            }
        }

        info!(
            "🔄 Double-click: Selected {} visible units of same type",
            selected_count
        );
    }
}
