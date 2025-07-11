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
