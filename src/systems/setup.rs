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

    // Define box positions early so we can check against them during character spawning
    let box_positions = [
        Vec3::new(2.0, 0.25, 1.0),   // On the road
        Vec3::new(-3.0, 0.25, -2.0), // On grass
        Vec3::new(4.0, 0.25, -3.0),  // On grass
        Vec3::new(-1.0, 0.25, 3.0),  // On road
        Vec3::new(6.0, 0.25, 2.0),   // On grass (adjusted to grid)
        Vec3::new(-5.0, 0.25, 0.0),  // On grass
    ];

    // Number of characters to spawn
    let num_characters = 3;
    let min_distance = 1.0; // Minimum distance between characters
    let box_clearance = 1.0; // Minimum distance from boxes
    info!("🎭 Spawning {} player characters", num_characters);

    let mut spawn_positions = Vec::new();

    for i in 0..num_characters {
        let mut attempts = 0;
        let max_attempts = 50;
        let mut valid_position = None;

        // Try to find a valid spawn position that doesn't conflict with existing characters or boxes
        while attempts < max_attempts {
            let x = rng.gen_range(-8..8) as f32; // Integer grid coordinates
            let z = rng.gen_range(-8..8) as f32;
            let potential_pos = Vec3::new(x, 0.05, z); // Grid-aligned position

            let mut position_valid = true;

            // Check distance to all previously spawned characters
            for existing_pos in &spawn_positions {
                if potential_pos.distance(*existing_pos) < min_distance {
                    position_valid = false;
                    break;
                }
            }

            // Check distance to all boxes (only check if character check passed)
            if position_valid {
                for box_pos in &box_positions {
                    let box_ground_pos = Vec3::new(box_pos.x, potential_pos.y, box_pos.z); // Same Y level for distance calc
                    if potential_pos.distance(box_ground_pos) < box_clearance {
                        position_valid = false;
                        break;
                    }
                }
            }

            if position_valid {
                valid_position = Some(potential_pos);
                break;
            }

            attempts += 1;
        }

        // Use the valid position, or fallback to a safe grid-based position if we couldn't find one
        let final_position = valid_position.unwrap_or_else(|| {
            // Try fallback positions that avoid both characters and boxes
            for fallback_attempt in 0..20 {
                let row = (i + fallback_attempt) / 5;
                let col = (i + fallback_attempt) % 5;
                let fallback_pos = Vec3::new(
                    col as f32 - 2.0, // Start further from center
                    0.05,
                    row as f32 - 2.0,
                );

                // Check if this fallback position is safe
                let mut safe = true;

                // Check against existing characters
                for existing_pos in &spawn_positions {
                    if fallback_pos.distance(*existing_pos) < min_distance {
                        safe = false;
                        break;
                    }
                }

                // Check against boxes
                if safe {
                    for box_pos in &box_positions {
                        let box_ground_pos = Vec3::new(box_pos.x, fallback_pos.y, box_pos.z);
                        if fallback_pos.distance(box_ground_pos) < box_clearance {
                            safe = false;
                            break;
                        }
                    }
                }

                if safe {
                    return fallback_pos;
                }
            }

            // Final emergency fallback - far from everything
            Vec3::new((i as f32) * 2.0 - 10.0, 0.05, -10.0)
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
            UnitCollision {
                radius: 0.5,
                allow_friendly_overlap: true,
            },
            Inventory::new(),    // Add inventory component for resource gathering
            Capacity::default(), // Add capacity component (28 slots, high stack limit)
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

    // Use the box positions we defined earlier (now characters have spawned safely around them)
    for (i, position) in box_positions.iter().enumerate() {
        info!(
            "📦 Spawning obstacle box {} at position ({:.2}, {:.2})",
            i + 1,
            position.x,
            position.z
        );

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
            near: -50.0, // Extended near plane (negative for orthographic)
            far: 50.0,   // Extended far plane
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainCamera,
    ));
}
