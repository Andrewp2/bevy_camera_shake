use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use bevy_camera_shake::{CameraShakePlugin, RandomSource, Shake3d};
use rand::{thread_rng, Rng};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(CameraShakePlugin)
        .add_plugin(NoCameraPlayerPlugin)
        .add_startup_system(setup)
        .add_system(add_shake)
        .run();
}

fn random_number() -> f32 {
    let mut rng = thread_rng();
    let x: f32 = rng.gen();
    x * 2.0 - 1.0
}
struct MyRandom;

impl RandomSource for MyRandom {
    fn rand(&self, _time: f32) -> f32 {
        random_number()
    }
}

#[derive(Component)]
struct Player {
    pub speed: f32,
}

#[derive(Component)]
struct ThirdPersonCamera;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 150.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        transform: Transform::from_xyz(0.0, -0.5, 0.0),
        ..default()
    });
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    let camera_id = commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, -15.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(ThirdPersonCamera)
        .id();
    let shake_id = commands
        .spawn(Shake3d {
            max_offset: Vec3::new(0.0, 0.0, 0.0),
            max_yaw_pitch_roll: Vec3::new(0.1, 0.1, 0.1),
            trauma: 0.0,
            trauma_power: 2.0,
            decay: 0.8,
            random_sources: [
                Box::new(MyRandom),
                Box::new(MyRandom),
                Box::new(MyRandom),
                Box::new(MyRandom),
                Box::new(MyRandom),
                Box::new(MyRandom),
            ],
        })
        .insert(SpatialBundle::default())
        .id();

    let player_id = commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(1.0, 0.8, 0.6).into()),
            transform: Transform::default(),
            ..default()
        })
        .insert(Player { speed: 5.0 })
        .id();

    let player_face_id = commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(1.0, 0.8, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.5).with_scale(Vec3::new(0.3, 0.1, 0.5)),
            ..default()
        })
        .id();

    for _ in 0..250 {
        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.3, 0.7, 0.8).into()),
            transform: Transform {
                translation: Vec3::new(
                    (random_number()) * 10.0,
                    ((random_number()) + 1.0) * 10.0,
                    (random_number()) * 10.0,
                ),
                rotation: Quat::default(),
                scale: Vec3::new(0.3, 0.3, 0.3),
            },
            ..default()
        });
    }
    commands.entity(player_id).push_children(&[shake_id]);
    commands.entity(player_id).push_children(&[player_face_id]);
    commands.entity(shake_id).push_children(&[camera_id]);
    println!("Press R to add trauma to the camera.");
}

const TRAUMA_AMOUNT: f32 = 0.5;

fn add_shake(mut shakeables: Query<&mut Shake3d>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::R) {
        info!("Time to shake!");
        for mut shakeable in shakeables.iter_mut() {
            shakeable.trauma = f32::min(shakeable.trauma + TRAUMA_AMOUNT, 1.0);
        }
    }
}

// Code shamelessly stolen and edited from https://github.com/sburris0/bevy_flycam/blob/master/src/lib.rs

use bevy::ecs::event::{Events, ManualEventReader};
use bevy::input::mouse::MouseMotion;

/// Keeps track of mouse motion events, pitch, and yaw
#[derive(Default, Debug, Resource)]
struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
    pitch: f32,
    yaw: f32,
}

/// Mouse sensitivity
#[derive(Resource)]
pub struct MouseSensitivity {
    pub sensitivity: f32,
}

impl Default for MouseSensitivity {
    fn default() -> Self {
        Self {
            sensitivity: 0.00012,
        }
    }
}

/// Grabs/ungrabs mouse cursor
fn toggle_grab_cursor(window: &mut Window) {
    window.set_cursor_grab_mode(match window.cursor_grab_mode() {
        CursorGrabMode::None => CursorGrabMode::Confined,
        CursorGrabMode::Confined => CursorGrabMode::None,
        CursorGrabMode::Locked => CursorGrabMode::None,
    });
    window.set_cursor_visibility(!window.cursor_visible());
}

/// Grabs the cursor when game first starts
fn initial_grab_cursor(mut windows: ResMut<Windows>) {
    if let Some(window) = windows.get_primary_mut() {
        toggle_grab_cursor(window);
    } else {
        warn!("Primary window not found for `initial_grab_cursor`!");
    }
}

/// Handles keyboard input and movement
fn player_move(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    windows: Res<Windows>,
    mut query: Query<(&mut Transform, &Player)>,
) {
    if let Some(window) = windows.get_primary() {
        for (mut transform, player) in query.iter_mut() {
            let mut velocity = Vec3::ZERO;
            let local_z = transform.local_z();
            let forward = Vec3::new(local_z.x, 0., local_z.z);
            let right = Vec3::new(-local_z.z, 0., local_z.x);

            for key in keys.get_pressed() {
                if let CursorGrabMode::Confined | CursorGrabMode::Locked = window.cursor_grab_mode()
                {
                    match key {
                        KeyCode::W => velocity += forward,
                        KeyCode::S => velocity -= forward,
                        KeyCode::A => velocity -= right,
                        KeyCode::D => velocity += right,
                        KeyCode::Space => velocity += Vec3::Y,
                        KeyCode::LShift => velocity -= Vec3::Y,
                        _ => (),
                    }
                }
            }

            velocity = velocity.normalize_or_zero();

            transform.translation += velocity * time.delta_seconds() * player.speed;
        }
    } else {
        warn!("Primary window not found for `player_move`!");
    }
}

/// Handles looking around if cursor is locked
fn player_look(
    settings: Res<MouseSensitivity>,
    windows: Res<Windows>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
    mut camera_query: Query<(&mut Transform, &ThirdPersonCamera), Without<Player>>,
    mut player_query: Query<(&mut Transform, &Player), Without<ThirdPersonCamera>>,
) {
    if let Some(window) = windows.get_primary() {
        let mut delta_state = state.as_mut();
        for (mut transform, _) in camera_query.iter_mut() {
            let (mut player_transform, _) = player_query.get_single_mut().unwrap();
            for ev in delta_state.reader_motion.iter(&motion) {
                if let CursorGrabMode::Confined | CursorGrabMode::Locked = window.cursor_grab_mode()
                {
                    let window_scale = window.height().min(window.width());
                    delta_state.pitch -=
                        (settings.sensitivity * ev.delta.y * window_scale).to_radians();
                    delta_state.yaw -=
                        (settings.sensitivity * ev.delta.x * window_scale).to_radians();
                    delta_state.pitch = delta_state
                        .pitch
                        .clamp(f32::to_radians(-75.0), f32::to_radians(5.0));
                    // Order is important to prevent unintended roll
                    // Yaw rotation is happening on player instead
                    transform.rotation = Quat::from_axis_angle(Vec3::Y, PI)
                        * Quat::from_axis_angle(Vec3::X, delta_state.pitch);
                    transform.translation = transform.rotation.mul_vec3(Vec3::Z) * 15.0;
                    player_transform.rotation = Quat::from_axis_angle(Vec3::Y, delta_state.yaw);
                }
            }
        }
    } else {
        warn!("Primary window not found for `player_look`!");
    }
}

fn cursor_grab(keys: Res<Input<KeyCode>>, mut windows: ResMut<Windows>) {
    if let Some(window) = windows.get_primary_mut() {
        if keys.just_pressed(KeyCode::Escape) {
            toggle_grab_cursor(window);
        }
    } else {
        warn!("Primary window not found for `cursor_grab`!");
    }
}

/// Same as [`PlayerPlugin`] but does not spawn a camera
pub struct NoCameraPlayerPlugin;
impl Plugin for NoCameraPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .init_resource::<MouseSensitivity>()
            .add_startup_system(initial_grab_cursor)
            .add_system(player_move)
            .add_system(player_look)
            .add_system(cursor_grab);
    }
}
