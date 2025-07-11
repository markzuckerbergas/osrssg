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
        brightness: 0.8,
        affects_lightmapped_meshes: false,
    });

    // Directional light
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.95, 0.85),
            illuminance: 8000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-1.0, 10.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
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

    // Simple road with grass on the sides
    
    // Main grass areas (left and right of road)
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(7.0, 20.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.15, 0.45, 0.1), // Darker grass green
            metallic: 0.0,
            perceptual_roughness: 1.0,
            reflectance: 0.0,
            ..default()
        })),
        Transform::from_xyz(-6.5, 0.0, 0.0),
    ));
    
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(7.0, 20.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.15, 0.45, 0.1), // Darker grass green
            metallic: 0.0,
            perceptual_roughness: 1.0,
            reflectance: 0.0,
            ..default()
        })),
        Transform::from_xyz(6.5, 0.0, 0.0),
    ));
    
    // Small road in the middle
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(4.0, 20.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.4, 0.4, 0.4),
            metallic: 0.0,
            perceptual_roughness: 0.8,
            reflectance: 0.0,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.01, 0.0),
    ));

    // Add some boxes as obstacles
    let box_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.4, 0.2), // Brown wooden color
        metallic: 0.0,
        perceptual_roughness: 0.8,
        reflectance: 0.0,
        ..default()
    });

    // Create several boxes scattered around the scene
    let box_positions = [
        Vec3::new(2.0, 0.25, 1.0),   // On the road
        Vec3::new(-3.0, 0.25, -2.0), // On grass
        Vec3::new(4.0, 0.25, -3.0),  // On grass
        Vec3::new(-1.0, 0.25, 3.0),  // On road
        Vec3::new(5.5, 0.25, 2.0),   // On grass
        Vec3::new(-5.0, 0.25, 0.0),  // On grass
    ];

    for (i, position) in box_positions.iter().enumerate() {
        info!("📦 Spawning obstacle box {} at position ({:.2}, {:.2})", i + 1, position.x, position.z);
        
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.8, 0.5, 0.8))), // 0.8x0.5x0.8 box
            MeshMaterial3d(box_material.clone()),
            Transform::from_translation(*position),
            StaticObstacle, // Mark as static obstacle for collision detection
            CollisionRadius { radius: 0.6 }, // Slightly larger collision radius than visual size
        ));
    }

    // Main isometric camera
    commands.spawn((
        Camera3d::default(),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 2.0,
            },
            scale: 8.0,
            near: -50.0,  // Extended near plane (negative for orthographic)
            far: 50.0,    // Extended far plane
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainCamera,
    ));
}
