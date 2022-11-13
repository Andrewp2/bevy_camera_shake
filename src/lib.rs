use bevy::{
    prelude::{warn, Component, EulerRot, Plugin, Quat, Query, Res, Transform, Vec2, Vec3},
    time::Time,
};

// A source of randomness for shaking the camera.
pub trait RandomSource: Send + Sync {
    // Produces a random float between -1.0 and 1.0.
    fn rand(&self, time: f32) -> f32;
}

struct NotRandom;

impl RandomSource for NotRandom {
    fn rand(&self, _time: f32) -> f32 {
        warn!("You probably need to set a random source for the shaking to work properly!!!");
        0.5
    }
}

#[derive(Component)]
pub struct Shake3d {
    pub max_offset: Vec3,
    pub max_yaw_pitch_roll: Vec3,
    pub trauma: f32,
    pub trauma_power: f32,
    pub decay: f32,
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

#[derive(Component)]
pub struct Shake2d {
    pub max_offset: Vec2,
    pub max_roll: f32,
    pub trauma: f32,
    pub trauma_power: f32,
    pub decay: f32,
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
            shake_settings.trauma - shake_settings.decay * time.delta_seconds(),
            0.0,
        );

        let trauma_amount = f32::powf(shake_settings.trauma, shake_settings.trauma_power);

        if trauma_amount > 0.0 {
            let shake_translation = shake_settings.max_offset
                * trauma_amount
                * Vec3::new(
                    shake_settings.random_sources[0].rand(time.elapsed_seconds()),
                    shake_settings.random_sources[1].rand(time.elapsed_seconds()),
                    shake_settings.random_sources[2].rand(time.elapsed_seconds()),
                );

            let rotation = shake_settings.max_yaw_pitch_roll
                * trauma_amount
                * Vec3::new(
                    shake_settings.random_sources[3].rand(time.elapsed_seconds()),
                    shake_settings.random_sources[4].rand(time.elapsed_seconds()),
                    shake_settings.random_sources[5].rand(time.elapsed_seconds()),
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
            shake_settings.trauma - shake_settings.decay * time.delta_seconds(),
            0.0,
        );

        let trauma_amount = f32::powf(shake_settings.trauma, shake_settings.trauma_power);
        if trauma_amount > 0.0 {
            let offset = shake_settings.max_offset
                * trauma_amount
                * Vec2::new(
                    shake_settings.random_sources[0].rand(time.elapsed_seconds()),
                    shake_settings.random_sources[1].rand(time.elapsed_seconds()),
                );

            let shake_translation = Vec3::new(offset.x, offset.y, 0.0);

            let shake_rotation = Quat::from_euler(
                EulerRot::YXZ,
                0.0,
                0.0,
                shake_settings.max_roll
                    * trauma_amount
                    * shake_settings.random_sources[2].rand(time.elapsed_seconds()),
            );
            transform.translation = shake_translation;
            transform.rotation = shake_rotation;
        } else {
            transform.translation = Vec3::default();
            transform.rotation = Quat::default();
        }
    }
}

pub struct CameraShakePlugin;

impl Plugin for CameraShakePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(apply_shake_2d).add_system(apply_shake_3d);
    }
}
