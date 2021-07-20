use smallvec::SmallVec;
use bevy::ecs::entity::Entity;
use bevy::math::{Vec3, Quat, Vec2};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::TypeInfo;
use bevy::ecs::query::{Changed, Without, Added};
use bevy::transform::components::Transform;
use bevy::ecs::system::{Query, Res};
use bevy::input::Input;
use bevy::input::mouse::{MouseButton, MouseMotion, MouseWheel};
use bevy::ecs::event::EventReader;

#[derive(Clone, Debug, PartialEq)]
pub struct SphereCamera {
    vector_of_interest: Quat,
    from_target_angle: (f32, f32),
    layers: SmallVec<[Entity; 5]>,
    target_speed: Option<f32>,
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
            target_speed: None,
            target_scale: None,
        }
    }
}

pub fn move_cameras(
    mut cameras: Query<&mut SphereCamera>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
) {
    let total = mouse_motion_events
        .iter()
        .map(|x| x.delta)
        .fold(Vec2::ZERO, |acc, x| acc + x)
        / 150.0;

    let mut camera = cameras.iter_mut().next().unwrap();

    if mouse_button_input.pressed(MouseButton::Left) {
        camera.from_target_angle.0 = (camera.from_target_angle.0 - total.x).rem_euclid(std::f32::consts::TAU);
        camera.from_target_angle.1 = (camera.from_target_angle.1 + total.y).clamp(0.01, std::f32::consts::FRAC_PI_2);
    } else if mouse_button_input.pressed(MouseButton::Right) {
        let quat = Quat::from_rotation_y(camera.from_target_angle.0) *
            Quat::from_rotation_z(camera.from_target_angle.1);

        let x = quat.mul_vec3(Vec3::X);
        let z = quat.mul_vec3(Vec3::Z);

        camera.vector_of_interest = camera.vector_of_interest * Quat::from_axis_angle(x, -total.x) * Quat::from_axis_angle(z, -total.y);
    }

    let total = mouse_wheel_events
        .iter()
        .map(|event| event.y)
        .fold(0.0f32, |acc, x| acc + x);

    if total != 0.0 {
        camera.distance += (-total) * camera.distance.sqrt() * 0.3;
        camera.distance = camera.distance.max(0.1);
    }
}

pub fn update_camera_transform(
    mut cameras: Query<(&mut Transform, &SphereCamera), Changed<SphereCamera>>,
    transforms: Query<&Transform, Without<SphereCamera>>,
) {
    for (mut transform, camera) in cameras.iter_mut() {
        let transform: &mut Transform = &mut *transform;
        let camera: &SphereCamera = &*camera;
        let target_vector = camera.vector_of_interest.mul_vec3(Vec3::Y);
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
