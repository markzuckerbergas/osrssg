use bevy::prelude::*;
use crate::{components::*, resources::GameState};

/// Moves units toward their destination
pub fn move_units(
    mut moving_units: Query<(&mut Transform, Entity), With<Moving>>,
    game_state: Res<GameState>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let Some(destination) = game_state.move_destination else { return; };
    let move_speed = 2.0; // units per second
    let arrival_threshold = 0.1; // how close to consider "arrived"

    for (mut transform, entity) in moving_units.iter_mut() {
        let current_pos = transform.translation;
        let target_pos = Vec3::new(destination.x, current_pos.y, destination.z);
        
        let direction = (target_pos - current_pos).normalize_or_zero();
        let distance = current_pos.distance(target_pos);
        
        if distance <= arrival_threshold {
            // Arrived - stop moving
            commands.entity(entity).remove::<Moving>();
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
