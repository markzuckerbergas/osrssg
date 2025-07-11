use crate::components::*;
use bevy::prelude::*;

/// Moves units toward their individual destinations with collision detection
pub fn move_units(
    mut moving_units: Query<(&mut Transform, Entity, &Destination, &CollisionRadius, Option<&mut StuckTimer>), With<Moving>>,
    other_units: Query<(&Transform, &CollisionRadius), (With<Controllable>, Without<Moving>)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let move_speed = 2.0; // units per second
    let arrival_threshold = 0.1; // how close to consider "arrived"
    let movement_threshold = 0.01; // minimum movement to not be considered stuck

    // Collect positions to check for collisions
    let mut unit_positions: Vec<(Entity, Vec3, f32)> = Vec::new();
    
    // Add stationary units
    for (transform, collision_radius) in other_units.iter() {
        unit_positions.push((Entity::PLACEHOLDER, transform.translation, collision_radius.radius));
    }
    
    // Add moving units
    for (transform, entity, _, collision_radius, _) in moving_units.iter() {
        unit_positions.push((entity, transform.translation, collision_radius.radius));
    }

    for (mut transform, entity, destination, collision_radius, mut stuck_timer) in moving_units.iter_mut() {
        let current_pos = transform.translation;
        let target_pos = Vec3::new(destination.target.x, current_pos.y, destination.target.z);

        let direction = (target_pos - current_pos).normalize_or_zero();
        let distance = current_pos.distance(target_pos);

        if distance <= arrival_threshold {
            // Arrived - stop moving and remove destination
            info!(
                "✅ Unit arrived at destination ({:.2}, {:.2})",
                target_pos.x, target_pos.z
            );
            commands
                .entity(entity)
                .remove::<Moving>()
                .remove::<Destination>()
                .remove::<StuckTimer>();
        } else {
            // Initialize or update stuck timer
            let stuck_timer = match stuck_timer {
                Some(ref mut timer) => timer,
                None => {
                    // Add stuck timer if it doesn't exist
                    commands.entity(entity).insert(StuckTimer {
                        last_position: current_pos,
                        ..Default::default()
                    });
                    continue; // Skip this frame to let the component be added
                }
            };

            // Calculate desired movement
            let move_distance = move_speed * time.delta_secs();
            let desired_position = current_pos + direction * move_distance;

            // Check for collisions with other units
            let mut can_move = true;
            let mut avoidance_direction = Vec3::ZERO;
            
            for (other_entity, other_pos, other_radius) in &unit_positions {
                // Skip checking collision with self
                if *other_entity == entity {
                    continue;
                }
                
                let distance_to_other = desired_position.distance(*other_pos);
                let combined_radius = collision_radius.radius + other_radius + 0.1; // Add small buffer
                
                if distance_to_other < combined_radius {
                    can_move = false;
                    
                    // Calculate avoidance direction (perpendicular to movement and away from obstacle)
                    let to_obstacle = (*other_pos - current_pos).normalize_or_zero();
                    let perpendicular = Vec3::new(-to_obstacle.z, 0.0, to_obstacle.x);
                    
                    // Choose the perpendicular direction that's closer to the target
                    let option1 = current_pos + perpendicular * move_distance;
                    let option2 = current_pos - perpendicular * move_distance;
                    
                    let dist1 = option1.distance(target_pos);
                    let dist2 = option2.distance(target_pos);
                    
                    avoidance_direction = if dist1 < dist2 { perpendicular } else { -perpendicular };
                    break;
                }
            }
            
            let mut actually_moved = false;
            let old_position = current_pos;
            
            if can_move {
                // Move directly toward target
                transform.translation = desired_position;
                actually_moved = true;
            } else {
                // Try to move around the obstacle
                let avoidance_position = current_pos + avoidance_direction * move_distance;
                
                // Check if avoidance path is clear
                let mut avoidance_clear = true;
                for (other_entity, other_pos, other_radius) in &unit_positions {
                    if *other_entity == entity {
                        continue;
                    }
                    
                    let distance_to_other = avoidance_position.distance(*other_pos);
                    let combined_radius = collision_radius.radius + other_radius + 0.1;
                    
                    if distance_to_other < combined_radius {
                        avoidance_clear = false;
                        break;
                    }
                }
                
                if avoidance_clear {
                    transform.translation = avoidance_position;
                    actually_moved = true;
                }
                // If can't move at all, actually_moved remains false
            }

            // Update stuck timer
            if actually_moved && old_position.distance(transform.translation) > movement_threshold {
                // Unit moved significantly, reset stuck timer
                stuck_timer.timer = 0.0;
                stuck_timer.last_position = transform.translation;
            } else {
                // Unit didn't move or moved very little, increment stuck timer
                stuck_timer.timer += time.delta_secs();
                
                // Check if unit has been stuck too long
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

            // Rotate to face movement direction
            if direction.length() > 0.001 {
                transform.rotation = Quat::from_rotation_y(direction.x.atan2(direction.z));
            }
        }
    }
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
