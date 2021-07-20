use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

use arrayvec::ArrayVec;
use bevy::asset::LoadState;
use bevy::render::mesh::Indices;
use rand::{thread_rng, Rng};
use shaders::{LowPolyMaterial, LowPolyPBRBundle, LowPolyPBRPlugin};
use sphereorder::{
    BoardInitializationType, FaceMaterialIdx, GeographicalParams, NeighbourOf, OldFaceMaterialIdx,
    PlanetDesc, SkyParams,
};
use bevy::ecs::component::{ComponentDescriptor, StorageType};
use sphereorder::camera::{SphereCamera, update_camera_transform, move_cameras, added_camera};

// mod geometry;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum GameState {
    Load,
    Game,
}

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(LowPolyPBRPlugin)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(sphereorder::BoardPlugin)
        // .add_startup_system(make_sparse_set.exclusive_system())
        .add_state(GameState::Load)
        // .add_system(debug_ram.exclusive_system())
        .add_system_set(SystemSet::on_enter(GameState::Load)
            .with_system(setup.system())
            .with_system(added_camera)
        )
        .add_system_set(
            SystemSet::on_update(GameState::Load)
                .with_system(poll_repeating_textures_load.system()),
        )
        .add_system_set(
            SystemSet::on_enter(GameState::Game),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Game)
                .with_system(rotate.system())
                .with_system(move_cameras.system().chain(update_camera_transform.system())),
        )
        .run();
}

struct RotationAxis(Vec3);

fn rotate(mut transforms: Query<(&mut Transform, &RotationAxis), With<Draw>>) {
    for (mut transform, axis) in transforms.iter_mut() {
        transform.rotate(Quat::from_axis_angle(axis.0, 0.005));
    }
}

fn debug_ram(world: &mut World) {
    world.debug_ram_usage();
}

struct PendingRepeatTextures(Vec<Handle<Texture>>);

fn poll_repeating_textures_load(
    mut loading: ResMut<PendingRepeatTextures>,
    server: Res<AssetServer>,
    mut textures: ResMut<Assets<Texture>>,
    mut state: ResMut<State<GameState>>,
) {
    loading
        .0
        .retain(|x| match server.get_load_state(x.clone()) {
            LoadState::Loaded => {
                let texture = textures.get_mut(x.clone()).unwrap();
                texture.reinterpret_stacked_2d_as_array(6);
                false
            }
            LoadState::Failed => panic!(),
            LoadState::Loading | LoadState::NotLoaded => true,
            LoadState::Unloaded => panic!(),
        });

    if loading.0.is_empty() {
        state.set(GameState::Game).unwrap();
    }
}

/// set up a simple 3D scene
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let normal_map = asset_server.load::<Texture, _>("normal_map.png");
    commands.insert_resource(PendingRepeatTextures(vec![normal_map.clone()]));

    let id = commands
        .spawn()
        .insert(PlanetDesc {
            subvidisions: 8,
            planet_type: BoardInitializationType::Base(GeographicalParams { temp_seed: 1, metal_seed: 2 }),
        })
        .insert(RotationAxis(Vec3::X))
        .id();

    commands
        .spawn()
        .insert(PlanetDesc {
            subvidisions: 11,
            planet_type: BoardInitializationType::Sky(SkyParams { land_seed: 3 }),
        })
        .insert(RotationAxis(Vec3::Y));

    commands
        .spawn()
        .insert(PlanetDesc {
            subvidisions: 14,
            planet_type: BoardInitializationType::Sky(SkyParams { land_seed: 4 }),
        })
        .insert(RotationAxis(Vec3::Z));

    commands
        .spawn()
        .insert(PlanetDesc {
            subvidisions: 17,
            planet_type: BoardInitializationType::Sky(SkyParams { land_seed: 5 }),
        })
        .insert(RotationAxis(Vec3::new((2.0_f32).sqrt().recip(), (2.0_f32).sqrt().recip(), 0.0)));

    commands
        .spawn()
        .insert(PlanetDesc {
            subvidisions: 20,
            planet_type: BoardInitializationType::Sky(SkyParams { land_seed: 6 }),
        })
        .insert(RotationAxis(Vec3::new((2.0_f32).sqrt().recip(), 0.0, (2.0_f32).sqrt().recip())));

    // light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(40.0, 80.0, 40.0),
        point_light: PointLight {
            intensity: 20000.0,
            range: 200.0,
            radius: 0.0,
            ..Default::default()
        },
        ..Default::default()
    });

    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-20.0, 25.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    })
        .insert(SphereCamera::new(&[id]));
}
