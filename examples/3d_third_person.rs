use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_camera_shake::{CameraShakePlugin, RandomSource, Shake3d};
use rand::{thread_rng, Rng};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CameraShakePlugin)
        .add_plugins(NoCameraPlayerPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, add_shake)
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
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Plane3d::new(Vec3::Y, Vec2::splat(150.0))))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::from_xyz(0.0, -0.5, 0.0),
    ));
    commands.spawn((
        PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    let camera_id = commands
        .spawn((
            Camera3d::default(),
            Transform::from_xyz(0.0, 0.5, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        ))
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
        .insert(Transform::default())
        .id();

    let player_transform = Transform::default();
    let player_id = commands
        .spawn((
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(1.0, 1.0, 1.0)))),
            MeshMaterial3d(materials.add(Color::srgb(1.0, 0.8, 0.6))),
            player_transform,
        ))
        .insert(Player { speed: 5.0 })
        .id();

    let player_face_id = commands
        .spawn((
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(1.0, 1.0, 1.0)))),
            MeshMaterial3d(materials.add(Color::srgb(0.0, 0.8, 0.6))),
            Transform::from_translation(player_transform.forward() * 0.5)
                .with_scale(Vec3::new(0.3, 0.1, 0.5)),
        ))
        .id();

    for _ in 0..250 {
        commands.spawn((
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(1.0, 1.0, 1.0)))),
            MeshMaterial3d(materials.add(Color::srgb(0.3, 0.7, 0.8))),
            Transform {
                translation: Vec3::new(
                    (random_number()) * 10.0,
                    ((random_number()) + 1.0) * 10.0,
                    (random_number()) * 10.0,
                ),
                rotation: Quat::default(),
                scale: Vec3::new(0.3, 0.3, 0.3),
            },
        ));
    }
    commands.entity(player_id).add_children(&[shake_id]);
    commands.entity(player_id).add_children(&[player_face_id]);
    commands.entity(shake_id).add_children(&[camera_id]);
    println!("Press R to add trauma to the camera.");
}

const TRAUMA_AMOUNT: f32 = 0.5;

fn add_shake(mut shakeables: Query<&mut Shake3d>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        for mut shakeable in shakeables.iter_mut() {
            let past_trauma = shakeable.trauma;
            let current_trauma = f32::min(shakeable.trauma + TRAUMA_AMOUNT, 1.0);
            info!(
                "Past trauma: {}, Current trauma: {}",
                past_trauma, current_trauma
            );
            shakeable.trauma = current_trauma;
        }
    }
}

// Code shamelessly stolen and edited from https://github.com/sburris0/bevy_flycam/blob/master/src/lib.rs

use bevy::ecs::event::{EventCursor, Events};
use bevy::input::mouse::MouseMotion;

/// Keeps track of mouse motion events, pitch, and yaw
#[derive(Default, Debug, Resource)]
struct InputState {
    reader_motion: EventCursor<MouseMotion>,
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
    window.cursor_options.grab_mode = match window.cursor_options.grab_mode {
        CursorGrabMode::None => CursorGrabMode::Confined,
        CursorGrabMode::Confined => CursorGrabMode::None,
        CursorGrabMode::Locked => CursorGrabMode::None,
    };
    window.cursor_options.visible = !window.cursor_options.visible;
}

/// Grabs the cursor when game first starts
fn initial_grab_cursor(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut frame_count: Local<u32>,
    mut grabbed_once: Local<bool>,
) {
    if *grabbed_once {
        return;
    }
    // wait until 5 frames have passed (see https://github.com/bevyengine/bevy/issues/16237)
    *frame_count += 1;
    if *frame_count < 5 {
        return;
    }
    match windows.single_mut() {
        Ok(mut window) => {
            toggle_grab_cursor(&mut window);
            *grabbed_once = true;
        }
        Err(_) => warn!("Primary window not found for initial_grab_cursor!"),
    }
}

/// Handles keyboard input and movement
fn player_move(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<(&mut Transform, &Player)>,
) {
    if let Ok(window) = windows.single() {
        for (mut transform, player) in query.iter_mut() {
            let mut velocity = Vec3::ZERO;
            let forward = transform.forward();
            let right = transform.right();
            for key in keys.get_pressed() {
                if let CursorGrabMode::Confined | CursorGrabMode::Locked =
                    window.cursor_options.grab_mode
                {
                    match key {
                        KeyCode::KeyW => velocity += *forward,
                        KeyCode::KeyS => velocity -= *forward,
                        KeyCode::KeyA => velocity -= *right,
                        KeyCode::KeyD => velocity += *right,
                        KeyCode::Space => velocity += Vec3::Y,
                        KeyCode::ShiftLeft => velocity -= Vec3::Y,
                        _ => (),
                    }
                }
            }

            velocity = velocity.normalize_or_zero();

            transform.translation += velocity * time.delta_secs() * player.speed;
        }
    } else {
        warn!("Primary window not found for `player_move`!");
    }
}

/// Handles looking around if cursor is locked
fn player_look(
    settings: Res<MouseSensitivity>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
    mut camera_query: Query<(&mut Transform, &ThirdPersonCamera), Without<Player>>,
    mut player_query: Query<(&mut Transform, &Player), Without<ThirdPersonCamera>>,
) {
    if let Ok(window) = windows.single() {
        let delta_state = state.as_mut();
        for (mut transform, _) in camera_query.iter_mut() {
            let (mut player_transform, _) = player_query.single_mut().unwrap();
            for ev in delta_state.reader_motion.read(&motion) {
                if let CursorGrabMode::Confined | CursorGrabMode::Locked =
                    window.cursor_options.grab_mode
                {
                    let window_scale = window.height().min(window.width());
                    delta_state.pitch -=
                        (settings.sensitivity * ev.delta.y * window_scale).to_radians();
                    delta_state.yaw -=
                        (settings.sensitivity * ev.delta.x * window_scale).to_radians();
                    delta_state.pitch = delta_state
                        .pitch
                        .clamp(f32::to_radians(-75.0), f32::to_radians(-5.0));
                    player_transform.rotation = Quat::from_axis_angle(Vec3::Y, delta_state.yaw);
                    transform.rotation = Quat::from_axis_angle(Vec3::X, delta_state.pitch);
                    transform.translation = transform.rotation.mul_vec3(Vec3::Z) * 15.0;
                }
            }
        }
    } else {
        warn!("Primary window not found for `player_look`!");
    }
}

fn cursor_grab(
    keys: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = windows.single_mut() {
        if keys.just_pressed(KeyCode::Escape) {
            toggle_grab_cursor(&mut window);
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
            .add_systems(Update, initial_grab_cursor)
            .add_systems(Update, player_move)
            .add_systems(Update, player_look)
            .add_systems(Update, cursor_grab);
    }
}
