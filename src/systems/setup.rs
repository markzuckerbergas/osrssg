use bevy::{
    animation::graph::AnimationGraph,
    gltf::GltfAssetLabel,
    prelude::*,
    render::camera::ScalingMode,
};
use crate::{components::*, resources::*};

/// Sets up the game scene
pub fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Insert resources
    commands.insert_resource(GameState::default());
    commands.insert_resource(CameraSettings::default());

    // Lighting setup
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.3,
        affects_lightmapped_meshes: true,
    });

    // Main directional light (sun)
    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 5000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-2.0, 6.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Setup animations
    let walk_animation = asset_server.load(GltfAssetLabel::Animation(0).from_asset("player.glb"));
    let idle_animation = asset_server.load(GltfAssetLabel::Animation(1).from_asset("player.glb"));
    
    let (graph, animation_indices) = AnimationGraph::from_clips([walk_animation, idle_animation]);
    animation_graphs.add(graph);
    
    let unit_animations = UnitAnimations {
        walk: animation_indices[0],
        idle: animation_indices[1],
    };

    // Spawn player unit
    let player_scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset("player.glb"));
    commands.spawn((
        SceneRoot(player_scene),
        Transform {
            translation: Vec3::new(0.0, 0.05, 0.0),
            scale: Vec3::splat(0.03),
            ..default()
        },
        Controllable,
        unit_animations,
    ));

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::default(),
    ));

    // Isometric camera
    commands.spawn((
        Camera3d::default(),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 2.0,
            },
            scale: 5.0,
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
