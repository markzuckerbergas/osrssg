use bevy::prelude::*;
use crate::components::{Controllable, Moving};
use crate::resources::UnitAnimations;

pub fn setup_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    info!("🔄 Setting up animation system");
    
    // Create a new animation graph
    let mut animation_graph = AnimationGraph::new();
    
    // Load the walk and idle animations from the GLTF file
    let walk_animation = asset_server.load("player.glb#Animation0");
    let idle_animation = asset_server.load("player.glb#Animation1");
    
    info!("📦 Loading animations: walk={:?}, idle={:?}", walk_animation, idle_animation);
    
    // Add animations to the graph (these return node indices)
    let walk_node = animation_graph.add_clip(walk_animation, 1.0, animation_graph.root);
    let idle_node = animation_graph.add_clip(idle_animation, 1.0, animation_graph.root);
    
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

pub fn setup_animation_players(
    mut commands: Commands,
    query: Query<Entity, (With<Controllable>, With<SceneRoot>, Without<AnimationPlayer>)>,
    animations: Res<UnitAnimations>,
    mut done: Local<bool>,
) {
    if *done || query.is_empty() {
        return;
    }
    
    info!("🎭 Setting up animation players for {} controllable entities", query.iter().count());
    
    for entity in query.iter() {
        info!("🎮 Adding AnimationPlayer to entity {:?}", entity);
        commands.entity(entity).insert((
            AnimationPlayer::default(),
            AnimationGraphHandle(animations.animation_graph.clone()),
        ));
    }
    
    *done = true; // Only run once
}

pub fn animate_units(
    mut query: Query<&mut AnimationPlayer, With<Controllable>>,
    moving_query: Query<Entity, (With<Controllable>, With<Moving>)>,
    animations: Res<UnitAnimations>,
) {
    for mut player in query.iter_mut() {
        // Check if this entity (or any controllable entity) is moving
        let is_moving = !moving_query.is_empty();
        
        if is_moving {
            // Only change animation if not already playing the correct one
            if !player.is_playing_animation(animations.walk_node) {
                info!("🚶 Playing walk animation");
                player.start(animations.walk_node).repeat();
            }
        } else {
            // Only change animation if not already playing the correct one
            if !player.is_playing_animation(animations.idle_node) {
                info!("🧍 Playing idle animation");
                player.start(animations.idle_node).repeat();
            }
        }
    }
}

// Debug system to check animation assets loading (run once only)
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
