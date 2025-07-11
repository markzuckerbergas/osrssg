use crate::{components::*, resources::*};
use bevy::prelude::*;
use rand::Rng;

/// Resource node spawning system for the RuneScape-style gathering game
/// 
/// This system spawns resource nodes (trees, copper rocks, tin rocks) in the world
/// following these principles:
/// 
/// **Spawning Rules:**
/// - Grid-aligned positions (integer coordinates)
/// - Collision avoidance with existing entities (units, obstacles)
/// - Minimum distance between resources (2.0 units)
/// - Clearance from existing entities (1.5 units)
/// 
/// **Resource Types:**
/// - Trees: Green tall boxes (40% spawn rate) - 50 logs each
/// - Copper rocks: Brown/orange rocky shapes (30% spawn rate) - 30 ore each  
/// - Tin rocks: Silver rocky shapes (30% spawn rate) - 25 ore each
/// 
/// **Visual Design:**
/// - Trees: 0.4x1.5x0.4 size, positioned with base at ground level
/// - Copper rocks: 0.9x0.7x0.7 size, centered on ground
/// - Tin rocks: 0.8x0.6x0.8 size, centered on ground
/// - Bright, distinct colors for easy identification
/// 
/// **World Layout:**
/// - Spawns 8 resources per "chunk" (16x16 area)
/// - World bounds: -8 to +8 in both X and Z
/// - Uses existing grid helper for clean tile alignment

/// Configuration for resource spawning
pub struct ResourceSpawnConfig {
    /// World bounds for spawning resources
    pub world_min: Vec3,
    pub world_max: Vec3,
    /// Number of resources per chunk (roughly 16x16 area)
    pub resources_per_chunk: usize,
    /// Minimum distance between resources
    pub min_resource_distance: f32,
    /// Minimum distance from existing obstacles/entities
    pub clearance_distance: f32,
}

impl Default for ResourceSpawnConfig {
    fn default() -> Self {
        Self {
            world_min: Vec3::new(-8.0, 0.0, -8.0),
            world_max: Vec3::new(8.0, 0.0, 8.0),
            resources_per_chunk: 8, // 8 resources in a 16x16 area
            min_resource_distance: 2.0,
            clearance_distance: 1.5, // Don't spawn too close to obstacles or units
        }
    }
}

/// Spawns resource nodes in the world
pub fn spawn_resources(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    gathering_config: Res<GatheringConfig>,
    // Query existing entities to avoid collisions
    obstacles: Query<&Transform, (With<StaticObstacle>, Without<ResourceNode>)>,
    units: Query<&Transform, (With<Controllable>, Without<ResourceNode>)>,
) {
    let config = ResourceSpawnConfig::default();
    let mut rng = rand::thread_rng();

    info!("🌲⛏️ Spawning resource nodes in the world");

    // Create materials for different resource types
    let tree_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.8, 0.2), // Brighter green for trees
        metallic: 0.0,
        perceptual_roughness: 0.8,
        reflectance: 0.0,
        ..default()
    });

    let copper_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.6, 0.3), // Brighter copper/orange color
        metallic: 0.3,
        perceptual_roughness: 0.6,
        reflectance: 0.2,
        ..default()
    });

    let tin_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 0.9), // Brighter silver-blue for tin
        metallic: 0.4,
        perceptual_roughness: 0.5,
        reflectance: 0.3,
        ..default()
    });

    // Collect existing entity positions for collision avoidance
    let mut existing_positions = Vec::new();
    
    // Add obstacle positions
    for transform in obstacles.iter() {
        existing_positions.push(transform.translation);
    }
    
    // Add unit positions
    for transform in units.iter() {
        existing_positions.push(transform.translation);
    }

    let mut spawned_resources: Vec<Vec3> = Vec::new();

    // Define resource types to spawn
    let resource_types = [
        (ResourceKind::Wood, 0.4),   // 40% trees
        (ResourceKind::Copper, 0.3), // 30% copper
        (ResourceKind::Tin, 0.3),    // 30% tin
    ];

    for _ in 0..config.resources_per_chunk {
        let mut attempts = 0;
        let max_attempts = 100;
        let mut spawn_position = None;

        // Try to find a valid spawn position
        while attempts < max_attempts {
            // Generate grid-aligned position
            let x = rng.gen_range(config.world_min.x as i32..=config.world_max.x as i32) as f32;
            let z = rng.gen_range(config.world_min.z as i32..=config.world_max.z as i32) as f32;
            let potential_pos = Vec3::new(x, 0.1, z); // Slightly above ground

            let mut position_valid = true;

            // Check distance to existing entities (obstacles, units)
            for existing_pos in &existing_positions {
                let ground_distance = Vec3::new(potential_pos.x, 0.0, potential_pos.z)
                    .distance(Vec3::new(existing_pos.x, 0.0, existing_pos.z));
                if ground_distance < config.clearance_distance {
                    position_valid = false;
                    break;
                }
            }

            // Check distance to other spawned resources
            if position_valid {
                for resource_pos in &spawned_resources {
                    let ground_distance = Vec3::new(potential_pos.x, 0.0, potential_pos.z)
                        .distance(Vec3::new(resource_pos.x, 0.0, resource_pos.z));
                    if ground_distance < config.min_resource_distance {
                        position_valid = false;
                        break;
                    }
                }
            }

            if position_valid {
                spawn_position = Some(potential_pos);
                break;
            }

            attempts += 1;
        }

        if let Some(position) = spawn_position {
            spawned_resources.push(position);

            // Choose resource type based on weighted probability
            let random_val: f32 = rng.gen();
            let mut cumulative_weight = 0.0;
            let mut chosen_resource = ResourceKind::Wood; // Default fallback

            for (resource_kind, weight) in &resource_types {
                cumulative_weight += weight;
                if random_val <= cumulative_weight {
                    chosen_resource = *resource_kind;
                    break;
                }
            }

            // Create the resource node
            let (mesh, material, position_offset, remaining, collision_size) = match chosen_resource {
                ResourceKind::Wood => (
                    meshes.add(Cuboid::new(0.4, 1.5, 0.4)), // Taller, thicker tree
                    tree_material.clone(),
                    Vec3::new(0.0, 0.75, 0.0), // Raise tree so base is at ground level
                    50, // 50 logs per tree
                    Vec3::new(1.2, 1.5, 1.2), // Larger collision area for trees to make them easier to avoid
                ),
                ResourceKind::Copper => (
                    meshes.add(Cuboid::new(0.9, 0.7, 0.7)), // Rocky shape for ore
                    copper_material.clone(),
                    Vec3::new(0.0, 0.35, 0.0), // Center on ground
                    30, // 30 copper ore per node
                    Vec3::new(1.1, 0.7, 1.1), // Slightly larger collision than visual
                ),
                ResourceKind::Tin => (
                    meshes.add(Cuboid::new(0.8, 0.6, 0.8)), // Slightly different shape
                    tin_material.clone(),
                    Vec3::new(0.0, 0.3, 0.0), // Center on ground
                    25, // 25 tin ore per node
                    Vec3::new(1.0, 0.6, 1.0), // Slightly larger collision than visual
                ),
            };

            let transform = Transform {
                translation: position + position_offset,
                scale: Vec3::splat(1.0),
                ..default()
            };

            // Calculate gather rate and radius based on resource type
            let gather_rate = chosen_resource.base_gather_rate();
            let gather_radius = gathering_config.default_gather_radius;

            info!(
                "🌱 Spawning {} at position ({:.1}, {:.1}) with {} remaining",
                chosen_resource.display_name(),
                position.x,
                position.z,
                remaining
            );

            // Spawn the resource node entity
            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                transform,
                ResourceNode::new(chosen_resource, remaining, gather_rate, gather_radius),
                StaticObstacle, // Make resource nodes collidable obstacles
                CollisionSize::new(collision_size), // Add collision size for proper collision detection
                Name::new(format!("{} Node", chosen_resource.display_name())),
            ));
        } else {
            warn!("❌ Failed to find valid position for resource after {} attempts", max_attempts);
        }
    }

    info!("✅ Successfully spawned {} resource nodes", spawned_resources.len());
}

/// Helper function to get a safe spawn position avoiding existing entities
pub fn find_safe_resource_position(
    rng: &mut impl Rng,
    world_min: Vec3,
    world_max: Vec3,
    existing_positions: &[Vec3],
    min_distance: f32,
    max_attempts: usize,
) -> Option<Vec3> {
    for _ in 0..max_attempts {
        let x = rng.gen_range(world_min.x as i32..=world_max.x as i32) as f32;
        let z = rng.gen_range(world_min.z as i32..=world_max.z as i32) as f32;
        let potential_pos = Vec3::new(x, 0.1, z);

        let mut position_valid = true;
        for existing_pos in existing_positions {
            let ground_distance = Vec3::new(potential_pos.x, 0.0, potential_pos.z)
                .distance(Vec3::new(existing_pos.x, 0.0, existing_pos.z));
            if ground_distance < min_distance {
                position_valid = false;
                break;
            }
        }

        if position_valid {
            return Some(potential_pos);
        }
    }

    None
}
