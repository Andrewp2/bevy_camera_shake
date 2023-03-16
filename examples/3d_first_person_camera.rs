use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
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

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane {
            size: 150.0,
            subdivisions: 0,
        })),
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
            transform: Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::NEG_Z, Vec3::Y),
            ..Default::default()
        })
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
        .spawn(Player { speed: 5.0 })
        .insert(SpatialBundle {
            ..Default::default()
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
    commands.entity(shake_id).push_children(&[camera_id]);
    println!("Press R to add trauma to the camera.");
}

const TRAUMA_AMOUNT: f32 = 0.5;

fn add_shake(mut shakeables: Query<&mut Shake3d>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::R) {
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

use bevy::ecs::event::{Events, ManualEventReader};
use bevy::input::mouse::MouseMotion;

/// Keeps track of mouse motion events, pitch, and yaw
#[derive(Default, Resource)]
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
    window.cursor.grab_mode = match window.cursor.grab_mode {
        CursorGrabMode::None => CursorGrabMode::Confined,
        CursorGrabMode::Confined => CursorGrabMode::None,
        CursorGrabMode::Locked => CursorGrabMode::None,
    };
    window.cursor.visible = !window.cursor.visible;
}

/// Grabs the cursor when game first starts
fn initial_grab_cursor(mut windows: Query<(&mut Window, With<PrimaryWindow>)>) {
    match windows.get_single_mut() {
        Ok(mut window) => toggle_grab_cursor(&mut window.0),
        Err(_) => warn!("Primary window not found for `initial_grab_cursor`!"),
    }
}

/// Handles keyboard input and movement
fn player_move(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut windows: Query<(&Window, With<PrimaryWindow>)>,
    mut query: Query<(&mut Transform, &Player)>,
) {
    if let Ok(window) = windows.get_single() {
        for (mut transform, player) in query.iter_mut() {
            let mut velocity = Vec3::ZERO;
            let local_z = transform.local_z();
            let forward = -Vec3::new(local_z.x, 0., local_z.z);
            let right = Vec3::new(local_z.z, 0., -local_z.x);

            for key in keys.get_pressed() {
                if let CursorGrabMode::Confined | CursorGrabMode::Locked = window.0.cursor.grab_mode
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
    windows: Query<(&Window, With<PrimaryWindow>)>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
    mut query: Query<(&mut Transform, &Player)>,
) {
    if let Ok(window) = windows.get_single() {
        let mut delta_state = state.as_mut();
        for (mut transform, _) in query.iter_mut() {
            for ev in delta_state.reader_motion.iter(&motion) {
                if let CursorGrabMode::Confined | CursorGrabMode::Locked = window.0.cursor.grab_mode
                {
                    // Using smallest of height or width ensures equal vertical and horizontal sensitivity
                    let window_scale = window.0.height().min(window.0.width());
                    delta_state.pitch -=
                        (settings.sensitivity * ev.delta.y * window_scale).to_radians();
                    delta_state.yaw -=
                        (settings.sensitivity * ev.delta.x * window_scale).to_radians();
                }

                delta_state.pitch = delta_state
                    .pitch
                    .clamp(f32::to_radians(-180.0), f32::to_radians(180.0));

                // Order is important to prevent unintended roll
                transform.rotation = Quat::from_axis_angle(Vec3::Y, delta_state.yaw)
                    * Quat::from_axis_angle(Vec3::X, delta_state.pitch);
            }
        }
    } else {
        warn!("Primary window not found for `player_look`!");
    }
}

fn cursor_grab(keys: Res<Input<KeyCode>>, mut windows: Query<(&mut Window, With<PrimaryWindow>)>) {
    if let Ok(mut window) = windows.get_single_mut() {
        if keys.just_pressed(KeyCode::Escape) {
            toggle_grab_cursor(&mut window.0);
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
