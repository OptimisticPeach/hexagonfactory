use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

use bevy::asset::LoadState;
use bevy::render::mesh::Indices;
use shaders::{LowPolyMaterial, LowPolyPBRBundle, LowPolyPBRPlugin};
use rand::{thread_rng, Rng};
use sphereorder::{BoardMember, FaceMaterialIdx, OldFaceMaterialIdx};
use arrayvec::ArrayVec;

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
        .add_state(GameState::Load)
        .add_system_set(SystemSet::on_enter(GameState::Load).with_system(setup.system()))
        .add_system_set(
            SystemSet::on_update(GameState::Load)
                .with_system(poll_repeating_textures_load.system()),
        )
        .add_system_set(SystemSet::on_enter(GameState::Game)
            // .with_system(init_crawler.system())
            // .with_system(test_all.system())
        )
        .add_system_set(SystemSet::on_update(GameState::Game)
            .with_system(rotate.system())
            // .with_system(crawl.system())
        )
        .run();
}

fn rotate(mut transforms: Query<(&mut Transform, With<Draw>)>) {
    for (mut transform, _) in transforms.iter_mut() {
        transform.rotate(Quat::from_rotation_x(0.005));
    }
}

struct PendingRepeatTextures(Vec<Handle<Texture>>);

fn poll_repeating_textures_load(
    mut loading: ResMut<PendingRepeatTextures>,
    server: Res<AssetServer>,
    mut textures: ResMut<Assets<Texture>>,
    mut state: ResMut<State<GameState>>,
) {
    loading.0.retain(|x| {
        match server.get_load_state(x.clone()) {
            LoadState::Loaded => {
                let texture = textures.get_mut(x.clone()).unwrap();
                texture.reinterpret_stacked_2d_as_array(6);
                false
            }
            LoadState::Failed => panic!(),
            LoadState::Loading | LoadState::NotLoaded => true,
            LoadState::Unloaded => panic!(),
        }
    });

    if loading.0.is_empty() {
        state.set(GameState::Game).unwrap();
    }
}

struct CrawlerState {
    entity: Entity,
}

fn test_all(
    mut member: Query<(&mut FaceMaterialIdx, &BoardMember)>,
) {
    member
        .iter_mut()
        .for_each(|(mut x, member)| x.0 = 25);
}

fn init_crawler(
    mut crawler: ResMut<CrawlerState>,
    children: Query<&Children>,
    mut member: Query<&mut FaceMaterialIdx>,
) {
    let mut rng = thread_rng();
    let children = children.get(crawler.entity).unwrap();
    crawler.entity = children[rng.gen_range(0..children.len())];

    let mut member = member
        .get_mut(crawler.entity)
        .unwrap();

    member.0 = 25;
}

fn crawl(
    mut crawler: ResMut<CrawlerState>,
    member: Query<&BoardMember>,
    mut faces: Query<(&OldFaceMaterialIdx, &mut FaceMaterialIdx)>,
) {
    // faces
    //     .get_mut(crawler.entity)
    //     .map(|(old, mut new)| new.0 = old.0)
    //     .unwrap();

    let member = member
        .get(crawler.entity)
        .unwrap();

    let next = member
        .board
        .total_graph
        .neighbors(
            member
                .graph_idx
        )
        .collect::<ArrayVec<_, 6>>();

    let next = next[thread_rng().gen_range(0..next.len())];

    let (chunk, per_chunk) = sphereorder::idx_to_chunk(
        *member
            .board
            .total_graph
            .node_weight(next)
            .unwrap(),
        member
            .board
            .per_chunk,
    );

    let entity = member
            .board
            .tile_data[chunk]
            .read()[per_chunk]
            .layers
            .base;

    crawler.entity = entity;
    faces
        .get_mut(entity)
        .map(|(_old, mut new)| new.0 = 25)
        .unwrap();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    mut planet_materials: ResMut<Assets<LowPolyMaterial>>,
) {
    let normal_map = asset_server.load::<Texture, _>("normal_map.png");
    commands.insert_resource(PendingRepeatTextures(vec![normal_map.clone()]));

    let start = std::time::Instant::now();
    let (mesh, mut per_face_data, planet) = sphereorder::BoardGraph::build(8).create(&mut commands);
    let time = start.elapsed();
    println!("time: {:?}", time);
    // println!("{:#?}", per_face_data);

    per_face_data.push(
        shaders::PerFaceData {
            colour: bevy::render::color::Color::GOLD.as_linear_rgba_f32(),
            roughness: 0.5,
            metallic: 1.0,
            reflectance: 1.0,
            ..Default::default()
        }
    );
    
    println!(
        "{}, {}, {}",
        per_face_data.len(),
        mesh.indices()
            .map(|x| match x {
                Indices::U32(x) => x.len(),
                Indices::U16(x) => x.len(),
            })
            .unwrap(),
        mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().len()
    );

    let mesh_handle = meshes.add(mesh);

    commands
        .insert_resource(CrawlerState {
            entity: planet,
        });

    commands
        .entity(planet)
        .insert_bundle(LowPolyPBRBundle {
        mesh: mesh_handle.clone(),
        material: planet_materials.add(LowPolyMaterial {
            base_color_texture: None,
            metallic_roughness_texture: None,
            normal_map: Some(normal_map),
            per_face_data,
            ..Default::default()
        }),
        transform: {
            let mut t = Transform::from_xyz(0.0, 0.0, 0.0);
            t.apply_non_uniform_scale(Vec3::splat(2.0));
            t.rotate(Quat::from_rotation_y(-0.3));
            t.rotate(Quat::from_rotation_x(0.4));
            t
        },
        ..Default::default()
    });

    // light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        // transform: Transform::from_xyz(-2.0, 2.5, 5.0),
        ..Default::default()
    });

    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}
