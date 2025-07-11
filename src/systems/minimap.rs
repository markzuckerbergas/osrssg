use crate::components::*;
use crate::resources::*;
use bevy::prelude::*;

/// Setup the minimap UI using proper Bevy UI components
pub fn setup_minimap(mut commands: Commands, minimap_settings: Res<MinimapSettings>) {
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

            // Map area container (this is where we'll add the clickable interaction)
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(90.0),
                    position_type: PositionType::Relative,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.2, 0.3, 0.2, 0.5)), // Dark green map background
                Interaction::default(), // Make it interactive for clicking
                MinimapUI,              // Also tag this as minimap for easy querying
            ));
        });
}

/// Update minimap player dots and camera viewport indicator
pub fn update_minimap(
    mut commands: Commands,
    _minimap_settings: Res<MinimapSettings>,
    camera_settings: Res<CameraSettings>,
    players_query: Query<&Transform, (With<Controllable>, Without<MinimapPlayerDot>)>,
    main_camera_query: Query<&Transform, (With<MainCamera>, Without<Controllable>)>,
    minimap_dots_query: Query<Entity, With<MinimapPlayerDot>>,
    minimap_viewport_query: Query<Entity, With<MinimapCameraViewport>>,
    minimap_containers: Query<Entity, (With<MinimapUI>, With<Node>)>,
) {
    // Clear existing dots and viewport indicator
    for entity in minimap_dots_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in minimap_viewport_query.iter() {
        commands.entity(entity).despawn();
    }

    // Find the map area container (we'll use the second MinimapUI entity which is the map area)
    let minimap_entities: Vec<Entity> = minimap_containers.iter().collect();
    if minimap_entities.len() >= 2 {
        let map_container = minimap_entities[1]; // The map area container

        // Spawn new dots for each player
        for player_transform in players_query.iter() {
            // Convert world position to minimap position with isometric transformation
            let world_pos = Vec2::new(
                player_transform.translation.x,
                player_transform.translation.z,
            );

            // Apply isometric transformation to match the camera view
            let angle = -std::f32::consts::PI / 4.0 + std::f32::consts::PI / 2.0; // -45° + 90° = 45°
            let cos_a = angle.cos();
            let sin_a = angle.sin();

            let isometric_pos = Vec2::new(
                world_pos.x * cos_a - world_pos.y * sin_a,
                world_pos.x * sin_a + world_pos.y * cos_a,
            );

            // Transform the bounds using the same rotation
            let bounds_min_world = camera_settings.bounds_min.xz();
            let bounds_max_world = camera_settings.bounds_max.xz();

            let corners = [
                bounds_min_world,
                Vec2::new(bounds_max_world.x, bounds_min_world.y),
                bounds_max_world,
                Vec2::new(bounds_min_world.x, bounds_max_world.y),
            ];

            let mut iso_bounds_min = Vec2::new(f32::INFINITY, f32::INFINITY);
            let mut iso_bounds_max = Vec2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);

            for corner in corners {
                let iso_corner = Vec2::new(
                    corner.x * cos_a - corner.y * sin_a,
                    corner.x * sin_a + corner.y * cos_a,
                );
                iso_bounds_min = iso_bounds_min.min(iso_corner);
                iso_bounds_max = iso_bounds_max.max(iso_corner);
            }

            let iso_world_bounds = iso_bounds_max - iso_bounds_min;
            let normalized_pos = (isometric_pos - iso_bounds_min) / iso_world_bounds;

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

        // Add camera viewport indicator
        if let Ok(camera_transform) = main_camera_query.single() {
            let camera_world_pos = Vec2::new(
                camera_transform.translation.x,
                camera_transform.translation.z,
            );

            // Apply isometric transformation to match the visual perspective
            let angle = -std::f32::consts::PI / 4.0 + std::f32::consts::PI / 2.0; // -45° + 90° = 45°
            let cos_a = angle.cos();
            let sin_a = angle.sin();

            let isometric_pos = Vec2::new(
                camera_world_pos.x * cos_a - camera_world_pos.y * sin_a,
                camera_world_pos.x * sin_a + camera_world_pos.y * cos_a,
            );

            // Transform the bounds using the same rotation
            let bounds_min_world = camera_settings.bounds_min.xz();
            let bounds_max_world = camera_settings.bounds_max.xz();

            let corners = [
                bounds_min_world,
                Vec2::new(bounds_max_world.x, bounds_min_world.y),
                bounds_max_world,
                Vec2::new(bounds_min_world.x, bounds_max_world.y),
            ];

            let mut iso_bounds_min = Vec2::new(f32::INFINITY, f32::INFINITY);
            let mut iso_bounds_max = Vec2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);

            for corner in corners {
                let iso_corner = Vec2::new(
                    corner.x * cos_a - corner.y * sin_a,
                    corner.x * sin_a + corner.y * cos_a,
                );
                iso_bounds_min = iso_bounds_min.min(iso_corner);
                iso_bounds_max = iso_bounds_max.max(iso_corner);
            }

            let iso_world_bounds = iso_bounds_max - iso_bounds_min;
            let normalized_camera_pos = (isometric_pos - iso_bounds_min) / iso_world_bounds;
            let camera_minimap_percent = normalized_camera_pos * 100.0;

            // Camera viewport size on minimap (represents standard zoom level)
            let viewport_size = 25.0; // Percentage of minimap that represents the camera view

            // Center the viewport around the camera position, but allow it to touch edges
            let viewport_left =
                (camera_minimap_percent.x - viewport_size / 2.0).clamp(0.0, 100.0 - viewport_size);
            let viewport_top =
                (camera_minimap_percent.y - viewport_size / 2.0).clamp(0.0, 100.0 - viewport_size);

            commands.entity(map_container).with_children(|parent| {
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(viewport_left),
                        top: Val::Percent(viewport_top),
                        width: Val::Percent(viewport_size),
                        height: Val::Percent(viewport_size),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::NONE), // Transparent background
                    BorderColor(Color::WHITE),    // White border for visibility
                    GlobalZIndex(102),            // Ensure viewport appears above dots
                    Interaction::default(),       // Make viewport draggable
                    MinimapCameraViewport,
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

/// Handle minimap clicks to move camera
pub fn handle_minimap_click(
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    camera_settings: Res<CameraSettings>,
    minimap_settings: Res<MinimapSettings>,
    minimap_query: Query<&Interaction, (Changed<Interaction>, With<MinimapUI>)>,
    windows: Query<&Window>,
) {
    // Only handle clicks on the minimap area itself
    for interaction in minimap_query.iter() {
        if *interaction == Interaction::Pressed {
            if let Ok(window) = windows.single() {
                if let Some(cursor_position) = window.cursor_position() {
                    // SIMPLIFIED APPROACH: Direct mapping without complex transformations

                    // Calculate minimap area bounds
                    let minimap_size = minimap_settings.size.x; // 200.0
                    let window_size = Vec2::new(window.width(), window.height());

                    // Account for the UI structure: minimap container with padding and title
                    let container_padding = 4.0; // From Node padding in setup_minimap
                    let title_height = 16.0; // Approximate height for title text
                    let border_width = 2.0; // From border in setup_minimap

                    // Calculate actual map area position and size
                    let total_padding = container_padding + border_width;
                    let map_area_size = minimap_size - (total_padding * 2.0);
                    let map_area_height = map_area_size * 0.9; // 90% height for map area

                    // Minimap screen position (bottom-right with padding)
                    let minimap_screen_pos = Vec2::new(
                        window_size.x - minimap_size - 10.0,
                        window_size.y - minimap_size - 10.0,
                    );

                    // Map area position within the minimap
                    let map_area_pos = Vec2::new(
                        minimap_screen_pos.x + total_padding,
                        minimap_screen_pos.y + total_padding + title_height,
                    );

                    // Check if click is within the actual map area
                    let relative_to_map = cursor_position - map_area_pos;
                    if relative_to_map.x >= 0.0
                        && relative_to_map.x <= map_area_size
                        && relative_to_map.y >= 0.0
                        && relative_to_map.y <= map_area_height
                    {
                        // Convert click to normalized coordinates (0-1) within map area
                        let click_percent = Vec2::new(
                            relative_to_map.x / map_area_size,
                            relative_to_map.y / map_area_height,
                        );

                        // Convert from minimap isometric space back to world coordinates
                        // Use the same transformation as display but in reverse
                        let display_angle =
                            -std::f32::consts::PI / 4.0 + std::f32::consts::PI / 2.0; // 45°
                        let cos_a = display_angle.cos();
                        let sin_a = display_angle.sin();

                        // Calculate the isometric bounds the same way as in display
                        let bounds_min_world = camera_settings.bounds_min.xz();
                        let bounds_max_world = camera_settings.bounds_max.xz();

                        let corners = [
                            bounds_min_world,
                            Vec2::new(bounds_max_world.x, bounds_min_world.y),
                            bounds_max_world,
                            Vec2::new(bounds_min_world.x, bounds_max_world.y),
                        ];

                        let mut iso_bounds_min = Vec2::new(f32::INFINITY, f32::INFINITY);
                        let mut iso_bounds_max = Vec2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);

                        for corner in corners {
                            let iso_corner = Vec2::new(
                                corner.x * cos_a - corner.y * sin_a,
                                corner.x * sin_a + corner.y * cos_a,
                            );
                            iso_bounds_min = iso_bounds_min.min(iso_corner);
                            iso_bounds_max = iso_bounds_max.max(iso_corner);
                        }

                        // Convert click percentage to isometric coordinate
                        let iso_world_bounds = iso_bounds_max - iso_bounds_min;
                        let click_iso_pos = iso_bounds_min + (click_percent * iso_world_bounds);

                        // Apply the INVERSE transformation to get back to world coordinates
                        // For inverse rotation: R^(-1) = R^T, so negate the sine component
                        let inv_cos_a = cos_a; // cos(-θ) = cos(θ)
                        let inv_sin_a = -sin_a; // sin(-θ) = -sin(θ)

                        let target_world_pos = Vec2::new(
                            click_iso_pos.x * inv_cos_a - click_iso_pos.y * inv_sin_a,
                            click_iso_pos.x * inv_sin_a + click_iso_pos.y * inv_cos_a,
                        );

                        // Move camera to the clicked position
                        if let Ok(mut camera_transform) = camera_query.single_mut() {
                            let new_camera_pos = Vec3::new(
                                target_world_pos.x,
                                camera_transform.translation.y, // Keep the same height
                                target_world_pos.y,
                            );

                            // Clamp to camera bounds - this ensures the camera stays within world boundaries
                            // The viewport box will be allowed to touch minimap edges when camera is at world boundaries
                            let clamped_pos = Vec3::new(
                                new_camera_pos.x.clamp(
                                    camera_settings.bounds_min.x,
                                    camera_settings.bounds_max.x,
                                ),
                                new_camera_pos.y,
                                new_camera_pos.z.clamp(
                                    camera_settings.bounds_min.z,
                                    camera_settings.bounds_max.z,
                                ),
                            );

                            camera_transform.translation = clamped_pos;
                            info!(
                                "🗺️ Camera moved to: {:?} from minimap click at {:?}",
                                clamped_pos, click_percent
                            );
                        }
                    }
                }
            }
        }
    }
}

/// Handle minimap viewport dragging to move camera
pub fn handle_minimap_drag(
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    camera_settings: Res<CameraSettings>,
    minimap_settings: Res<MinimapSettings>,
    mut drag_state: ResMut<MinimapDragState>,
    buttons: Res<ButtonInput<MouseButton>>,
    drag_selection_query: Query<&DragSelection>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    // Don't start minimap drag if there's an active drag selection
    if !drag_selection_query.is_empty() && !drag_state.is_dragging {
        return;
    }

    // Check if we're starting to drag anywhere on the minimap
    let mut should_start_drag = false;
    if buttons.just_pressed(MouseButton::Left) {
        // Check if clicking on minimap area
        let minimap_rect = Rect::from_corners(
            Vec2::new(window.width() - 200.0, window.height() - 200.0),
            Vec2::new(window.width(), window.height()),
        );

        if minimap_rect.contains(cursor_position) {
            should_start_drag = true;

            // If just clicking (not dragging yet), move camera to that position
            let container_padding = 4.0;
            let border_width = 2.0;
            let total_padding = container_padding + border_width;
            let map_area_size = minimap_settings.size.x - (total_padding * 2.0);

            // Convert click position to normalized minimap coordinates
            let minimap_top_left = Vec2::new(
                window.width() - 200.0 + total_padding,
                window.height() - 200.0 + total_padding,
            );
            let relative_pos = cursor_position - minimap_top_left;
            let normalized_pos = Vec2::new(
                relative_pos.x / map_area_size,
                relative_pos.y / map_area_size,
            );

            // Clamp to valid range
            let clamped_pos = normalized_pos.clamp(Vec2::ZERO, Vec2::ONE);

            // Convert to world coordinates using isometric transformation
            let display_angle = -std::f32::consts::PI / 4.0 + std::f32::consts::PI / 2.0;
            let cos_a = display_angle.cos();
            let sin_a = display_angle.sin();

            let bounds_min_world = camera_settings.bounds_min.xz();
            let bounds_max_world = camera_settings.bounds_max.xz();

            let corners = [
                bounds_min_world,
                Vec2::new(bounds_max_world.x, bounds_min_world.y),
                bounds_max_world,
                Vec2::new(bounds_min_world.x, bounds_max_world.y),
            ];

            let mut iso_bounds_min = Vec2::new(f32::INFINITY, f32::INFINITY);
            let mut iso_bounds_max = Vec2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);

            for corner in corners {
                let iso_corner = Vec2::new(
                    corner.x * cos_a - corner.y * sin_a,
                    corner.x * sin_a + corner.y * cos_a,
                );
                iso_bounds_min = iso_bounds_min.min(iso_corner);
                iso_bounds_max = iso_bounds_max.max(iso_corner);
            }

            let iso_world_bounds = iso_bounds_max - iso_bounds_min;
            let iso_pos = iso_bounds_min + clamped_pos * iso_world_bounds;

            // Apply inverse transformation to get world position
            let inv_cos_a = cos_a;
            let inv_sin_a = -sin_a;

            let world_pos = Vec2::new(
                iso_pos.x * inv_cos_a - iso_pos.y * inv_sin_a,
                iso_pos.x * inv_sin_a + iso_pos.y * inv_cos_a,
            );

            // Move camera to the clicked position
            if let Ok(mut camera_transform) = camera_query.single_mut() {
                let new_pos = Vec3::new(
                    world_pos
                        .x
                        .clamp(camera_settings.bounds_min.x, camera_settings.bounds_max.x),
                    camera_transform.translation.y,
                    world_pos
                        .y
                        .clamp(camera_settings.bounds_min.z, camera_settings.bounds_max.z),
                );
                camera_transform.translation = new_pos;

                info!(
                    "🗺️ Camera moved to: {} from minimap click at {}",
                    new_pos, clamped_pos
                );
            }
        }
    }

    if should_start_drag {
        drag_state.is_dragging = true;
        drag_state.last_mouse_pos = Some(cursor_position);
        info!("🗺️ Started dragging minimap");
        return;
    }

    // Handle ongoing drag (only if we started by clicking the viewport)
    if drag_state.is_dragging && buttons.pressed(MouseButton::Left) {
        if let Some(last_pos) = drag_state.last_mouse_pos {
            let delta = cursor_position - last_pos;

            // Convert mouse delta to world coordinate delta
            let minimap_size = minimap_settings.size.x;
            let container_padding = 4.0;
            let border_width = 2.0;
            let total_padding = container_padding + border_width;
            let map_area_size = minimap_size - (total_padding * 2.0);

            // Convert pixel delta to normalized minimap delta
            let normalized_delta = Vec2::new(delta.x / map_area_size, delta.y / map_area_size);

            // Apply isometric transformation to convert to world delta
            let display_angle = -std::f32::consts::PI / 4.0 + std::f32::consts::PI / 2.0;
            let cos_a = display_angle.cos();
            let sin_a = display_angle.sin();

            // Calculate world bounds in isometric space
            let bounds_min_world = camera_settings.bounds_min.xz();
            let bounds_max_world = camera_settings.bounds_max.xz();

            let corners = [
                bounds_min_world,
                Vec2::new(bounds_max_world.x, bounds_min_world.y),
                bounds_max_world,
                Vec2::new(bounds_min_world.x, bounds_max_world.y),
            ];

            let mut iso_bounds_min = Vec2::new(f32::INFINITY, f32::INFINITY);
            let mut iso_bounds_max = Vec2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);

            for corner in corners {
                let iso_corner = Vec2::new(
                    corner.x * cos_a - corner.y * sin_a,
                    corner.x * sin_a + corner.y * cos_a,
                );
                iso_bounds_min = iso_bounds_min.min(iso_corner);
                iso_bounds_max = iso_bounds_max.max(iso_corner);
            }

            let iso_world_bounds = iso_bounds_max - iso_bounds_min;
            let iso_delta = normalized_delta * iso_world_bounds;

            // Apply inverse transformation to get world delta
            let inv_cos_a = cos_a;
            let inv_sin_a = -sin_a;

            let world_delta = Vec2::new(
                iso_delta.x * inv_cos_a - iso_delta.y * inv_sin_a,
                iso_delta.x * inv_sin_a + iso_delta.y * inv_cos_a,
            );

            // Apply delta to camera position
            if let Ok(mut camera_transform) = camera_query.single_mut() {
                let new_pos = Vec3::new(
                    camera_transform.translation.x + world_delta.x,
                    camera_transform.translation.y,
                    camera_transform.translation.z + world_delta.y,
                );

                // Clamp to bounds
                let clamped_pos = Vec3::new(
                    new_pos
                        .x
                        .clamp(camera_settings.bounds_min.x, camera_settings.bounds_max.x),
                    new_pos.y,
                    new_pos
                        .z
                        .clamp(camera_settings.bounds_min.z, camera_settings.bounds_max.z),
                );

                camera_transform.translation = clamped_pos;
            }

            drag_state.last_mouse_pos = Some(cursor_position);
        }
    }

    // Stop dragging when mouse button is released
    if drag_state.is_dragging && buttons.just_released(MouseButton::Left) {
        drag_state.is_dragging = false;
        drag_state.last_mouse_pos = None;
        info!("🗺️ Stopped dragging minimap viewport");
    }
}
