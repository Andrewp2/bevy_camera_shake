use bevy::prelude::*;
use bevy_camera_shake::{CameraShakePlugin, RandomSource, Shake2d};
use rand::{thread_rng, Rng};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(CameraShakePlugin)
        .add_startup_system(setup)
        .add_system(player_movement)
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
struct Player;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let camera_id = commands.spawn_bundle(Camera2dBundle::default()).id();
    let shake_id = commands
        .spawn()
        .insert(Shake2d {
            max_offset: Vec2::new(90.0, 45.0),
            max_roll: 0.2,
            trauma: 0.0,
            trauma_power: 2.0,
            decay: 0.8,
            random_sources: [Box::new(MyRandom), Box::new(MyRandom), Box::new(MyRandom)],
        })
        .insert_bundle(SpatialBundle::default())
        .id();

    let player_id = commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("duck.png"),
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        })
        .insert(Player)
        .id();

    for _ in 0..5000 {
        commands.spawn_bundle(SpriteBundle {
            texture: asset_server.load("duck.png"),
            transform: Transform {
                translation: Vec3::new((random_number()) * 2000.0, (random_number()) * 1000.0, 0.0),
                rotation: Quat::default(),
                scale: Vec3::new(0.3, 0.3, 1.0),
            },
            ..default()
        });
    }
    commands.entity(player_id).push_children(&[shake_id]);
    commands.entity(shake_id).push_children(&[camera_id]);
}

fn player_movement(
    time: Res<Time>,
    mut player_position: Query<(&mut Transform, &Player)>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let speed = 150.0;
    let mut velocity = Vec2::new(0.0, 0.0);
    if keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up) {
        velocity.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left) {
        velocity.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down) {
        velocity.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right) {
        velocity.x += 1.0;
    }

    for (mut t, _) in player_position.iter_mut() {
        t.translation.x += velocity.x * time.delta_seconds() * speed;
        t.translation.y += velocity.y * time.delta_seconds() * speed;
    }
}

const TRAUMA_AMOUNT: f32 = 0.5;

fn add_shake(mut shakeables: Query<&mut Shake2d>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::R) {
        info!("Time to shake!");
        for mut shakeable in shakeables.iter_mut() {
            shakeable.trauma = f32::min(shakeable.trauma + TRAUMA_AMOUNT, 1.0);
        }
    }
}
