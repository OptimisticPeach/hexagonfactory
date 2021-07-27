use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

use bevy::asset::LoadState;
use shaders::LowPolyPBRPlugin;
use sphereorder::{
    BoardInitializationType, FaceMaterialIdx, GeographicalParams, NeighbourOf, OldFaceMaterialIdx,
    PlanetDesc, SkyParams,
};
use bevy::ecs::component::{ComponentDescriptor, StorageType};
use sphereorder::camera::{SphereCamera, update_camera_transform, move_cameras, added_camera, CameraDebugPoint, DebugPoint, CameraSpeedConfig, LayerChangeEvent};
use sphereorder::board_ops::Layers;
// use bevy_inspector_egui::InspectorPlugin;

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
        .add_event::<LayerChangeEvent>()
        // .add_plugin(InspectorPlugin::<CameraSpeedConfig>::new())
        .insert_resource(CameraSpeedConfig::default())
        .add_plugin(sphereorder::BoardPlugin)
        // .add_startup_system(make_sparse_set.exclusive_system())
        .add_state(GameState::Load)
        // .add_system(debug_ram.exclusive_system())
        .add_system_set(SystemSet::on_enter(GameState::Load)
            .with_system(setup.system())
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
                .with_system(added_camera)
                // .with_system(rotate.system())
                .with_system(move_cameras.chain(update_camera_transform)),
        )
        .run();
}

struct RotationAxis(Vec3);

fn rotate(mut transforms: Query<(&mut Transform, &RotationAxis), With<Draw>>) {
    for (mut transform, axis) in transforms.iter_mut() {
        transform.rotate(Quat::from_axis_angle(axis.0, 0.005));
    }
}

// fn debug_ram(world: &mut World) {
//     world.debug_ram_usage();
// }

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
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let normal_map = asset_server.load::<Texture, _>("normal_map.png");
    commands.insert_resource(PendingRepeatTextures(vec![normal_map.clone()]));

    let ids = [
        commands
            .spawn()
            .insert(PlanetDesc {
                subvidisions: 13,
                planet_type: BoardInitializationType::Base(GeographicalParams { temp_seed: 1, metal_seed: 2 }),
            })
            .insert(RotationAxis(Vec3::X))
            .id(),

        commands
            .spawn()
            .insert(PlanetDesc {
                subvidisions: 18,
                planet_type: BoardInitializationType::Sky(SkyParams { land_seed: 3 }),
            })
            .insert(RotationAxis(Vec3::Y))
            .id(),

        commands
            .spawn()
            .insert(PlanetDesc {
                subvidisions: 23,
                planet_type: BoardInitializationType::Sky(SkyParams { land_seed: 4 }),
            })
            .insert(RotationAxis(Vec3::Z))
            .id(),

        commands
            .spawn()
            .insert(PlanetDesc {
                subvidisions: 28,
                planet_type: BoardInitializationType::Sky(SkyParams { land_seed: 5 }),
            })
            .insert(RotationAxis(Vec3::new((2.0_f32).sqrt().recip(), (2.0_f32).sqrt().recip(), 0.0)))
            .id(),

        commands
            .spawn()
            .insert(PlanetDesc {
                subvidisions: 33,
                planet_type: BoardInitializationType::Sky(SkyParams { land_seed: 6 }),
            })
            .insert(RotationAxis(Vec3::new((2.0_f32).sqrt().recip(), 0.0, (2.0_f32).sqrt().recip())))
            .id(),
    ];

    let parent_planet = commands
        .spawn()
        .insert_bundle(
            (
                Layers::from(&ids),
                Transform::default(),
                GlobalTransform::default(),
            )
        )
        .id();

    let debug_point = commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Icosphere { subdivisions: 5, radius: 1.0 })),
        material: materials.add(Color::rgb(1.0, 0.0, 1.0).into()),
        transform: Transform::from_scale(Vec3::splat(0.075)),
        ..Default::default()
    })
        .insert(CameraDebugPoint)
        .id();

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
        .insert(SphereCamera::new(parent_planet))
        .insert(DebugPoint(debug_point));
}
