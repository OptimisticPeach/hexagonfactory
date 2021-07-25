use smallvec::SmallVec;
use bevy::ecs::entity::Entity;
use bevy::math::{Vec3, Quat, Vec2};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::TypeInfo;
use bevy::ecs::query::{Changed, Without, Added};
use bevy::transform::components::Transform;
use bevy::ecs::system::{Query, Res, ResMut};
use bevy::input::Input;
use bevy::input::mouse::{MouseButton, MouseMotion, MouseWheel};
use bevy::ecs::event::EventReader;
use bevy::ecs::prelude::With;
use bevy_inspector_egui::Inspectable;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct CameraDebugPoint;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct DebugPoint(pub Entity);

#[derive(Copy, Clone, Debug, PartialEq, Inspectable)]
pub struct CameraSpeedConfig {
    #[inspectable(min = 0.0, max = 0.2)]
    pub vertical_max: f32,
    #[inspectable(min = 0.0, max = 0.2)]
    pub lateral_max: f32,
    #[inspectable(min = 3.0, max = 4.0)]
    pub logarithm_base: f32,
    #[inspectable(min = 150.0, max = 4000.0)]
    pub total_scale: f32,
}

impl Default for CameraSpeedConfig {
    fn default() -> Self {
        CameraSpeedConfig {
            vertical_max: 0.16,
            lateral_max: 0.16,
            logarithm_base: 3.3,
            total_scale: 1000.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SphereCamera {
    vector_of_interest: Quat,
    from_target_angle: (f32, f32),
    layers: SmallVec<[Entity; 5]>,
    pivot_speed: Option<(f32, f32)>,
    target_layer: usize,
    scale: f32,
    distance: f32,
    target_scale: Option<f32>,
}

impl SphereCamera {
    pub fn new(entities: &[Entity]) -> Self {
        assert!(entities.len() > 0);
        SphereCamera {
            vector_of_interest: Quat::IDENTITY,
            from_target_angle: (0.001, 0.001),
            layers: entities.iter().copied().collect(),
            target_layer: 0,
            scale: 10.0,
            distance: 0.1,
            pivot_speed: None,
            target_scale: None,
        }
    }
}

pub fn move_cameras(
    mut cameras: Query<&mut SphereCamera>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    speed_config: Res<CameraSpeedConfig>
) {
    // Mouse motion total
    let total = mouse_motion_events
        .iter()
        .map(|x| x.delta)
        .fold(Vec2::ZERO, |acc, x| acc + x)
        / speed_config.total_scale;

    let mut camera = cameras.iter_mut().next().unwrap();

    if mouse_button_input.pressed(MouseButton::Left) {
        let pivot_velocity = camera.pivot_speed.get_or_insert((0.0, 0.0));
        pivot_velocity.0 = (pivot_velocity.0 + total.x).clamp(-speed_config.lateral_max, speed_config.lateral_max);
        pivot_velocity.1 = (pivot_velocity.1 + total.y).clamp(-speed_config.vertical_max, speed_config.vertical_max);

        pivot_velocity.0 = (pivot_velocity.0.abs() + 1.0).log(speed_config.logarithm_base) * pivot_velocity.0.signum();
        pivot_velocity.1 = (pivot_velocity.1.abs() + 1.0).log(speed_config.logarithm_base) * pivot_velocity.1.signum();

        let pivot = *pivot_velocity;

        camera.from_target_angle.0 = (camera.from_target_angle.0 - pivot.0).rem_euclid(std::f32::consts::TAU);
        camera.from_target_angle.1 = (camera.from_target_angle.1 + pivot.1).clamp(0.01, std::f32::consts::FRAC_PI_2);
    } else {
        camera.pivot_speed = None;
        if mouse_button_input.pressed(MouseButton::Right) {
            let quat = Quat::from_rotation_y(camera.from_target_angle.0) *
                Quat::from_rotation_z(camera.from_target_angle.1);

            let x = quat.mul_vec3(Vec3::X);
            let z = quat.mul_vec3(Vec3::Z);

            camera.vector_of_interest = camera.vector_of_interest * Quat::from_axis_angle(x, -total.x) * Quat::from_axis_angle(z, -total.y);
        }
    }


    // Scroll wheel total
    let total = mouse_wheel_events
        .iter()
        .map(|event| event.y)
        .fold(0.0f32, |acc, x| acc + x);

    if total != 0.0 {
        camera.distance += (-total) * camera.distance.sqrt() * 0.3;
        camera.distance = camera.distance.max(0.01);
    }
}

pub fn update_camera_transform(
    mut cameras: Query<(&mut Transform, &SphereCamera, &DebugPoint), (Changed<SphereCamera>, Without<CameraDebugPoint>)>,
    mut point_transforms: Query<&mut Transform, (Without<SphereCamera>, With<CameraDebugPoint>)>,
    transforms: Query<&Transform, (Without<SphereCamera>, Without<CameraDebugPoint>)>,
) {
    for (mut transform, camera, &DebugPoint(debug_point)) in cameras.iter_mut() {
        let transform: &mut Transform = &mut *transform;
        let camera: &SphereCamera = &*camera;
        let target_vector = camera.vector_of_interest.mul_vec3(Vec3::Y);
        point_transforms.get_mut(debug_point).unwrap().translation = target_vector * camera.scale;
        let camera_delta = (
            camera.vector_of_interest *
                Quat::from_rotation_y(camera.from_target_angle.0) *
                Quat::from_rotation_z(camera.from_target_angle.1)
        ).mul_vec3(Vec3::Y);
        let up = (-camera_delta).cross(target_vector).cross(-camera_delta).normalize();

        let planet_delta = transforms.get(camera.layers[camera.target_layer]).unwrap().translation;
        
        *transform = Transform::from_translation(
            planet_delta + target_vector * camera.scale + camera_delta * camera.distance
        ).looking_at(planet_delta + target_vector * camera.scale, up);
    }
}

pub fn added_camera(
    mut cameras: Query<&mut SphereCamera, Added<SphereCamera>>,
    transforms: Query<&Transform>,
) {
    cameras
        .iter_mut()
        .for_each(|mut x| {
            x.scale = transforms.get(x.layers[x.target_layer]).unwrap().scale.x;
        });
}
