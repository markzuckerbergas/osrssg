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
                MinimapUI, // Also tag this as minimap for easy querying
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
            let world_pos = Vec2::new(player_transform.translation.x, player_transform.translation.z);
            
            // Apply isometric transformation to match the camera view
            let angle = -std::f32::consts::PI / 4.0 + std::f32::consts::PI / 2.0; // -45° + 90° = 45°
            let cos_a = angle.cos();
            let sin_a = angle.sin();
            
            let isometric_pos = Vec2::new(
                world_pos.x * cos_a - world_pos.y * sin_a,
                world_pos.x * sin_a + world_pos.y * cos_a
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
                    corner.x * sin_a + corner.y * cos_a
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
            let camera_world_pos = Vec2::new(camera_transform.translation.x, camera_transform.translation.z);
            
            // Apply isometric transformation to match the visual perspective
            let angle = -std::f32::consts::PI / 4.0 + std::f32::consts::PI / 2.0; // -45° + 90° = 45°
            let cos_a = angle.cos();
            let sin_a = angle.sin();
            
            let isometric_pos = Vec2::new(
                camera_world_pos.x * cos_a - camera_world_pos.y * sin_a,
                camera_world_pos.x * sin_a + camera_world_pos.y * cos_a
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
                    corner.x * sin_a + corner.y * cos_a
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
            let viewport_left = (camera_minimap_percent.x - viewport_size/2.0).clamp(0.0, 100.0 - viewport_size);
            let viewport_top = (camera_minimap_percent.y - viewport_size/2.0).clamp(0.0, 100.0 - viewport_size);
            
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
                    BorderColor(Color::WHITE), // White border for visibility
                    GlobalZIndex(102), // Ensure viewport appears above dots
                    Interaction::default(), // Make it interactive for dragging
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
    minimap_query: Query<&Interaction, (Changed<Interaction>, With<MinimapUI>, Without<MinimapCameraViewport>)>,
    viewport_query: Query<&Interaction, With<MinimapCameraViewport>>,
    windows: Query<&Window>,
) {
    // Don't handle clicks if we're clicking on the viewport box
    for viewport_interaction in viewport_query.iter() {
        if matches!(viewport_interaction, Interaction::Pressed | Interaction::Hovered) {
            return;
        }
    }

    // Only handle clicks on the minimap area itself (not the container and not the viewport)
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
                    if relative_to_map.x >= 0.0 && relative_to_map.x <= map_area_size && 
                       relative_to_map.y >= 0.0 && relative_to_map.y <= map_area_height {
                        
                        // Convert click to normalized coordinates (0-1) within map area
                        let click_percent = Vec2::new(
                            relative_to_map.x / map_area_size,
                            relative_to_map.y / map_area_height
                        );
                        
                        // Convert from minimap isometric space back to world coordinates
                        // Use the same transformation as display but in reverse
                        let display_angle = -std::f32::consts::PI / 4.0 + std::f32::consts::PI / 2.0; // 45°
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
                                corner.x * sin_a + corner.y * cos_a
                            );
                            iso_bounds_min = iso_bounds_min.min(iso_corner);
                            iso_bounds_max = iso_bounds_max.max(iso_corner);
                        }
                        
                        // Convert click percentage to isometric coordinate
                        let iso_world_bounds = iso_bounds_max - iso_bounds_min;
                        let click_iso_pos = iso_bounds_min + (click_percent * iso_world_bounds);
                        
                        // Apply the INVERSE transformation to get back to world coordinates
                        // For inverse rotation: R^(-1) = R^T, so negate the sine component
                        let inv_cos_a = cos_a;  // cos(-θ) = cos(θ)
                        let inv_sin_a = -sin_a; // sin(-θ) = -sin(θ)
                        
                        let target_world_pos = Vec2::new(
                            click_iso_pos.x * inv_cos_a - click_iso_pos.y * inv_sin_a,
                            click_iso_pos.x * inv_sin_a + click_iso_pos.y * inv_cos_a
                        );
                        
                        // Move camera to the clicked position
                        if let Ok(mut camera_transform) = camera_query.single_mut() {
                            let new_camera_pos = Vec3::new(
                                target_world_pos.x,
                                camera_transform.translation.y, // Keep the same height
                                target_world_pos.y
                            );
                            
                            // Clamp to camera bounds - this ensures the camera stays within world boundaries
                            // The viewport box will be allowed to touch minimap edges when camera is at world boundaries
                            let clamped_pos = Vec3::new(
                                new_camera_pos.x.clamp(camera_settings.bounds_min.x, camera_settings.bounds_max.x),
                                new_camera_pos.y,
                                new_camera_pos.z.clamp(camera_settings.bounds_min.z, camera_settings.bounds_max.z),
                            );
                            
                            camera_transform.translation = clamped_pos;
                            info!("🗺️ Camera moved to: {:?} from minimap click at {:?}", clamped_pos, click_percent);
                        }
                    }
                }
            }
        }
    }
}

/// Handle starting viewport drag
pub fn handle_minimap_viewport_drag_start(
    mut commands: Commands,
    camera_query: Query<&Transform, With<MainCamera>>,
    viewport_query: Query<(Entity, &Interaction), (Changed<Interaction>, With<MinimapCameraViewport>)>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.single() else { return; };
    let Some(cursor_position) = window.cursor_position() else { return; };

    // Check if we should start dragging
    for (viewport_entity, interaction) in viewport_query.iter() {
        if *interaction == Interaction::Pressed && mouse_input.just_pressed(MouseButton::Left) {
            // Start dragging
            if let Ok(camera_transform) = camera_query.single() {
                info!("🖱️ Starting viewport drag at cursor: {:?}", cursor_position);
                commands.entity(viewport_entity).insert(MinimapViewportDragging {
                    start_cursor_pos: cursor_position,
                    start_camera_pos: camera_transform.translation,
                });
            }
        }
    }
}

/// Handle ongoing viewport drag
pub fn handle_minimap_viewport_drag(
    mut commands: Commands,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    camera_settings: Res<CameraSettings>,
    minimap_settings: Res<MinimapSettings>,
    dragging_query: Query<(Entity, &MinimapViewportDragging)>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.single() else { return; };
    let Some(cursor_position) = window.cursor_position() else { return; };

    // Handle ongoing drag - check if we have any entities being dragged AND mouse is still pressed
    if mouse_input.pressed(MouseButton::Left) {
        let dragging_count = dragging_query.iter().count();
        if dragging_count > 0 {
            info!("🖱️ Found {} dragging entities, mouse pressed: {}", dragging_count, mouse_input.pressed(MouseButton::Left));
        }
        
        for (_viewport_entity, dragging) in dragging_query.iter() {
            info!("🖱️ Dragging viewport: cursor={:?}, delta={:?}", cursor_position, cursor_position - dragging.start_cursor_pos);
            let cursor_delta = cursor_position - dragging.start_cursor_pos;
            
            // Skip if no significant movement
            if cursor_delta.length() < 1.0 {
                continue;
            }
            
            // Convert cursor delta to world space movement
            // Account for minimap size and world bounds
            let minimap_size = minimap_settings.size.x;
            let container_padding = 4.0;
            let border_width = 2.0;
            let total_padding = container_padding + border_width;
            let map_area_size = minimap_size - (total_padding * 2.0);
            let map_area_height = map_area_size * 0.9; // 90% height for map area
            
            // Convert cursor delta to normalized coordinates (0-1) within map area
            let delta_percent = Vec2::new(
                cursor_delta.x / map_area_size,
                cursor_delta.y / map_area_height
            );
            
            info!("🖱️ Delta percent: {:?}", delta_percent);
            
            // Convert from minimap space to world coordinates using isometric transformation
            let display_angle = -std::f32::consts::PI / 4.0 + std::f32::consts::PI / 2.0; // 45°
            let cos_a = display_angle.cos();
            let sin_a = display_angle.sin();
            
            // Calculate the isometric bounds
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
                    corner.x * sin_a + corner.y * cos_a
                );
                iso_bounds_min = iso_bounds_min.min(iso_corner);
                iso_bounds_max = iso_bounds_max.max(iso_corner);
            }
            
            let iso_world_bounds = iso_bounds_max - iso_bounds_min;
            let delta_iso_pos = delta_percent * iso_world_bounds;
            
            // Apply the INVERSE transformation to get back to world coordinates
            let inv_cos_a = cos_a;  // cos(-θ) = cos(θ)
            let inv_sin_a = -sin_a; // sin(-θ) = -sin(θ)
            
            let delta_world_pos = Vec2::new(
                delta_iso_pos.x * inv_cos_a - delta_iso_pos.y * inv_sin_a,
                delta_iso_pos.x * inv_sin_a + delta_iso_pos.y * inv_cos_a
            );
            
            info!("🖱️ Delta world pos: {:?}", delta_world_pos);
            
            // Update camera position
            if let Ok(mut camera_transform) = camera_query.single_mut() {
                let new_camera_pos = Vec3::new(
                    dragging.start_camera_pos.x + delta_world_pos.x,
                    dragging.start_camera_pos.y,
                    dragging.start_camera_pos.z + delta_world_pos.y
                );
                
                // Clamp to camera bounds
                let clamped_pos = Vec3::new(
                    new_camera_pos.x.clamp(camera_settings.bounds_min.x, camera_settings.bounds_max.x),
                    new_camera_pos.y,
                    new_camera_pos.z.clamp(camera_settings.bounds_min.z, camera_settings.bounds_max.z),
                );
                
                camera_transform.translation = clamped_pos;
                info!("🖱️ Camera moved via drag to: {:?}", clamped_pos);
            }
        }
    }

    // Stop dragging when mouse is released
    if mouse_input.just_released(MouseButton::Left) {
        for (viewport_entity, _) in dragging_query.iter() {
            info!("🖱️ Stopping viewport drag");
            commands.entity(viewport_entity).remove::<MinimapViewportDragging>();
        }
    }
}

/// Update viewport appearance based on interaction state
pub fn update_minimap_viewport_appearance(
    mut viewport_query: Query<(&Interaction, &mut BorderColor), (With<MinimapCameraViewport>, Changed<Interaction>)>,
    dragging_query: Query<&MinimapViewportDragging>,
) {
    for (interaction, mut border_color) in viewport_query.iter_mut() {
        match interaction {
            Interaction::Hovered => {
                border_color.0 = Color::srgb(0.8, 0.8, 1.0); // Light blue when hovered
            }
            Interaction::Pressed => {
                border_color.0 = Color::srgb(1.0, 1.0, 0.0); // Yellow when pressed/dragging
            }
            Interaction::None => {
                // Check if we're currently dragging
                let is_dragging = dragging_query.iter().next().is_some();
                if is_dragging {
                    border_color.0 = Color::srgb(1.0, 1.0, 0.0); // Yellow when dragging
                } else {
                    border_color.0 = Color::WHITE; // Default white
                }
            }
        }
    }
}
