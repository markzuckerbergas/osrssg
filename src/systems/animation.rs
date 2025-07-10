use bevy::prelude::*;
use crate::components::*;

/// Handles animations when units start or stop moving
pub fn update_movement_animations(
    mut animation_players: Query<&mut AnimationPlayer>,
    unit_animations: Query<&UnitAnimations>,
    children: Query<&Children>,
    // Units that just started moving
    started_moving: Query<Entity, Added<Moving>>,
    // Units that just stopped moving
    mut removed_moving: RemovedComponents<Moving>,
) {
    // Handle units that started moving
    for entity in started_moving.iter() {
        if let (Ok(animations), Ok(children_list)) = (unit_animations.get(entity), children.get(entity)) {
            for child_entity in children_list.iter() {
                if let Ok(mut player) = animation_players.get_mut(child_entity) {
                    player.play(animations.walk).repeat();
                }
            }
        }
    }

    // Handle units that stopped moving
    for entity in removed_moving.read() {
        if let (Ok(animations), Ok(children_list)) = (unit_animations.get(entity), children.get(entity)) {
            for child_entity in children_list.iter() {
                if let Ok(mut player) = animation_players.get_mut(child_entity) {
                    player.play(animations.idle).repeat();
                }
            }
        }
    }
}

/// Sets up initial idle animations for newly created animation players
pub fn setup_initial_animations(
    mut animation_players: Query<&mut AnimationPlayer, Added<AnimationPlayer>>,
    unit_animations: Query<&UnitAnimations, With<SceneRoot>>,
) {
    // Start with idle animation for all new players
    for mut player in animation_players.iter_mut() {
        // Get the first available unit animations (assumes single unit for now)
        if let Ok(animations) = unit_animations.single() {
            player.play(animations.idle).repeat();
        }
    }
}
