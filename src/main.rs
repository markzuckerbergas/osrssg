//! A simple 3D scene with light shining over a osrs player model.
//! Simple animation control and camera movement.
//!
//! Controls:
//! - Mouse: Left click to select player, right click to move player
//! - arrows/mouse: move camera

use bevy::input::mouse::{MouseButtonInput, MouseWheel};
use bevy::input::ButtonState;
use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_mod_picking::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(
            DefaultPickingPlugins
                .build()
                .disable::<DebugPickingPlugin>()
                .disable::<DefaultHighlightingPlugin>(),
        )
        .init_resource::<GameData>()
        .add_startup_system(setup)
        .add_system(keyboard_camera_movement)
        .add_system(mouse_camera_movement)
        .add_system(make_pickable)
        .add_system(set_location_and_start_movement)
        .add_system(move_entities_to_location)
        .add_event::<DeselectAllEvent>()
        .add_system(deselect_all_entities.run_if(on_event::<DeselectAllEvent>()))
        .run();
}

#[derive(Resource)]
struct Animations(Vec<Handle<AnimationClip>>);

#[derive(Component)]
struct Movable {}

#[derive(Component)]
struct PlayerName(String);

#[derive(Component)]
struct Selected {}

#[derive(Component)]
struct Moving {}

#[derive(Bundle)]
struct PlayerBundle {
    name: PlayerName,

    #[bundle]
    scene: SceneBundle,
}

#[derive(Component)]
struct Ground;

#[derive(Resource, Default)]
struct GameData {
    destination: Vec3,
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(20.0).into()),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        },
        Ground,
        OnPointer::<Click>::send_event::<DeselectAllEvent>(),
        PickHighlight,
    ));

    // default player
    let mut player_transform = Transform::from_xyz(0.0, 0.05, 0.0);
    player_transform.scale = Vec3::splat(0.03);

    let player = PlayerBundle {
        name: PlayerName("Player1".to_string()),
        scene: SceneBundle {
            scene: asset_server.load("player.glb#Scene0"),
            transform: player_transform,
            ..default()
        },
    };

    commands.spawn((
        player,
        Movable {},
        OnPointer::<Click>::commands_mut(|event, commands| {
            info!("Player selected!");
            commands.entity(event.listener).insert(Selected {});
        }),
    ));

    // animations
    commands.insert_resource(Animations(vec![
        asset_server.load("player.glb#Animation0"), // Start flying animation
        asset_server.load("player.glb#Animation1"), // Return to idle
    ]));

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // camera
    commands.spawn((
        Camera3dBundle {
            projection: OrthographicProjection {
                scale: 5.0,
                scaling_mode: ScalingMode::FixedVertical(2.0),
                ..default()
            }
            .into(),
            transform: Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        RaycastPickCamera::default(),
    ));
}

/// Makes everything in the scene with a mesh pickable
fn make_pickable(
    mut commands: Commands,
    meshes: Query<Entity, (With<Handle<Mesh>>, Without<RaycastPickTarget>)>,
) {
    for entity in meshes.iter() {
        commands
            .entity(entity)
            .insert((PickableBundle::default(), RaycastPickTarget::default()));
    }
}

fn keyboard_camera_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut camera: Query<&mut Transform, With<Camera>>,
) {
    for mut transform in camera.iter_mut() {
        let mut translation = Vec3::ZERO;
        if keyboard_input.pressed(KeyCode::Left) {
            translation -= transform.rotation * Vec3::X;
        }
        if keyboard_input.pressed(KeyCode::Right) {
            translation += transform.rotation * Vec3::X;
        }
        if keyboard_input.pressed(KeyCode::Up) {
            translation += transform.rotation * Vec3::Y;
        }
        if keyboard_input.pressed(KeyCode::Down) {
            translation -= transform.rotation * Vec3::Y;
        }
        transform.translation += translation * 0.1;
    }
}

fn mouse_camera_movement(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut camera: Query<&mut Transform, With<Camera>>,
    mut windows: Query<&mut Window>,
) {
    for event in cursor_moved_events.iter() {
        // Camera needs to move when the mouse is near the edge of the screen
        // 1) First we need to get the size of the window
        let window = windows.single_mut();
        let scale_factor = window.resolution.scale_factor() as f32;
        let physical_width = window.resolution.physical_width() as f32;
        let physical_height = window.resolution.physical_height() as f32;

        let actual_resolution = Vec2::new(
            physical_width / scale_factor,
            physical_height / scale_factor,
        );

        // 2) Then we need to get the mouse position
        let mouse_position = event.position;

        // 3) Then we need to get the center of the screen
        let center = actual_resolution / 2.0;

        // 4) Then we need to get the difference between the mouse position and the center
        let difference = mouse_position - center;

        // 5) Then we need to scale the difference based on the size of the window
        let scaled_difference = difference / actual_resolution;

        // 6) Then we need to move the camera based on the difference
        // 6.1) Only move the camera if the mouse is near the edge of the screen
        if scaled_difference.x.abs() > 0.48 || scaled_difference.y.abs() > 0.48 {
            for mut transform in camera.iter_mut() {
                transform.translation +=
                    Vec3::new(scaled_difference.x * 0.1, scaled_difference.y * 0.1, 0.0);
            }
        }
    }

    for event in mouse_wheel_events.iter() {
        // Handle zoom
        for mut transform in camera.iter_mut() {
            transform.scale *= 1.0 + -event.y / 20.0;
        }
    }
}

fn set_location_and_start_movement(
    mut commands: Commands,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    selected_entities: Query<(Entity, &mut Selected)>,
    ground_query: Query<&Transform, With<Ground>>,
    query_camera: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&mut Window>,
    mut animation_players: Query<&mut AnimationPlayer>,
    animations: Res<Animations>,
) {
    for event in mouse_button_input_events.iter() {
        if event.button == MouseButton::Right
            && event.state == ButtonState::Pressed
            && selected_entities.iter().count() > 0
        {
            let (camera, camera_transform) = query_camera.single();
            let ground = ground_query.single();

            let Some(cursor_position) = windows.single().cursor_position() else { return; };

            // Calculate a ray pointing from the camera into the world based on the cursor's position.
            let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else { return; };

            // Calculate if and where the ray is hitting the ground plane.
            let Some(distance) = ray.intersect_plane(ground.translation, ground.up()) else { return; };
            let point = ray.get_point(distance);

            commands.insert_resource(GameData { destination: point });

            for (entity, _) in selected_entities.iter() {
                commands.entity(entity).insert(Moving {});
            }

            for mut player in animation_players.iter_mut() {
                player.play(animations.0[0].clone_weak());
            }
        }
    }
}

struct DeselectAllEvent();

impl From<ListenedEvent<Click>> for DeselectAllEvent {
    fn from(_: ListenedEvent<Click>) -> Self {
        DeselectAllEvent()
    }
}

fn deselect_all_entities(
    mut commands: Commands,
    query: Query<(Entity, &Selected)>,
    mouse_button_input: Res<Input<MouseButton>>,
) {
    if mouse_button_input.just_released(MouseButton::Left) {
        for (entity, _) in query.iter() {
            commands.entity(entity).remove::<Selected>();
        }
    }
}

fn move_entities_to_location(
    mut query: Query<(&mut Transform, &Moving, &Movable, Entity)>,
    mut commands: Commands,
    game_data: ResMut<GameData>,
    mut animation_players: Query<&mut AnimationPlayer>,
    animations: Res<Animations>,
) {
    for (mut transform, _, _, entity) in query.iter_mut() {
        let destination = game_data.destination;

        // Rotate the player to face the point
        let direction = destination - transform.translation;
        let rotation = Quat::from_rotation_y(direction.x.atan2(direction.z));
        transform.rotation = rotation;

        // Ignore the y axis
        // Smoothly move the player to the point
        let new_point = Vec3::new(destination.x, transform.translation.y, destination.z);

        // if player is near the destination, just set the position
        if transform.translation.distance(new_point) < 0.1 {
            commands.entity(entity).remove::<Moving>();
            for mut player in animation_players.iter_mut() {
                player.play(animations.0[1].clone_weak());
            }
        } else {
            transform.translation = transform.translation.lerp(new_point, 0.01);
        }
    }
}
