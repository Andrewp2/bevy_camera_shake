use bevy::{
    prelude::{
        Camera2d, Camera3d, Component, EulerRot, Plugin, Quat, Query, Res, Transform, Vec2, Vec3,
    },
    time::Time,
};

#[derive(Component)]
pub struct Camera3dShake {
    pub max_offset: Vec3,
    pub max_yaw_pitch_roll: Vec3,
    pub trauma: f32,
    pub trauma_power: f32,
    pub decay: f32,
    pub base_transform: Transform,
    pub shake_transform: Transform,
    pub random_source: fn(f32, f32) -> f32,
}

fn not_random(_seed: f32, _time: f32) -> f32 {
    0.5
}

impl Default for Camera3dShake {
    fn default() -> Self {
        Self {
            max_offset: Vec3::new(0.0, 0.0, 0.0),
            max_yaw_pitch_roll: Vec3::new(0.1, 0.1, 0.1),
            trauma: 0.0,
            trauma_power: 2.0,
            decay: 0.8,
            base_transform: Transform::default(),
            shake_transform: Transform::default(),
            random_source: not_random,
        }
    }
}

#[derive(Component)]
pub struct Camera2dShake {
    pub max_offset: Vec2,
    pub max_roll: f32,
    pub trauma: f32,
    pub trauma_power: f32,
    pub decay: f32,
    pub base_transform: Transform,
    pub shake_transform: Transform,
    pub random_source: fn(f32, f32) -> f32,
}

impl Default for Camera2dShake {
    fn default() -> Self {
        Self {
            max_offset: Vec2::new(100.0, 100.0),
            max_roll: 0.1,
            trauma: 0.0,
            trauma_power: 2.0,
            decay: 0.8,
            base_transform: Transform::default(),
            shake_transform: Transform::default(),
            random_source: not_random,
        }
    }
}

struct CameraShakePlugin;

impl Plugin for CameraShakePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(apply_shake_2d).add_system(apply_shake_3d);
    }
}

fn apply_shake_2d(
    mut query: Query<(&mut Transform, &mut Camera2dShake, &Camera2d)>,
    time: Res<Time>,
) {
    for (mut transform, mut shake_settings, _) in query.iter_mut() {
        shake_settings.trauma = f32::max(
            shake_settings.trauma - shake_settings.decay * time.delta_seconds(),
            0.0,
        );

        let trauma_amount = f32::powf(shake_settings.trauma, shake_settings.trauma_power);

        let offset = shake_settings.max_offset
            * trauma_amount
            * Vec2::new(
                (shake_settings.random_source)(0.0, time.delta_seconds()),
                (shake_settings.random_source)(1.0, time.delta_seconds()),
            );

        shake_settings.shake_transform.translation = Vec3::new(offset.x, offset.y, 0.0);

        shake_settings.shake_transform.rotation = Quat::from_euler(
            EulerRot::YXZ,
            0.0,
            0.0,
            shake_settings.max_roll
                * trauma_amount
                * (shake_settings.random_source)(2.0, time.delta_seconds()),
        );

        let t = transform.as_mut();
        t.translation =
            shake_settings.base_transform.translation + shake_settings.shake_transform.translation;
        t.rotation = shake_settings
            .shake_transform
            .rotation
            .mul_quat(shake_settings.base_transform.rotation);
    }
}

fn apply_shake_3d(
    mut query: Query<(&mut Transform, &mut Camera3dShake, &Camera3d)>,
    time: Res<Time>,
) {
    for (mut transform, mut shake_settings, _) in query.iter_mut() {
        shake_settings.trauma = f32::max(
            shake_settings.trauma - shake_settings.decay * time.delta_seconds(),
            0.0,
        );

        let trauma_amount = f32::powf(shake_settings.trauma, shake_settings.trauma_power);

        shake_settings.shake_transform.translation = shake_settings.max_offset
            * trauma_amount
            * Vec3::new(
                (shake_settings.random_source)(0.0, time.delta_seconds()),
                (shake_settings.random_source)(1.0, time.delta_seconds()),
                (shake_settings.random_source)(2.0, time.delta_seconds()),
            );

        let rotation = shake_settings.max_yaw_pitch_roll
            * trauma_amount
            * Vec3::new(
                (shake_settings.random_source)(3.0, time.delta_seconds()),
                (shake_settings.random_source)(4.0, time.delta_seconds()),
                (shake_settings.random_source)(5.0, time.delta_seconds()),
            );

        shake_settings.shake_transform.rotation =
            Quat::from_euler(EulerRot::YXZ, rotation.x, rotation.y, rotation.z);

        let t = transform.as_mut();
        t.translation =
            shake_settings.base_transform.translation + shake_settings.shake_transform.translation;
        t.rotation = shake_settings
            .shake_transform
            .rotation
            .mul_quat(shake_settings.base_transform.rotation);
    }
}
