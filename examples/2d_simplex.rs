use bevy::prelude::*;
use bevy_camera_shake::{CameraShakePlugin, RandomSource, Shake2d};
use noise::{NoiseFn, OpenSimplex};
use rand::{thread_rng, Rng};

// In this example, we use Simplex noise instead of random noise, to get a "smoother" shake.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CameraShakePlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, player_movement)
        .add_systems(Update, add_shake)
        .run();
}

// Struct used ot hold our simplex noise.
// We have to use a seperate struct (rather than implementing `RandomSource` on `OpenSimplex` directly)
// due to https://doc.rust-lang.org/error_codes/E0117.html
struct MyNoise {
    pub simplex: OpenSimplex,
}

impl MyNoise {
    pub fn new(seed: u32) -> MyNoise {
        MyNoise {
            simplex: OpenSimplex::new(seed),
        }
    }
}

impl RandomSource for MyNoise {
    fn rand(&self, time: f32) -> f32 {
        self.simplex.get([(time * 15.0) as f64, 0.0]) as f32
    }
}

fn random_number() -> f32 {
    let mut rng = thread_rng();
    let x: f32 = rng.gen();
    x * 2.0 - 1.0
}

// Player marker component.
#[derive(Component)]
struct Player;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Create a default camera.
    let camera_id = commands.spawn(Camera2d).id();
    // Create a Shake2d entity.
    // We set decay to a lower value of 0.7. That means after 1 second, only `1.0 - 0.7` or 30% of the trauma will remain.
    // After two seconds, we will have no trauma left because `(1.0 - (2.0 * 0.7)) < 0.0`.
    let shake_id = commands
        .spawn(Shake2d {
            max_offset: Vec2::new(90.0, 45.0),
            max_roll: 0.1,
            trauma: 0.0,
            trauma_power: 2.0,
            decay: 0.7,
            random_sources: [
                Box::new(MyNoise::new(0)),
                Box::new(MyNoise::new(1)),
                Box::new(MyNoise::new(2)),
            ],
        })
        .insert(Transform::default())
        .id();

    // Create the player entity.
    let player_id = commands
        .spawn((
            Sprite {
                image: asset_server.load("duck.png"),
                ..default()
            },
            Transform::from_xyz(0., 0., 0.),
        ))
        .insert(Player)
        .id();

    // Spawn 5,000 ducks (to see the effects of shaking).
    for _ in 0..5000 {
        commands.spawn((
            Sprite {
                image: asset_server.load("duck.png"),
                ..default()
            },
            Transform {
                translation: Vec3::new((random_number()) * 2000.0, (random_number()) * 1000.0, 0.0),
                rotation: Quat::default(),
                scale: Vec3::new(0.3, 0.3, 1.0),
            },
        ));
    }
    // Make the player the parent of the Shake2d entity.
    commands.entity(player_id).add_children(&[shake_id]);
    // Make the Shake2d entity the parent of the camera (so the camera moves when the Shake2d component shakes).
    commands.entity(shake_id).add_children(&[camera_id]);
    println!("Press R to add trauma to the camera.");
}

// Move up/down/left/right based on player's input.
fn player_movement(
    time: Res<Time>,
    mut player_position: Query<(&mut Transform, &Player)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let speed = 150.0;
    let mut velocity = Vec2::new(0.0, 0.0);
    if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
        velocity.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
        velocity.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
        velocity.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
        velocity.x += 1.0;
    }

    for (mut t, _) in player_position.iter_mut() {
        t.translation.x += velocity.x * time.delta_secs() * speed;
        t.translation.y += velocity.y * time.delta_secs() * speed;
    }
}

// The amount of trauma to add per button press.
const TRAUMA_AMOUNT: f32 = 0.5;

// Adds trauma to all Shake2d entities when the `r` key is pressed.
// Limits trauma to a maximum of `1.0`.
fn add_shake(mut shakeables: Query<&mut Shake2d>, keyboard_input: Res<ButtonInput<KeyCode>>) {
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
