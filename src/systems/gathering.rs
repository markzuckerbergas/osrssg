use crate::components::*;
use bevy::prelude::*;

/// System that processes GatherEvent and creates GatherTask components
/// This runs right after movement command generation
pub fn issue_gather_task(
    mut gather_events: EventReader<GatherEvent>,
    resource_nodes: Query<(&ResourceNode, &Transform)>,
    mut units: Query<(Entity, &Transform, &mut Inventory), With<Controllable>>,
    mut commands: Commands,
) {
    for gather_event in gather_events.read() {
        let unit_entity = gather_event.unit;
        let resource_entity = gather_event.resource;

        // Get resource node information
        let Ok((resource_node, resource_transform)) = resource_nodes.get(resource_entity) else {
            warn!("⚠️ Gather event references invalid resource entity");
            continue;
        };

        // Get unit information
        let Ok((_, _unit_transform, inventory)) = units.get_mut(unit_entity) else {
            warn!("⚠️ Gather event references invalid unit entity");
            continue;
        };

        // Check if unit's inventory is already full
        if inventory.is_full() {
            info!("📦 Unit inventory is full, cannot gather more resources");
            continue;
        }

        // Remove any existing movement or gather tasks
        let mut unit_commands = commands.entity(unit_entity);
        unit_commands.remove::<Moving>();
        unit_commands.remove::<GatherTask>();
        unit_commands.remove::<Destination>();

        // Create new gather task
        let gather_task = GatherTask::new(resource_entity, resource_node.gather_rate);
        unit_commands.insert(gather_task);

        // Set destination to the resource node position (grid-aligned)
        let target_position = snap_to_grid(resource_transform.translation);
        unit_commands.insert((
            Destination {
                target: target_position,
            },
            Moving,
        ));

        info!(
            "⛏️ Unit assigned gather task for {} at ({:.0}, {:.0})",
            resource_node.kind.display_name(),
            target_position.x,
            target_position.z
        );
    }
}

/// Helper function to snap position to grid (copied from input.rs for consistency)
fn snap_to_grid(position: Vec3) -> Vec3 {
    Vec3::new(
        position.x.round(),
        position.y,
        position.z.round(),
    )
}

/// System that processes the gathering state machine
/// Handles Walking → Harvesting → Full state transitions
pub fn process_gathering_state_machine(
    time: Res<Time>,
    mut units: Query<(Entity, &Transform, &mut GatherTask, &mut Inventory), With<Controllable>>,
    mut resource_nodes: Query<(&mut ResourceNode, &Transform), (With<ResourceNode>, Without<Controllable>)>,
    mut commands: Commands,
) {
    for (unit_entity, unit_transform, mut gather_task, mut inventory) in units.iter_mut() {
        let Ok((mut resource_node, resource_transform)) = resource_nodes.get_mut(gather_task.target) else {
            // Resource node no longer exists, remove gather task
            commands.entity(unit_entity).remove::<GatherTask>();
            continue;
        };

        let distance = unit_transform.translation.distance(resource_transform.translation);

        match gather_task.state {
            GatherState::Walking => {
                // Check if we're close enough to start harvesting
                if distance <= resource_node.gather_radius {
                    gather_task.state = GatherState::Harvesting;
                    // Remove Moving component so unit stops walking
                    commands.entity(unit_entity).remove::<Moving>();
                    info!(
                        "🔨 Unit started harvesting {} (distance: {:.1})",
                        resource_node.kind.display_name(),
                        distance
                    );
                }
            }
            
            GatherState::Harvesting => {
                // Check if we're still in range
                if distance > resource_node.gather_radius {
                    gather_task.state = GatherState::Walking;
                    // Add Moving component back to resume walking
                    commands.entity(unit_entity).insert(Moving);
                    info!("🚶 Unit moved out of range, resuming walk to resource");
                    continue;
                }

                // Check if inventory is full
                if inventory.is_full() {
                    gather_task.state = GatherState::Full;
                    info!("📦 Unit inventory is full, stopping harvest");
                    continue;
                }

                // Check if resource is depleted
                if resource_node.remaining == 0 {
                    // Resource depleted, remove gather task
                    commands.entity(unit_entity).remove::<GatherTask>();
                    info!("💀 Resource depleted, removing gather task");
                    continue;
                }

                // Process gathering timer
                gather_task.timer.tick(time.delta());
                if gather_task.timer.just_finished() {
                    // Calculate how much we can gather (limited by resource remaining)
                    let gather_amount = resource_node.gather_rate.min(resource_node.remaining as f32) as u16;
                    
                    if gather_amount > 0 {
                        // Convert resource type to item
                        let item_id = ItemId::from(resource_node.kind);
                        
                        // Add to inventory (respecting stacking rules)
                        let max_stack = item_id.max_stack_size();
                        let added = inventory.add_items(item_id, gather_amount, max_stack);
                        
                        if added > 0 {
                            // Reduce resource node remaining
                            resource_node.remaining = resource_node.remaining.saturating_sub(added as u32);
                            
                            info!(
                                "⛏️ Gathered {} {} (remaining: {}, inventory: {}/28 slots)", 
                                added,
                                resource_node.kind.display_name(),
                                resource_node.remaining,
                                inventory.used_slots()
                            );
                        }
                    }
                }
            }
            
            GatherState::Full => {
                // Unit is idle with full inventory - wait for player command
                // Could add visual feedback here (different animation, etc.)
            }
        }
    }
}
