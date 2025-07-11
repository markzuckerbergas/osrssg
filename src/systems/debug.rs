use bevy::prelude::*;

/// Diagnostic system to debug animation issues
pub fn debug_animations(
    animation_players: Query<(Entity, &AnimationPlayer), Added<AnimationPlayer>>,
    animation_graphs: Res<Assets<AnimationGraph>>,
    animations: Res<crate::resources::UnitAnimations>,
) {
    for (entity, player) in animation_players.iter() {
        info!("🎭 New AnimationPlayer found on entity: {:?}", entity);
        
        // Check if the animation graph exists
        if let Some(graph) = animation_graphs.get(&animations.graph) {
            info!("✅ Animation graph found with {} nodes", graph.graph.node_count());
            info!("🚶 Walk animation index: {:?}", animations.walk);
            info!("🧍 Idle animation index: {:?}", animations.idle);
        } else {
            info!("❌ Animation graph not found or not loaded yet");
        }
        
        info!("🎮 AnimationPlayer state: playing={}", player.is_playing());
    }
}

/// Debug system to track what happens when we try to play animations
pub fn debug_animation_playback(
    mut animation_players: Query<&mut AnimationPlayer>,
    animations: Res<crate::resources::UnitAnimations>,
    input: Res<ButtonInput<KeyCode>>,
) {
    // Press 'P' to test playing idle animation
    if input.just_pressed(KeyCode::KeyP) {
        info!("🔄 Testing idle animation playback...");
        for mut player in animation_players.iter_mut() {
            let result = player.play(animations.idle);
            info!("🎵 Play result: {:?}", result);
            result.repeat();
        }
    }
    
    // Press 'W' to test playing walk animation  
    if input.just_pressed(KeyCode::KeyW) {
        info!("🔄 Testing walk animation playback...");
        for mut player in animation_players.iter_mut() {
            let result = player.play(animations.walk);
            info!("🎵 Play result: {:?}", result);
            result.repeat();
        }
    }
}

/// Debug system to check scene loading
pub fn debug_scene_loading(
    scenes: Query<(Entity, &SceneRoot), Added<SceneRoot>>,
    children: Query<&Children>,
) {
    for (entity, _scene) in scenes.iter() {
        info!("🎬 Scene loaded on entity: {:?}", entity);
        
        // Check children recursively
        fn print_children(entity: Entity, children_query: &Query<&Children>, depth: usize) {
            let indent = "  ".repeat(depth);
            if let Ok(children) = children_query.get(entity) {
                for child in children.iter() {
                    info!("{}└─ Child: {:?}", indent, child);
                    print_children(*child, children_query, depth + 1);
                }
            }
        }
        
        print_children(entity, &children, 1);
    }
}
