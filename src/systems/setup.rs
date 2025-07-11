use crate::{components::*, resources::*};
use bevy::{gltf::GltfAssetLabel, prelude::*, render::camera::ScalingMode};
use rand::Rng;

/// Sets up the game scene
pub fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Insert resources
    commands.insert_resource(GameState::default());
    commands.insert_resource(CameraSettings::default());
    commands.insert_resource(MinimapSettings::default());

    // Setup will be handled by the animation graph system
    // Remove the old UnitAnimations resource setup from here

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

    // Spawn multiple player units with proper spacing
    let player_scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset("player.glb"));
    let mut rng = rand::thread_rng();

    // Number of characters to spawn
    let num_characters = 5;
    let min_distance = 1.0; // Minimum distance between characters
    info!("🎭 Spawning {} player characters", num_characters);

    let mut spawn_positions = Vec::new();

    for i in 0..num_characters {
        let mut attempts = 0;
        let max_attempts = 50;
        let mut valid_position = None;

        // Try to find a valid spawn position that doesn't conflict with existing characters
        while attempts < max_attempts {
            let x = rng.gen_range(-8.0..8.0);
            let z = rng.gen_range(-8.0..8.0);
            let potential_pos = Vec3::new(x, 0.05, z);

            // Check distance to all previously spawned characters
            let mut too_close = false;
            for existing_pos in &spawn_positions {
                if potential_pos.distance(*existing_pos) < min_distance {
                    too_close = true;
                    break;
                }
            }

            if !too_close {
                valid_position = Some(potential_pos);
                break;
            }

            attempts += 1;
        }

        // Use the valid position, or fallback to a grid-based position if we couldn't find one
        let final_position = valid_position.unwrap_or_else(|| {
            let row = i / 3;
            let col = i % 3;
            Vec3::new(
                (col as f32 - 1.0) * 1.5,
                0.05,
                (row as f32 - 1.0) * 1.5,
            )
        });

        spawn_positions.push(final_position);

        let character_transform = Transform {
            translation: final_position,
            scale: Vec3::splat(0.03),
            ..default()
        };

        info!(
            "👤 Spawning character {} at position ({:.2}, {:.2})",
            i + 1,
            final_position.x,
            final_position.z
        );

        commands.spawn((
            SceneRoot(player_scene.clone()),
            character_transform,
            Controllable,
            CollisionRadius::default(),
        ));
    }

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::default(),
    ));

    // Main isometric camera
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
        MainCamera,
    ));
}
