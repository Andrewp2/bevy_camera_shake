use bevy::{
    log::warn,
    prelude::{Component, EulerRot, Plugin, Quat, Query, Res, Transform, Update, Vec2, Vec3},
    time::Time,
};

/// A source of randomness for shaking the camera.
pub trait RandomSource: Send + Sync {
    /// Produces a random float between -1.0 and 1.0.
    fn rand(&self, time: f32) -> f32;
}

/// A not-random RandomSource. Always returns 0.5.
struct NotRandom;

impl RandomSource for NotRandom {
    fn rand(&self, _time: f32) -> f32 {
        warn!("You need to set a random source for the shaking to work properly!");
        0.5
    }
}

/// The component to be added for a 3d shake.
/// Will modify the transform of the entity that this component is added to.
/// The entity must also have a `Transform` component for the shake to work.
#[derive(Component)]
pub struct Shake3d {
    /// The maximum amount to translate laterally in all 3 dimensions.
    /// Defaults to `Vec3::new(0.0, 0.0, 0.0)`.
    /// Setting this to a non-zero value can cause the camera to poke through walls.
    pub max_offset: Vec3,
    /// The maximum amount to rotate in yaw, pitch, and roll.
    /// Defaults to  `Vec3::new(0.1, 0.1, 0.1)`.
    pub max_yaw_pitch_roll: Vec3,
    /// The starting trauma when created.
    /// Defaults to `0.0`.
    pub trauma: f32,
    /// The exponent of the trauma used when calculating offset and rotational shakiness.
    /// Should likely be set to a value between `2.0` and `3.0`.
    /// Defaults to `2.0`.
    pub trauma_power: f32,
    /// The percentage to decrease trauma per second.
    /// If set to 1, there will be no trauma after 1 second. If set to 0, trauma will not decrease over time.
    /// Defaults to `0.8`.
    pub decay: f32,
    /// The random sources for all 6 dimensions.
    /// The first 3 are XYZ lateral motion, the last 3 are yaw/pitch/roll.
    /// Defaults to a `NotRandom`, which always returns `0.5`.
    pub random_sources: [Box<dyn RandomSource>; 6],
}

impl Default for Shake3d {
    fn default() -> Self {
        Self {
            max_offset: Vec3::new(0.0, 0.0, 0.0),
            max_yaw_pitch_roll: Vec3::new(0.1, 0.1, 0.1),
            trauma: 0.0,
            trauma_power: 2.0,
            decay: 0.8,
            random_sources: [
                Box::new(NotRandom),
                Box::new(NotRandom),
                Box::new(NotRandom),
                Box::new(NotRandom),
                Box::new(NotRandom),
                Box::new(NotRandom),
            ],
        }
    }
}

/// The component to be added for a 2d shake.
/// Will modify the transform of the entity that this component is added to.
/// The entity must also have a `Transform` component for the shake to work.
#[derive(Component)]
pub struct Shake2d {
    /// The maximum amount of offset in the X and Y dimensions.
    /// Defaults to `Vec2::new(100.0, 100.0)`.
    pub max_offset: Vec2,
    /// The maximum amount of roll allowed in radians.
    /// Defaults to `0.1`.
    pub max_roll: f32,
    /// The starting trauma when created.
    /// Defaults to `0.0`.
    pub trauma: f32,
    /// The exponent of the trauma used when calculating offset and rotational shakiness.
    /// Should likely be set to a value between `2.0` and `3.0`.
    /// Defaults to `2.0`.
    pub trauma_power: f32,
    /// The percentage to decrease trauma per second.
    /// If set to 1, there will be no trauma after 1 second. If set to 0, trauma will not decrease over time.
    /// If set below 0, trauma will *increase* over time, and if set above 1, trauma will decrease very quickly.
    /// Defaults to `0.8`.
    pub decay: f32,
    /// The random sources for all 3 dimensions.
    /// The first 2 are for XY lateral motion, the last one is for roll.
    /// Defaults to a `NotRandom`, which always returns `0.5`.
    pub random_sources: [Box<dyn RandomSource>; 3],
}

impl Default for Shake2d {
    fn default() -> Self {
        Self {
            max_offset: Vec2::new(100.0, 100.0),
            max_roll: 0.1,
            trauma: 0.0,
            trauma_power: 2.0,
            decay: 0.8,
            random_sources: [
                Box::new(NotRandom),
                Box::new(NotRandom),
                Box::new(NotRandom),
            ],
        }
    }
}

fn apply_shake_3d(mut query: Query<(&mut Transform, &mut Shake3d)>, time: Res<Time>) {
    for (mut transform, mut shake_settings) in query.iter_mut() {
        shake_settings.trauma = f32::max(
            shake_settings.trauma - shake_settings.decay * time.delta_secs(),
            0.0,
        );

        let trauma_amount = f32::powf(shake_settings.trauma, shake_settings.trauma_power);

        if trauma_amount > 0.0 {
            let shake_translation = shake_settings.max_offset
                * trauma_amount
                * Vec3::new(
                    shake_settings.random_sources[0].rand(time.elapsed_secs()),
                    shake_settings.random_sources[1].rand(time.elapsed_secs()),
                    shake_settings.random_sources[2].rand(time.elapsed_secs()),
                );

            let rotation = shake_settings.max_yaw_pitch_roll
                * trauma_amount
                * Vec3::new(
                    shake_settings.random_sources[3].rand(time.elapsed_secs()),
                    shake_settings.random_sources[4].rand(time.elapsed_secs()),
                    shake_settings.random_sources[5].rand(time.elapsed_secs()),
                );

            let shake_rotation =
                Quat::from_euler(EulerRot::YXZ, rotation.x, rotation.y, rotation.z);
            transform.translation = shake_translation;
            transform.rotation = shake_rotation;
        } else {
            transform.translation = Vec3::default();
            transform.rotation = Quat::default();
        }
    }
}

fn apply_shake_2d(mut query: Query<(&mut Transform, &mut Shake2d)>, time: Res<Time>) {
    for (mut transform, mut shake_settings) in query.iter_mut() {
        shake_settings.trauma = f32::max(
            shake_settings.trauma - shake_settings.decay * time.delta_secs(),
            0.0,
        );

        let trauma_amount = f32::powf(shake_settings.trauma, shake_settings.trauma_power);
        if trauma_amount > 0.0 {
            let offset = shake_settings.max_offset
                * trauma_amount
                * Vec2::new(
                    shake_settings.random_sources[0].rand(time.elapsed_secs()),
                    shake_settings.random_sources[1].rand(time.elapsed_secs()),
                );

            let shake_translation = Vec3::new(offset.x, offset.y, 0.0);

            let shake_rotation = Quat::from_euler(
                EulerRot::YXZ,
                0.0,
                0.0,
                shake_settings.max_roll
                    * trauma_amount
                    * shake_settings.random_sources[2].rand(time.elapsed_secs()),
            );
            transform.translation = shake_translation;
            transform.rotation = shake_rotation;
        } else {
            transform.translation = Vec3::default();
            transform.rotation = Quat::default();
        }
    }
}

/// The CameraShakePlugin runs the systems required to shake something with a `Shake2d` or `Shake3d` component.
pub struct CameraShakePlugin;

impl Plugin for CameraShakePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, (apply_shake_2d, apply_shake_3d));
    }
}
