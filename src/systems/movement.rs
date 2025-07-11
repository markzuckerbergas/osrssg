use crate::components::*;
use bevy::prelude::*;

/// Moves units toward their individual destinations with destination collision prevention only
pub fn move_units(
    mut moving_units: Query<(&mut Transform, Entity, &Destination, Option<&mut StuckTimer>, Option<&crate::components::PrimaryTarget>), With<Moving>>,
    stationary_units: Query<&Transform, (With<Controllable>, Without<Moving>)>,
    static_obstacles: Query<&Transform, (With<StaticObstacle>, Without<Controllable>, Without<Moving>)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let move_speed = 2.0; // units per second
    let arrival_threshold = 0.2; // how close to consider "arrived" (larger for grid-based movement)

    // Collect destination positions to prevent final position conflicts
    let mut occupied_destinations: Vec<Vec3> = Vec::new();
    
    // Add current positions of stationary units (they occupy their current position)
    for transform in stationary_units.iter() {
        let grid_pos = snap_to_grid(transform.translation);
        occupied_destinations.push(grid_pos);
    }
    
    // Process moving units to determine who gets priority for contested destinations
    let mut unit_data: Vec<(Entity, Vec3, Vec3, f32, bool)> = Vec::new(); // (entity, current_pos, target_pos, distance, is_primary)
    
    for (transform, entity, destination, _, primary_target) in moving_units.iter() {
        let current_pos = transform.translation;
        let target_pos = Vec3::new(destination.target.x, current_pos.y, destination.target.z);
        let distance = current_pos.distance(target_pos);
        let is_primary = primary_target.is_some();
        unit_data.push((entity, current_pos, target_pos, distance, is_primary));
    }
    
    // Sort by primary target first (they get absolute priority), then by distance
    unit_data.sort_by(|a, b| {
        match (a.4, b.4) {
            (true, false) => std::cmp::Ordering::Less,  // Primary target goes first
            (false, true) => std::cmp::Ordering::Greater, // Non-primary goes after
            _ => a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal), // Same priority level, sort by distance
        }
    });
    
    // Assign destinations in order of priority (primary target first, then closest first)
    let mut assigned_destinations: Vec<Vec3> = occupied_destinations.clone();
    let mut unit_final_destinations: std::collections::HashMap<Entity, Vec3> = std::collections::HashMap::new();
    
    for (entity, current_pos, target_pos, _distance, _is_primary) in unit_data {
        let grid_target = snap_to_grid(target_pos);
        
        // Check if target destination is available
        let destination_available = !assigned_destinations.iter()
            .any(|&occupied_pos| occupied_pos.distance(grid_target) < 0.1);
        
        if destination_available {
            // Assign the exact target (closest unit gets priority)
            assigned_destinations.push(grid_target);
            unit_final_destinations.insert(entity, grid_target);
        } else {
            // Find alternative position prioritizing formation cohesion
            let nearby_positions = generate_formation_alternatives(grid_target, current_pos);
            
            let mut found_alternative = false;
            for &alternative_pos in &nearby_positions {
                let alternative_available = !assigned_destinations.iter()
                    .any(|&occupied_pos| occupied_pos.distance(alternative_pos) < 0.1);
                
                if alternative_available {
                    assigned_destinations.push(alternative_pos);
                    unit_final_destinations.insert(entity, alternative_pos);
                    found_alternative = true;
                    break;
                }
            }
            
            if !found_alternative {
                // No alternatives found, assign current position (stay put)
                let current_grid = snap_to_grid(current_pos);
                unit_final_destinations.insert(entity, current_grid);
            }
        }
    }

    for (mut transform, entity, destination, stuck_timer, _primary_target) in moving_units.iter_mut() {
        let current_pos = transform.translation;
        let original_target = Vec3::new(destination.target.x, current_pos.y, destination.target.z);
        
        // Get the pre-assigned final destination for this unit
        let final_destination = unit_final_destinations.get(&entity)
            .copied()
            .unwrap_or_else(|| snap_to_grid(original_target));

        // Move toward the FINAL destination, not the original target
        let direction = (Vec3::new(final_destination.x, current_pos.y, final_destination.z) - current_pos).normalize_or_zero();
        let distance = current_pos.distance(Vec3::new(final_destination.x, current_pos.y, final_destination.z));

        if distance <= arrival_threshold {
            // Smoothly arrive at the final destination (no teleporting)
            transform.translation = Vec3::new(final_destination.x, current_pos.y, final_destination.z);
            
            if final_destination.distance(snap_to_grid(original_target)) < 0.1 {
                info!(
                    "✅ Unit arrived at exact destination ({:.0}, {:.0})",
                    final_destination.x, final_destination.z
                );
            } else {
                info!(
                    "✅ Unit arrived at alternative position ({:.0}, {:.0}) near target ({:.0}, {:.0})",
                    final_destination.x, final_destination.z, original_target.x, original_target.z
                );
            }
            
            // Remove movement components
            commands
                .entity(entity)
                .remove::<Moving>()
                .remove::<Destination>()
                .remove::<StuckTimer>();
        } else {
            // Still moving toward destination - check for box collisions during transit
            // Initialize stuck timer if needed
            if stuck_timer.is_none() {
                commands.entity(entity).insert(StuckTimer {
                    last_position: current_pos,
                    ..Default::default()
                });
                continue; // Skip this frame to let the component be added
            }
            
            let mut stuck_timer = stuck_timer.unwrap();
            
            // Calculate movement
            let move_distance = move_speed * time.delta_secs();
            let desired_position = current_pos + direction * move_distance;
            
            // Check for box collisions during transit
            let mut final_position = desired_position;
            let unit_radius = 0.3; // Unit collision radius
            
            for obstacle_transform in static_obstacles.iter() {
                let box_center = obstacle_transform.translation;
                let box_size = Vec3::new(0.8, 0.5, 0.8); // Size of our boxes
                let box_radius = (box_size.x + box_size.z) * 0.25; // Approximate radius for collision
                
                let distance_to_box = desired_position.distance(box_center);
                
                if distance_to_box < (unit_radius + box_radius) {
                    // Collision detected - push unit away from box
                    let push_direction = (desired_position - box_center).normalize_or_zero();
                    let safe_distance = unit_radius + box_radius + 0.1; // Add small buffer
                    final_position = box_center + push_direction * safe_distance;
                    
                    // If pushed position is further from target, try to slide around the box
                    if final_position.distance(Vec3::new(final_destination.x, current_pos.y, final_destination.z)) > 
                       current_pos.distance(Vec3::new(final_destination.x, current_pos.y, final_destination.z)) {
                        
                        // Try sliding perpendicular to the box
                        let to_box = (box_center - current_pos).normalize_or_zero();
                        let perpendicular = Vec3::new(-to_box.z, 0.0, to_box.x);
                        
                        let slide_option1 = current_pos + perpendicular * move_distance;
                        let slide_option2 = current_pos - perpendicular * move_distance;
                        
                        // Choose the slide direction that gets us closer to target
                        let target_3d = Vec3::new(final_destination.x, current_pos.y, final_destination.z);
                        if slide_option1.distance(target_3d) < slide_option2.distance(target_3d) {
                            final_position = slide_option1;
                        } else {
                            final_position = slide_option2;
                        }
                        
                        // Check if sliding position also collides
                        if final_position.distance(box_center) < (unit_radius + box_radius) {
                            // Can't slide, just stop
                            final_position = current_pos;
                        }
                    }
                    break; // Only handle one collision at a time
                }
            }
            
            let new_position = final_position;
            
            // Check if unit is making progress (for stuck detection)
            let movement_threshold = 0.01;
            if current_pos.distance(new_position) > movement_threshold {
                // Unit is moving, reset stuck timer
                stuck_timer.timer = 0.0;
                stuck_timer.last_position = new_position;
            } else {
                // Unit isn't moving much, increment stuck timer
                stuck_timer.timer += time.delta_secs();
                
                // Check if unit has been stuck too long (e.g., blocked by static obstacle)
                if stuck_timer.timer > stuck_timer.stuck_threshold {
                    info!("🚫 Unit stuck for {:.1}s, cancelling movement", stuck_timer.timer);
                    commands
                        .entity(entity)
                        .remove::<Moving>()
                        .remove::<Destination>()
                        .remove::<StuckTimer>();
                    continue;
                }
            }
            
            // Move the unit (no collision checking with other units during transit)
            transform.translation = new_position;

            // Rotate to face movement direction
            if direction.length() > 0.001 {
                transform.rotation = Quat::from_rotation_y(direction.x.atan2(direction.z));
            }
        }
    }
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

/// Generates alternative positions that prioritize formation cohesion
fn generate_formation_alternatives(target: Vec3, current_pos: Vec3) -> Vec<Vec3> {
    let mut alternatives = vec![
        // Ring 1: Immediate neighbors (prioritize these for tight formations)
        target + Vec3::new(1.0, 0.0, 0.0),  // East
        target + Vec3::new(-1.0, 0.0, 0.0), // West
        target + Vec3::new(0.0, 0.0, 1.0),  // North
        target + Vec3::new(0.0, 0.0, -1.0), // South
        target + Vec3::new(1.0, 0.0, 1.0),  // Northeast
        target + Vec3::new(-1.0, 0.0, 1.0), // Northwest
        target + Vec3::new(1.0, 0.0, -1.0), // Southeast
        target + Vec3::new(-1.0, 0.0, -1.0), // Southwest
        
        // Ring 2: Slightly further (for larger groups)
        target + Vec3::new(2.0, 0.0, 0.0),  // East 2
        target + Vec3::new(-2.0, 0.0, 0.0), // West 2
        target + Vec3::new(0.0, 0.0, 2.0),  // North 2
        target + Vec3::new(0.0, 0.0, -2.0), // South 2
        target + Vec3::new(2.0, 0.0, 1.0),  // Northeast variants
        target + Vec3::new(1.0, 0.0, 2.0),
        target + Vec3::new(-2.0, 0.0, 1.0), // Northwest variants
        target + Vec3::new(-1.0, 0.0, 2.0),
        target + Vec3::new(2.0, 0.0, -1.0), // Southeast variants
        target + Vec3::new(1.0, 0.0, -2.0),
        target + Vec3::new(-2.0, 0.0, -1.0), // Southwest variants
        target + Vec3::new(-1.0, 0.0, -2.0),
    ];
    
    // Sort by distance to target first (to maintain formation), then by distance to current position
    alternatives.sort_by(|a, b| {
        let target_dist_a = target.distance(*a);
        let target_dist_b = target.distance(*b);
        
        // Primary sort: distance to formation target
        match target_dist_a.partial_cmp(&target_dist_b) {
            Some(std::cmp::Ordering::Equal) => {
                // Secondary sort: distance to current position (for efficiency)
                let current_dist_a = current_pos.distance(*a);
                let current_dist_b = current_pos.distance(*b);
                current_dist_a.partial_cmp(&current_dist_b).unwrap_or(std::cmp::Ordering::Equal)
            }
            other => other.unwrap_or(std::cmp::Ordering::Equal)
        }
    });
    
    alternatives
}

// Debug system to visualize collision circles (optional)
// Note: Commented out due to gizmo API changes - can be re-implemented later
/*
pub fn debug_collision_circles(
    units: Query<(&Transform, &CollisionRadius), With<Controllable>>,
    mut gizmos: Gizmos,
) {
    for (transform, collision) in units.iter() {
        // TODO: Update to use correct gizmo API for current Bevy version
    }
}
*/
