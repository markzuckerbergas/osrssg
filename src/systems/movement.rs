use crate::components::*;
use bevy::prelude::*;

/// Moves units toward their individual destinations
pub fn move_units(
    mut moving_units: Query<(&mut Transform, Entity, &Destination), With<Moving>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let move_speed = 2.0; // units per second
    let arrival_threshold = 0.1; // how close to consider "arrived"

    for (mut transform, entity, destination) in moving_units.iter_mut() {
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
                .remove::<Destination>();
        } else {
            // Continue moving
            let move_distance = move_speed * time.delta_secs();
            transform.translation += direction * move_distance;

            // Rotate to face movement direction
            if direction.length() > 0.001 {
                transform.rotation = Quat::from_rotation_y(direction.x.atan2(direction.z));
            }
        }
    }
}
