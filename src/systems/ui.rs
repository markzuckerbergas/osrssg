use crate::components::*;
use bevy::prelude::*;
use rand::Rng;

/// Component to mark the spawn button
#[derive(Component)]
pub struct SpawnButton;

/// Component to mark the UI container
#[derive(Component)]
pub struct GameUI;

/// Sets up the game UI with spawn button
pub fn setup_game_ui(mut commands: Commands) {
    info!("🎨 Setting up game UI");

    // Create UI container centered above the minimap
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(35.0), // Center button over minimap: 10px + (200px-150px)/2 = 35px
                bottom: Val::Px(220.0), // Position much higher above the minimap
                width: Val::Auto,
                height: Val::Auto,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::NONE), // Transparent background
            GlobalZIndex(1000),           // Much higher than minimap's GlobalZIndex(102)
            GameUI,
            Name::new("GameUI"),
        ))
        .with_children(|parent| {
            // Spawn Character Button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(40.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor(Color::srgb(0.8, 0.8, 0.8)),
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    GlobalZIndex(1001), // Even higher than container
                    SpawnButton,
                    Name::new("SpawnButton"),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Spawn Character"),
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

/// Handles spawn button interactions
pub fn handle_spawn_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<SpawnButton>),
    >,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    existing_units: Query<&Transform, With<Controllable>>,
    static_obstacles: Query<&Transform, With<StaticObstacle>>,
) {
    for (interaction, mut background_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                info!("🆕 Spawn button pressed!");

                // Button pressed visual feedback
                *background_color = Color::srgb(0.1, 0.1, 0.1).into();
                *border_color = Color::WHITE.into();

                // Spawn new character
                spawn_random_character(
                    &mut commands,
                    &asset_server,
                    &existing_units,
                    &static_obstacles,
                );
            }
            Interaction::Hovered => {
                // Button hover visual feedback
                *background_color = Color::srgb(0.3, 0.3, 0.3).into();
                *border_color = Color::srgb(0.9, 0.9, 0.9).into();
            }
            Interaction::None => {
                // Button normal visual feedback
                *background_color = Color::srgb(0.2, 0.2, 0.2).into();
                *border_color = Color::srgb(0.8, 0.8, 0.8).into();
            }
        }
    }
}

/// Spawns a new character at a random valid location
fn spawn_random_character(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    existing_units: &Query<&Transform, With<Controllable>>,
    static_obstacles: &Query<&Transform, With<StaticObstacle>>,
) {
    let mut rng = rand::thread_rng();
    let min_distance = 1.0; // Minimum distance between characters
    let box_clearance = 1.0; // Minimum distance from boxes
    let max_attempts = 50;

    // Collect existing positions
    let mut existing_positions = Vec::new();
    for transform in existing_units.iter() {
        existing_positions.push(transform.translation);
    }

    // Collect box positions
    let mut box_positions = Vec::new();
    for transform in static_obstacles.iter() {
        box_positions.push(transform.translation);
    }

    let mut attempts = 0;
    let mut spawn_position = None;

    // Try to find a valid spawn position
    while attempts < max_attempts {
        let x = rng.gen_range(-8..8) as f32; // Integer grid coordinates
        let z = rng.gen_range(-8..8) as f32;
        let potential_pos = Vec3::new(x, 0.05, z); // Grid-aligned position

        let mut position_valid = true;

        // Check distance to all existing characters
        for existing_pos in &existing_positions {
            if potential_pos.distance(*existing_pos) < min_distance {
                position_valid = false;
                break;
            }
        }

        // Check distance to all boxes
        if position_valid {
            for box_pos in &box_positions {
                if potential_pos.distance(*box_pos) < box_clearance {
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

    if let Some(pos) = spawn_position {
        let player_scene = asset_server.load("player.glb#Scene0");

        let character_transform = Transform {
            translation: pos,
            scale: Vec3::splat(0.03),
            ..default()
        };

        commands.spawn((
            SceneRoot(player_scene),
            character_transform,
            GlobalTransform::default(),
            Visibility::default(),
            // Game components
            Controllable,
            CollisionRadius { radius: 0.3 },
            StuckTimer::default(),
            Name::new(format!("Player_{}", rng.gen::<u32>())),
        ));

        info!(
            "👤 Spawned new character at position ({:.0}, {:.0})",
            pos.x, pos.z
        );
    } else {
        info!(
            "❌ Could not find valid spawn position after {} attempts",
            max_attempts
        );
    }
}
