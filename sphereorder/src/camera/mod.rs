use smallvec::SmallVec;
use bevy::ecs::entity::Entity;
use bevy::math::{Vec3, Quat, Vec2};
use bevy::ecs::query::{Without, Added, Changed};
use bevy::transform::components::Transform;
use bevy::ecs::system::{Query, Res, Commands};
use bevy::input::Input;
use bevy::input::mouse::{MouseButton, MouseMotion, MouseWheel};
use bevy::ecs::event::EventReader;
use bevy::ecs::prelude::With;
use bevy_utils::Instant;
use bevy::prelude::KeyCode;
use bevy::app::EventWriter;
use crate::board_ops::Layers;
// use bevy_inspector_egui::Inspectable;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct LayerChangeEvent {
    parent_planet: Entity,
    old_shell: Entity,
    old: usize,
    new_shell: Entity,
    new: usize,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct CameraDebugPoint;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct DebugPoint(pub Entity);

// #[derive(Copy, Clone, Debug, PartialEq, Inspectable)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CameraSpeedConfig {
    // #[inspectable(min = 0.0, max = 0.2)]
    pub vertical_max: f32,
    // #[inspectable(min = 0.0, max = 0.2)]
    pub lateral_max: f32,
    // #[inspectable(min = 3.0, max = 4.0)]
    pub logarithm_base: f32,
    // #[inspectable(min = 150.0, max = 4000.0)]
    pub total_scale: f32,
    pub scale_per_second: f32,
}

impl Default for CameraSpeedConfig {
    fn default() -> Self {
        CameraSpeedConfig {
            vertical_max: 0.16,
            lateral_max: 0.16,
            logarithm_base: 3.3,
            total_scale: 1000.0,
            scale_per_second: 4.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SphereCamera {
    vector_of_interest: Quat,
    from_target_angle: (f32, f32),
    layered_planet: Entity,
    pivot_speed: Option<(f32, f32)>,
    target_layer: usize,
    scale: f32,
    distance: f32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TargetSphereCamera {
    original_scale: f32,
    original_distance: f32,
    original_layer: usize,
    target_scale: f32,
    target_distance: f32,
    target_layer: usize,
    start: Instant,
    // Don't store scale_per_second, because it's always constant.
    distance_per_second: f32,
}

impl SphereCamera {
    pub fn new(layered: Entity) -> Self {
        SphereCamera {
            vector_of_interest: Quat::IDENTITY,
            from_target_angle: (0.001, 0.001),
            layered_planet: layered,
            target_layer: 0,
            scale: 10.0,
            distance: 0.1,
            pivot_speed: None,
        }
    }
}

pub fn move_cameras(
    mut cameras: Query<(&mut SphereCamera, &mut Option<TargetSphereCamera>)>,
    planet_layers: Query<&Layers>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    speed_config: Res<CameraSpeedConfig>,
    keyboard: Res<Input<KeyCode>>,
    transforms: Query<&Transform>,
    mut events: EventWriter<LayerChangeEvent>,
) {
    // Mouse motion total
    let total = mouse_motion_events
        .iter()
        .map(|x| x.delta)
        .fold(Vec2::ZERO, |acc, x| acc + x)
        / speed_config.total_scale;

    let (mut camera, mut camera_target) = if let Ok(x) = cameras.single_mut() {
        x
    } else {
        return;
    };

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

        if let Some(target) = &mut *camera_target {
            target.target_distance += (-total) * target.target_distance.sqrt() * 0.3;
            target.target_distance = target.target_distance.max(0.01);

            target.original_distance += (-total) * target.original_distance.sqrt() * 0.3;
            target.original_distance = target.original_distance.max(0.01);
        }
    }

    if let Some(target) = &mut *camera_target {
        let time = (Instant::now() - target.start).as_secs_f32();
        let delta_scale = target.target_scale - target.original_scale;
        let total_time = delta_scale.abs() / speed_config.scale_per_second;

        if time > total_time {
            camera.scale = target.target_scale;
            camera.distance = target.target_distance;
            camera.target_layer = target.target_layer;
            *camera_target = None;
        } else {
            let add_scale = time * speed_config.scale_per_second * delta_scale.signum();
            println!("Add scale: {}", add_scale);
            let add_dist = target.distance_per_second * time;

            camera.scale = target.original_scale + add_scale;
            camera.distance = target.original_distance + add_dist;
        }
    }

    let layers_changed = keyboard.just_pressed(KeyCode::W) as isize -
            keyboard.just_pressed(KeyCode::S) as isize;

    let effective_layer = camera_target
        .map(|x| x.target_layer)
        .unwrap_or_else(|| camera.target_layer);

    if layers_changed == 0 || (layers_changed == -1 && effective_layer == 0) {
        return;
    }

    let new_target = (effective_layer as isize + layers_changed) as usize;

    let layers = planet_layers.get(camera.layered_planet).unwrap();

    if new_target == layers.len() {
        return;
    }

    let new_max = layers
        .get(new_target + 1)
        .map(|&x| transforms.get(x).unwrap().scale.x - 0.1)
        .unwrap_or(f32::INFINITY);

    let new_scl = transforms.get(layers[new_target]).unwrap().scale.x;

    let new_dst = new_max - new_scl;

    match &mut *camera_target {
        Some(old) => {
            let old_target_layer = old.target_layer;
            println!("Migrating target change: Current: {}, Old: {}, New: {}", camera.target_layer, old_target_layer, new_target);

            *old = TargetSphereCamera {
                original_scale: camera.scale,
                original_distance: camera.distance,
                original_layer: old_target_layer,
                target_scale: new_scl,
                target_distance: new_dst.min(camera.distance),
                target_layer: new_target,
                start: Instant::now(),
                distance_per_second: 0.0
            };

            let delta_scale = old.target_scale - old.original_scale;
            let total_time = delta_scale / speed_config.scale_per_second;

            old.distance_per_second = (old.target_distance - old.original_distance) / total_time.abs();
            events
                .send(LayerChangeEvent {
                    parent_planet: camera.layered_planet,
                    old_shell: layers[old.original_layer],
                    old: old.original_layer,
                    new_shell: layers[new_target],
                    new: new_target,
                });
        },
        None => {
            let mut new = TargetSphereCamera {
                original_scale: camera.scale,
                original_distance: camera.distance,
                original_layer: camera.target_layer,
                target_scale: new_scl,
                target_distance: new_dst.min(camera.distance),
                target_layer: new_target,
                start: Instant::now(),
                distance_per_second: 0.0
            };

            let delta_scale = new.target_scale - new.original_scale;
            let total_time = delta_scale / speed_config.scale_per_second;

            new.distance_per_second = (new.target_distance - new.original_distance) / total_time.abs();
            *camera_target = Some(new);

            events
                .send(LayerChangeEvent {
                    parent_planet: camera.layered_planet,
                    old_shell: layers[camera.target_layer],
                    old: camera.target_layer,
                    new_shell: layers[new_target],
                    new: new_target,
                });
        }
    }
}

pub fn update_camera_transform(
    mut cameras: Query<(&mut Transform, &SphereCamera, &DebugPoint), (Without<CameraDebugPoint>, Changed<SphereCamera>)>,
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

        let planet_delta = transforms.get(camera.layered_planet).unwrap().translation;
        
        *transform = Transform::from_translation(
            planet_delta + target_vector * camera.scale + camera_delta * camera.distance
        ).looking_at(planet_delta + target_vector * camera.scale, up);
    }
}

pub fn added_camera(
    mut cameras: Query<(Entity, &mut SphereCamera), Added<SphereCamera>>,
    layers: Query<&Layers>,
    transforms: Query<&Transform>,
    mut commands: Commands,
) {
    cameras
        .iter_mut()
        .for_each(move |(entity, mut x)| {
            x.scale = transforms.get(layers.get(x.layered_planet).unwrap()[x.target_layer]).unwrap().scale.x;
            commands
                .entity(entity)
                .insert(None::<TargetSphereCamera>);
        });
}
