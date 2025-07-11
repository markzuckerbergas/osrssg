use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;

/// Setup the minimap UI using proper Bevy UI components
pub fn setup_minimap(
    mut commands: Commands,
    minimap_settings: Res<MinimapSettings>,
) {
    info!("🗺️ Setting up minimap UI");
    
    // Create the minimap container using proper Bevy UI
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Percent((1.0 - minimap_settings.position.x) * 100.0),
                bottom: Val::Percent((1.0 - minimap_settings.position.y) * 100.0),
                width: Val::Px(minimap_settings.size.x),
                height: Val::Px(minimap_settings.size.y),
                border: UiRect::all(Val::Px(2.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                padding: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)), // Semi-transparent dark background
            BorderColor(Color::WHITE),
            GlobalZIndex(100), // Ensure minimap appears above other UI
            MinimapUI,
        ))
        .with_children(|parent| {
            // Title text
            parent.spawn((
                Text::new("Map"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            
            // Map area container
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(90.0),
                    position_type: PositionType::Relative,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.2, 0.3, 0.2, 0.5)), // Dark green map background
                MinimapUI, // Also tag this as minimap for easy querying
            ));
        });
}

/// Update minimap player dots
pub fn update_minimap(
    mut commands: Commands,
    _minimap_settings: Res<MinimapSettings>,
    camera_settings: Res<CameraSettings>,
    players_query: Query<&Transform, (With<Controllable>, Without<MinimapPlayerDot>)>,
    minimap_dots_query: Query<Entity, With<MinimapPlayerDot>>,
    minimap_containers: Query<Entity, (With<MinimapUI>, With<Node>)>,
) {
    // Clear existing dots
    for entity in minimap_dots_query.iter() {
        commands.entity(entity).despawn();
    }

    // Find the map area container (we'll use the second MinimapUI entity which is the map area)
    let minimap_entities: Vec<Entity> = minimap_containers.iter().collect();
    if minimap_entities.len() >= 2 {
        let map_container = minimap_entities[1]; // The map area container

        // Spawn new dots for each player
        for player_transform in players_query.iter() {
            // Convert world position to minimap position
            let world_pos = Vec2::new(player_transform.translation.x, player_transform.translation.z);
            
            // Normalize world position to 0-1 range based on camera bounds
            let world_bounds = camera_settings.bounds_max.xz() - camera_settings.bounds_min.xz();
            let normalized_pos = (world_pos - camera_settings.bounds_min.xz()) / world_bounds;
            
            // Convert to minimap percentage (0-100%)
            let minimap_percent = normalized_pos * 100.0;
            
            // Clamp to minimap bounds
            let clamped_percent = minimap_percent.clamp(Vec2::ZERO, Vec2::new(95.0, 95.0));

            commands.entity(map_container).with_children(|parent| {
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(clamped_percent.x),
                        top: Val::Percent(clamped_percent.y),
                        width: Val::Px(6.0),
                        height: Val::Px(6.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.0, 1.0, 0.0)), // Green dots for players
                    GlobalZIndex(101), // Ensure dots appear above the map background
                    MinimapPlayerDot,
                ));
            });
        }
    }
}

/// System to handle minimap visibility toggle
pub fn toggle_minimap_visibility(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut minimap_query: Query<&mut Visibility, With<MinimapUI>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        for mut visibility in minimap_query.iter_mut() {
            *visibility = match *visibility {
                Visibility::Visible => Visibility::Hidden,
                _ => Visibility::Visible,
            };
        }
        info!("🗺️ Toggled minimap visibility (Press M to toggle)");
    }
}
