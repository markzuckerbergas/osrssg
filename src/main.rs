//! A simple 3D scene with light shining over a osrs player model.
//! //! This example illustrates how to load a glTF model with animations and play them back.
//!
//! Controls:
//! - return: start / change animation
//! - spacebar: play / pause
//! - arrow up / down: speed up / slow down animation playback
//! - arrow left / right: seek backward / forward

use bevy::{prelude::*, render::camera::ScalingMode};
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(setup_scene_once_loaded)
        .add_system(keyboard_animation_control)
        .add_system(keyboard_camera_movement)
        .run();
}

#[derive(Resource)]
struct Animations(Vec<Handle<AnimationClip>>);

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(5.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });

    // player
    let mut player_transform = Transform::from_xyz(0.0, 0.05, 0.0);
    player_transform.scale = Vec3::splat(0.03);

    commands.spawn(SceneBundle {
        scene: asset_server.load("player.glb#Scene0"),
        transform: player_transform,
        ..default()
    });

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
    commands.spawn(Camera3dBundle {
        projection: OrthographicProjection {
            scale: 3.0,
            scaling_mode: ScalingMode::FixedVertical(2.0),
            ..default()
        }
        .into(),
        transform: Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    println!("Animation controls:");
    println!("  - spacebar: play / pause");
    println!("  - arrow up / down: speed up / slow down animation playback");
    println!("  - arrow left / right: seek backward / forward");
    println!("  - return: change animation");
}

fn setup_scene_once_loaded(
    animations: Res<Animations>,
    mut players: Query<&mut AnimationPlayer, Added<AnimationPlayer>>,
) {
    for mut player in &mut players {
        player.play(animations.0[0].clone_weak());
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

fn keyboard_animation_control(
    keyboard_input: Res<Input<KeyCode>>,
    mut animation_players: Query<&mut AnimationPlayer>,
    animations: Res<Animations>,
    mut current_animation: Local<usize>,
) {
    for mut player in &mut animation_players {
        if keyboard_input.just_pressed(KeyCode::Space) {
            if player.is_paused() {
                player.resume();
            } else {
                player.pause();
            }
        }

        if keyboard_input.just_pressed(KeyCode::Return) {
            *current_animation = (*current_animation + 1) % animations.0.len();
            player.play_with_transition(
                animations.0[*current_animation].clone_weak(),
                Duration::from_millis(250),
            );
        }
    }
}
