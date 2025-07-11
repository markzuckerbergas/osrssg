use crate::components::{Controllable, Moving, UnitAnimationPlayer};
use crate::resources::UnitAnimations;
use bevy::prelude::*;

/// Sets up the animation graph for units
pub fn setup_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    info!("🔄 Setting up animation system");

    // Load animations using the modern GltfAssetLabel format
    // Animation0 = walk, Animation1 = idle
    let animation_0 = asset_server.load(GltfAssetLabel::Animation(0).from_asset("player.glb")); // walk
    let animation_1 = asset_server.load(GltfAssetLabel::Animation(1).from_asset("player.glb")); // idle

    info!(
        "📦 Loading animations: anim0={:?}, anim1={:?}",
        animation_0, animation_1
    );

    // Create animation graph with both clips
    // Animation0 = walk, Animation1 = idle
    // Assign them correctly: walk_node gets animation_0, idle_node gets animation_1
    let mut animation_graph = AnimationGraph::new();
    let walk_node = animation_graph.add_clip(animation_0, 1.0, animation_graph.root); // animation_0 = walk
    let idle_node = animation_graph.add_clip(animation_1, 1.0, animation_graph.root); // animation_1 = idle

    // Store the animation graph
    let animation_graph_handle = animation_graphs.add(animation_graph);

    info!(
        "✅ Animation graph created with walk_node={:?}, idle_node={:?}",
        walk_node, idle_node
    );

    // Store the animations in a resource for easy access
    commands.insert_resource(UnitAnimations {
        walk_node,
        idle_node,
        animation_graph: animation_graph_handle,
    });
}

/// Sets up animation players for newly spawned controllable entities
/// This runs when AnimationPlayer components are automatically added by the GLTF loader
pub fn setup_animation_players(
    mut commands: Commands,
    // Look for any newly added AnimationPlayer components
    mut new_players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    // Find all controllable entities and their children to map relationships
    controllable_query: Query<(Entity, &Children), With<Controllable>>,
    children_query: Query<&Children>,
    animations: Res<UnitAnimations>,
) {
    for (player_entity, mut player) in new_players.iter_mut() {
        info!(
            "🎮 Setting up AnimationPlayer for entity {:?}",
            player_entity
        );

        // FIRST: Add the animation graph handle immediately
        commands
            .entity(player_entity)
            .insert(AnimationGraphHandle(animations.animation_graph.clone()));

        // Immediately stop any default animations that might be playing
        player.stop_all();

        // IMMEDIATELY start idle animation before any linking
        player.play(animations.idle_node);

        // Find which controllable unit this AnimationPlayer belongs to
        let mut linked = false;
        for (unit_entity, children) in controllable_query.iter() {
            // Recursively search for the AnimationPlayer in the hierarchy
            if find_entity_in_hierarchy(player_entity, children, &children_query, 0) {
                commands
                    .entity(player_entity)
                    .insert(UnitAnimationPlayer { unit_entity });
                linked = true;
                break;
            }
        }

        // If no specific unit found, that's okay - we already added the graph and started idle
        if !linked {
            info!(
                "⚠️ No parent unit found for AnimationPlayer {:?}",
                player_entity
            );
        }
    }
}

/// Recursively search for an entity in a hierarchy
fn find_entity_in_hierarchy(
    target: Entity,
    current_children: &Children,
    children_query: &Query<&Children>,
    depth: usize,
) -> bool {
    if depth > 10 {
        // Prevent infinite recursion
        return false;
    }

    // Check direct children first
    if current_children.contains(&target) {
        return true;
    }

    // Check children's children
    for child in current_children.iter() {
        if let Ok(grandchildren) = children_query.get(child) {
            if find_entity_in_hierarchy(target, grandchildren, children_query, depth + 1) {
                return true;
            }
        }
    }

    false
}

/// Animates units based on their movement state
pub fn animate_units(
    mut players: Query<(&mut AnimationPlayer, &UnitAnimationPlayer), With<AnimationGraphHandle>>,
    moving_query: Query<Entity, (With<Controllable>, With<Moving>)>,
    all_units_query: Query<Entity, With<Controllable>>,
    animations: Res<UnitAnimations>,
) {
    for (mut player, unit_link) in players.iter_mut() {
        // Verify the unit still exists
        if !all_units_query.contains(unit_link.unit_entity) {
            continue;
        }

        // Check if THIS specific unit is moving
        let is_moving = moving_query.contains(unit_link.unit_entity);

        // Debug: Check what animations are currently playing
        let playing_walk = player.is_playing_animation(animations.walk_node);
        let playing_idle = player.is_playing_animation(animations.idle_node);

        // If no animation is playing at all, start with idle
        if !playing_walk && !playing_idle {
            player.play(animations.idle_node);
            continue;
        }

        if is_moving {
            if !playing_walk {
                player.stop_all();
                player.play(animations.walk_node);
            }
        } else {
            if !playing_idle {
                player.stop_all();
                player.play(animations.idle_node);
            }
        }
    }
}

/// Debug system to check animation system status
pub fn debug_animation_assets(
    animations: Res<UnitAnimations>,
    animation_clips: Res<Assets<AnimationClip>>,
    graphs: Res<Assets<AnimationGraph>>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }

    if let Some(graph) = graphs.get(&animations.animation_graph) {
        info!(
            "✅ Animation system ready: {} nodes, {} clips",
            graph.nodes().count(),
            animation_clips.len()
        );
        *done = true;
    }
}

/// Debug system to track Moving component changes
pub fn debug_moving_components(
    moving_query: Query<Entity, (With<Controllable>, Added<Moving>)>,
    mut removed_moving: RemovedComponents<Moving>,
    controllable_query: Query<Entity, With<Controllable>>,
) {
    // Log when Moving components are added
    for entity in moving_query.iter() {
        info!("🟢 Moving component ADDED to entity {:?}", entity);
    }

    // Log when Moving components are removed
    for entity in removed_moving.read() {
        if controllable_query.contains(entity) {
            info!("🔴 Moving component REMOVED from entity {:?}", entity);
        }
    }
}

/// Debug system to track when controllable entities are spawned
pub fn debug_entity_spawning(controllable_query: Query<Entity, Added<Controllable>>) {
    for entity in controllable_query.iter() {
        info!("🆕 Controllable entity SPAWNED: {:?}", entity);
    }
}
