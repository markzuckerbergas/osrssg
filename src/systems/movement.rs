use crate::components::*;
use bevy::prelude::*;
use rand::Rng;

/// Moves units toward their individual destinations with collision detection
pub fn move_units(
    mut moving_units: Query<(&mut Transform, Entity, &Destination, &CollisionRadius, Option<&mut StuckTimer>), With<Moving>>,
    other_units: Query<(&Transform, &CollisionRadius), (With<Controllable>, Without<Moving>)>,
    static_obstacles: Query<(&Transform, &CollisionRadius), (With<StaticObstacle>, Without<Moving>)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let move_speed = 2.0; // units per second
    let arrival_threshold = 0.2; // how close to consider "arrived" (larger for grid-based movement)
    let movement_threshold = 0.01; // minimum movement to not be considered stuck

    // Collect positions to check for collisions
    let mut unit_positions: Vec<(Entity, Vec3, f32)> = Vec::new();
    
    // Add stationary units
    for (transform, collision_radius) in other_units.iter() {
        unit_positions.push((Entity::PLACEHOLDER, transform.translation, collision_radius.radius));
    }
    
    // Add static obstacles
    for (transform, collision_radius) in static_obstacles.iter() {
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
            let mut final_position = desired_position;
            
            for (other_entity, other_pos, other_radius) in &unit_positions {
                // Skip checking collision with self
                if *other_entity == entity {
                    continue;
                }
                
                let distance_to_other = desired_position.distance(*other_pos);
                let combined_radius = collision_radius.radius + other_radius + 0.1; // Add small buffer
                
                if distance_to_other < combined_radius {
                    can_move = false;
                    
                    // Try multiple avoidance strategies with randomization to break deadlocks
                    let mut rng = rand::thread_rng();
                    let random_factor = rng.gen_range(-0.3..0.3); // Add some randomness
                    let entity_seed = entity.index() as f32 * 0.1; // Use entity ID for consistent but different behavior
                    
                    let avoidance_attempts = [
                        // Strategy 1: Perpendicular avoidance with randomization
                        {
                            let to_obstacle = (*other_pos - current_pos).normalize_or_zero();
                            let perpendicular = Vec3::new(-to_obstacle.z, 0.0, to_obstacle.x);
                            let option1 = current_pos + (perpendicular + Vec3::new(random_factor, 0.0, entity_seed)) * move_distance;
                            let option2 = current_pos + (-perpendicular + Vec3::new(-random_factor, 0.0, -entity_seed)) * move_distance;
                            let dist1 = option1.distance(target_pos);
                            let dist2 = option2.distance(target_pos);
                            if dist1 < dist2 { option1 } else { option2 }
                        },
                        // Strategy 2: Step back and try again (with slight randomization)
                        current_pos + direction * (move_distance * (0.2 + random_factor.abs())),
                        // Strategy 3: Move at an angle towards target (randomized angle)
                        {
                            let to_target = (target_pos - current_pos).normalize_or_zero();
                            let angle_offset = (std::f32::consts::PI / 4.0) + random_factor + entity_seed;
                            let rotated_dir = Vec3::new(
                                to_target.x * angle_offset.cos() - to_target.z * angle_offset.sin(),
                                0.0,
                                to_target.x * angle_offset.sin() + to_target.z * angle_offset.cos(),
                            );
                            current_pos + rotated_dir * move_distance
                        },
                        // Strategy 4: Try the opposite randomized angle
                        {
                            let to_target = (target_pos - current_pos).normalize_or_zero();
                            let angle_offset = -(std::f32::consts::PI / 4.0) - random_factor - entity_seed;
                            let rotated_dir = Vec3::new(
                                to_target.x * angle_offset.cos() - to_target.z * angle_offset.sin(),
                                0.0,
                                to_target.x * angle_offset.sin() + to_target.z * angle_offset.cos(),
                            );
                            current_pos + rotated_dir * move_distance
                        },
                        // Strategy 5: Random lateral movement (emergency deadlock breaker)
                        {
                            let random_angle = rng.gen_range(0.0..2.0 * std::f32::consts::PI);
                            let random_dir = Vec3::new(random_angle.cos(), 0.0, random_angle.sin());
                            current_pos + random_dir * (move_distance * 0.5)
                        },
                    ];
                    
                    // Try each avoidance strategy
                    for attempt_pos in &avoidance_attempts {
                        let mut attempt_clear = true;
                        for (check_entity, check_pos, check_radius) in &unit_positions {
                            if *check_entity == entity {
                                continue;
                            }
                            let check_distance = attempt_pos.distance(*check_pos);
                            let check_combined_radius = collision_radius.radius + check_radius + 0.1;
                            if check_distance < check_combined_radius {
                                attempt_clear = false;
                                break;
                            }
                        }
                        
                        if attempt_clear {
                            final_position = *attempt_pos;
                            can_move = true;
                            break;
                        }
                    }
                    
                    if can_move {
                        break; // Found a solution, no need to check other obstacles
                    }
                }
            }
            
            let mut actually_moved = false;
            let old_position = current_pos;
            
            if can_move {
                // Move to the calculated position (either direct or avoidance)
                transform.translation = final_position;
                actually_moved = true;
            }
            // If completely blocked, actually_moved remains false

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
