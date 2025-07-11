use bevy::prelude::*;
use crate::components::{Controllable, Moving};
use crate::resources::UnitAnimations;

/// Sets up the animation graph for units
pub fn setup_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    info!("🔄 Setting up animation system");
    
    // Load animations using the modern GltfAssetLabel format
    // Animation0 = walk, Animation1 = idle (as per your GLB file)
    let walk_clip = asset_server.load(GltfAssetLabel::Animation(0).from_asset("player.glb"));
    let idle_clip = asset_server.load(GltfAssetLabel::Animation(1).from_asset("player.glb"));
    
    info!("📦 Loading animations: walk={:?}, idle={:?}", walk_clip, idle_clip);
    
    // Create animation graph with both clips
    let mut animation_graph = AnimationGraph::new();
    let walk_node = animation_graph.add_clip(walk_clip, 1.0, animation_graph.root);
    let idle_node = animation_graph.add_clip(idle_clip, 1.0, animation_graph.root);
    
    // Store the animation graph
    let animation_graph_handle = animation_graphs.add(animation_graph);
    
    info!("✅ Animation graph created with walk_node={:?}, idle_node={:?}", walk_node, idle_node);
    
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
    query: Query<Entity, (With<Controllable>, Added<AnimationPlayer>)>,
    animations: Res<UnitAnimations>,
) {
    for entity in query.iter() {
        info!("🎮 Setting up AnimationPlayer for entity {:?}", entity);
        
        // Add the animation graph handle to the entity
        commands.entity(entity).insert(AnimationGraphHandle(animations.animation_graph.clone()));
    }
}

/// Animates units based on their movement state
pub fn animate_units(
    mut players: Query<&mut AnimationPlayer, With<Controllable>>,
    moving_query: Query<Entity, (With<Controllable>, With<Moving>)>,
    animations: Res<UnitAnimations>,
) {
    for mut player in players.iter_mut() {
        // Check if any controllable entity is moving
        let any_moving = !moving_query.is_empty();
        
        if any_moving {
            // Play walk animation if not already playing
            if !player.is_playing_animation(animations.walk_node) {
                info!("🚶 Playing walk animation");
                player.play(animations.walk_node).repeat();
            }
        } else {
            // Play idle animation if not already playing
            if !player.is_playing_animation(animations.idle_node) {
                info!("🧍 Playing idle animation");
                player.play(animations.idle_node).repeat();
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
        info!("✅ Animation system ready: {} nodes, {} clips", 
            graph.nodes().count(), animation_clips.len());
        *done = true;
    }
}
