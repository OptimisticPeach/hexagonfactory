use arrayvec::ArrayVec;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::Commands;
use bevy::math::{Mat4, Quat, Vec3A, Vec3};
use bevy::render::mesh::{Indices, Mesh};
use bevy::render::pipeline::PrimitiveTopology;
use bevy::utils::{HashMap, HashSet};
use hexasphere::shapes::IcoSphere;
use shaders::{PerFaceData, LowPolyMaterial, LowPolyPBRBundle};

use bevy::transform::components::{GlobalTransform, Transform};

use rand::Rng;

mod biome;
pub mod board_ops;
pub mod camera;

use bevy::prelude::BuildChildren;
pub use biome::Biome;
pub use board_ops::BoardPlugin;
use std::ops::Range;
use bevy::asset::Assets;
use std::hash::{Hash, Hasher};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct NeighbourOf;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct PackedRelations {
    relations: HashMap<RelationPair, NeighbourOf>
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RelationPair(pub Entity, pub Entity);

impl Hash for RelationPair {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.0 > self.1 {
            (self.0, self.1).hash(state);
        } else {
            (self.1, self.0).hash(state);
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PlanetDesc {
    pub subvidisions: usize,
    pub planet_type: BoardInitializationType,
}

pub(crate) struct TileDataIdx(usize);

pub struct TileData {
    pub biome: Biome,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FaceMaterialIdx(pub i32);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct OldFaceMaterialIdx(pub i32);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct GeographicalParams {
    pub metal_seed: i32,
    pub temp_seed: i32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SkyParams {
    pub land_seed: i32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BoardInitializationType {
    Empty,
    Base(GeographicalParams),
    Sky(SkyParams),
    Space(SkyParams),
}

impl BoardInitializationType {
    pub(crate) fn make_biomes(
        &self,
        mid_points: &[Vec3A],
    ) -> (
        Vec<Biome>,
        &'static HashMap<Biome, Range<i32>>,
        Vec<PerFaceData>,
    ) {
        match self {
            &BoardInitializationType::Base(GeographicalParams {
                metal_seed,
                temp_seed,
            }) => {
                let tile_datas = noise_gen::sample_all_noise(
                    mid_points,
                    [
                        // Whether it's dirt or metal
                        noise_gen::NoiseParameters {
                            scale: 0.5,
                            lac: 0.1,
                            gain: 0.9,
                            octaves: 4,
                            min: 0.0,
                            max: 1.0,
                            seed: metal_seed,
                        },
                        // Whether it's terrain or lava
                        noise_gen::NoiseParameters {
                            scale: 1.0,
                            lac: 1.0,
                            gain: 1.0,
                            octaves: 4,
                            min: -1.0,
                            max: 1.0,
                            seed: temp_seed,
                        },
                    ],
                );
                (
                    crate::biome::make_base_biomes(&tile_datas),
                    &*crate::biome::BASE_BIOME_MAP,
                    crate::biome::BASE_BIOME_COLOURS.to_vec(),
                )
            },
            &BoardInitializationType::Sky(SkyParams { land_seed }) => {
                let tile_datas = noise_gen::sample_all_noise(
                    mid_points,
                    [noise_gen::NoiseParameters {
                        scale: 2.0,
                        lac: 0.21,
                        gain: 0.0,
                        octaves: 2,
                        min: -2.0,
                        max: 1.0,
                        seed: land_seed,
                    }],
                );
                (
                    crate::biome::make_sky_biomes(&tile_datas),
                    &*crate::biome::SKY_BIOME_MAP,
                    crate::biome::SKY_BIOME_COLOURS.to_vec(),
                )
            },
            &BoardInitializationType::Space(SkyParams { land_seed }) => {
                let tile_datas = noise_gen::sample_all_noise(
                    mid_points,
                    [
                        // Whether it's dirt or metal
                        noise_gen::NoiseParameters {
                            scale: 8.0,
                            lac: 0.2,
                            gain: 0.9,
                            octaves: 4,
                            min: -1.0,
                            max: 1.0,
                            seed: land_seed,
                        },
                    ],
                );
                (
                    crate::biome::make_space_biomes(&tile_datas),
                    &*crate::biome::SPACE_BIOME_MAP,
                    crate::biome::SPACE_BIOME_COLOURS.to_vec(),
                )
            },
            BoardInitializationType::Empty => (
                mid_points.iter().map(|_| Biome::Empty).collect(),
                &*crate::biome::SPACE_BIOME_MAP,
                crate::biome::SPACE_BIOME_COLOURS.to_vec(),
            ),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BoardBuilder {
    pub subdivisions: usize,
    pub state: BoardInitializationType,
}

impl From<PlanetDesc> for BoardBuilder {
    fn from(x: PlanetDesc) -> Self {
        Self {
            subdivisions: x.subvidisions,
            state: x.planet_type,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Hash)]
enum Index {
    All(u32),
    Mid(u32),
}

impl Index {
    pub fn resolve(self, mid: u32) -> u32 {
        match self {
            Self::All(x) => x,
            Self::Mid(x) => x + mid,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct SurroundingEntry {
    edge: [u32; 2],
    avg_idx: u32,
}

fn add_chunk_triangle_indices(
    a: u32,
    b: u32,
    c: u32,
    unordered_edges: &mut HashMap<(u32, u32), NeighbourOf>,
    original_points: &[Vec3A],
    new_points: &mut Vec<Vec3A>,
    per_face_indices: &mut Vec<i32>,
    surrounding_points: &mut HashMap<u32, ArrayVec<SurroundingEntry, 6>>,
) {
    unordered_edges.extend(std::array::IntoIter::new([
        ((a.min(b), a.max(b)), NeighbourOf),
        ((b.min(c), b.max(c)), NeighbourOf),
        ((c.min(a), c.max(a)), NeighbourOf),
    ]));

    let avg =
        (original_points[a as usize] + original_points[b as usize] + original_points[c as usize])
            / 3.0;
    let avg_idx = new_points.len() as u32;
    new_points.push(avg);
    per_face_indices.push(0);

    surrounding_points
        .entry(a)
        .or_insert_with(ArrayVec::new)
        .push(SurroundingEntry {
            edge: [b, c],
            avg_idx,
        });

    surrounding_points
        .entry(b)
        .or_insert_with(ArrayVec::new)
        .push(SurroundingEntry {
            edge: [c, a],
            avg_idx,
        });

    surrounding_points
        .entry(c)
        .or_insert_with(ArrayVec::new)
        .push(SurroundingEntry {
            edge: [a, b],
            avg_idx,
        });
}

fn make_point_transform(normalized_point: Vec3A) -> Transform {
    // normalized_point is the new "y"

    let x = (Quat::from_rotation_y(0.1) * normalized_point).normalize();
    let z = normalized_point.cross(x).normalize();
    let x = normalized_point.cross(z).normalize();

    Transform::from_matrix(Mat4::from_cols(
        x.extend(0.0),
        normalized_point.extend(0.0),
        z.extend(0.0),
        normalized_point.extend(1.0),
    ))
}

impl BoardBuilder {
    pub fn create_on(
        &self,
        commands: &mut Commands,
        board: Entity,
        meshes: &mut Assets<Mesh>,
        planet_materials: &mut Assets<LowPolyMaterial>,
    ) {
        let sphere = IcoSphere::new(self.subdivisions, |_| ());
        let original_points = sphere.raw_points();
        // Keep the middle points and the between-points separate
        // since we only need most of the tile parameters for the
        // middle points. All points need a
        // Every point other than the middle ones
        let mut new_points = Vec::new();
        // The middle points
        let mut mid_points = Vec::new();

        // Index into the per-hexagon material.
        let mut per_face_indices = Vec::new();
        let mut mid_face_indices = Vec::new();

        // This is re-used per main chunk.
        // It contains, for each point (key), every pair of neighbours (.edge)
        // and the index of the point in the middle of the triangle (.avg_idx)
        let mut surrounding_points: HashMap<u32, ArrayVec<SurroundingEntry, 6>> =
            HashMap::default();

        // Stores all of the indices of the new geometry.
        // The elements are either:
        // All(x) -> x is an index into the edge points.
        // Mid(x) -> x is an index into the middle points and should
        //           be added to the number of edge points when resolved.
        let mut temp_out_indices =
            Vec::with_capacity(((original_points.len() - 12) * 6 + (12 * 5)) * 3);

        let mut entities = Vec::new();

        // Graph Edge creation stuff:
        //
        // Edges from center id to center id.
        let mut unordered_edges = HashMap::default();
        // Center id to node index.
        let mut old_center_to_node = HashMap::default();

        let old_indices = sphere.get_all_indices();

        // I chose 6 & 7 arbitrarily, trying to avoid pentagons and
        // mostly scale hexagons.
        let scale_factor = 1.0 / (original_points[old_indices[6] as usize] - original_points[old_indices[7] as usize]).length();

        for triangle in old_indices.chunks(3) {
            add_chunk_triangle_indices(
                triangle[0],
                triangle[1],
                triangle[2],
                &mut unordered_edges,
                &original_points,
                &mut new_points,
                &mut per_face_indices,
                &mut surrounding_points,
            );
        }

        // For every full hexagon we get...
        for (old_center, sides) in surrounding_points.iter() {
            let mut sides = sides.clone();
            let center = mid_points.len();
            let entity = commands.spawn().id();
            old_center_to_node.insert(*old_center, entity);

            let mut ordered_points = ArrayVec::<usize, 6>::new();

            let SurroundingEntry {
                edge: [first, mut next],
                avg_idx: f_data,
            } = sides.pop().unwrap();

            ordered_points.push(f_data as usize);
            while sides.len() > 0 {
                let SurroundingEntry {
                    edge: [_, n],
                    avg_idx: data,
                } = sides.remove(
                    sides
                        .iter()
                        .position(|SurroundingEntry { edge: [x, _], .. }| *x == next)
                        .unwrap(),
                );
                next = n;
                ordered_points.push(data as _);
            }
            assert_eq!(next, first);

            let avg = ordered_points
                .iter()
                .fold(Vec3A::ZERO, |prev, idx| prev + new_points[*idx])
                .normalize();

            mid_points.push(avg);

            let mut iter = ordered_points.iter().copied().peekable();
            while let Some(a) = iter.next() {
                use Index::*;

                let b = *iter.peek().unwrap_or(&ordered_points[0]);
                temp_out_indices.extend_from_slice(&[
                    Mid(center as u32),
                    All(a as u32),
                    All(b as u32),
                ]);
            }

            mid_face_indices.push(1);

            entities.push(entity);
        }

        let packed_relations = PackedRelations {
            relations: unordered_edges
                .into_iter()
                .map(|((edge_a, edge_b), relation_data)| {
                    let a = *old_center_to_node.get(&edge_a).unwrap();
                    let b = *old_center_to_node.get(&edge_b).unwrap();

                    (RelationPair(a, b), relation_data)
                })
                .collect()
        };

        let (biomes, hmap, per_face_data) = self.state.make_biomes(&mid_points);

        let mut rng = rand::thread_rng();

        entities
            .iter()
            .zip(biomes.into_iter())
            .enumerate()
            .for_each(|(idx, (&entity, biome))| {
                let biome_idx = rng.gen_range(hmap.get(&biome).unwrap().clone());

                commands.entity(entity).insert_bundle((
                    GlobalTransform::default(),
                    make_point_transform(mid_points[idx]),
                    FaceMaterialIdx(biome_idx),
                    OldFaceMaterialIdx(biome_idx),
                    TileData { biome },
                    TileDataIdx(idx + per_face_indices.len()),
                ));
            });

        // Resolve real indices.
        let mid = new_points.len() as u32;
        let indices = temp_out_indices
            .into_iter()
            .map(|x| x.resolve(mid))
            .collect::<Vec<_>>();
        new_points.extend(mid_points.into_iter());
        per_face_indices.extend(mid_face_indices.into_iter());

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.set_attribute(
            Mesh::ATTRIBUTE_UV_0,
            new_points
                .iter()
                .map(|point| {
                    let inclination = point.y.acos();
                    let azimuth = point.z.atan2(point.x);

                    let norm_inclination = inclination / std::f32::consts::PI;
                    let norm_azimuth = 0.5 - (azimuth / std::f32::consts::TAU);

                    let incl_factor = 1.0 - ((norm_inclination - 0.5) * (norm_inclination - 0.5));

                    const TILING: f32 = 10.0;

                    [
                        norm_azimuth * TILING * 2.0 * incl_factor.powf(1.0 / 2.40942),
                        norm_inclination * TILING,
                    ]
                })
                .collect::<Vec<[f32; 2]>>(),
        );
        mesh.set_attribute(
            Mesh::ATTRIBUTE_POSITION,
            new_points
                .into_iter()
                .map(|p| [p.x, p.y, p.z])
                .collect::<Vec<[_; 3]>>(),
        );
        mesh.set_attribute(shaders::ATTRIBUTE_PER_FACE_INDEX, per_face_indices);

        println!("{:?}", scale_factor);

        commands
            .entity(board)
            //TODO: https://github.com/OptimisticPeach/hexagonfactory/issues/2
            .push_children(&entities)
            .insert_bundle(LowPolyPBRBundle {
                mesh: meshes.add(mesh),
                material: planet_materials.add(LowPolyMaterial {
                    per_face_data,
                    double_sided: true,
                    ..Default::default()
                }),
                transform: Transform::from_scale(Vec3::splat(scale_factor)),
                ..Default::default()
            })
            .insert(packed_relations);
    }
}
