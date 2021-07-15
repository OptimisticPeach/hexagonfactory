use petgraph::graph::NodeIndex;
use petgraph::Graph;
use bevy::ecs::entity::Entity;
use bevy::math::{Vec3, Quat, Mat4};
use shaders::PerFaceData;
use bevy::ecs::system::{Commands, Query, ResMut, IntoSystem};
use arrayvec::ArrayVec;
use bevy::math::Vec3A;
use bevy::render::mesh::{Indices, Mesh, VertexAttributeValues};
use bevy::render::pipeline::PrimitiveTopology;
use bevy::utils::{HashMap, HashSet};
use hexasphere::shapes::IcoSphere;

use parking_lot::RwLock;
use std::sync::Arc;
use bevy::transform::components::{Transform, GlobalTransform, Parent};
use bevy::transform::hierarchy::BuildChildren;

use rand::Rng;
use bevy::app::{Plugin, AppBuilder};
use bevy::ecs::query::Changed;
use bevy::asset::{Assets, Handle};

mod biome;
pub mod board_ops;

pub use board_ops::BoardPlugin;

pub struct ChunkData {
    pub vector: Vec3,
    pub chunk_id: usize,
}

pub struct BoardMember {
    pub board: Arc<BoardGraph>,
    pub graph_idx: NodeIndex,
    pub data_idx: usize,
    pub face_idx: usize,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FaceMaterialIdx(pub i32);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct OldFaceMaterialIdx(pub i32);

#[derive(Copy, Clone, Hash, PartialEq, Debug)]
pub struct TileLayers {
    pub base: Entity,
    pub unit: Option<Entity>,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct TileData {
    pub layers: TileLayers,
    pub biome: crate::biome::Biome,
    pub temperature: f32,
}

pub struct BoardGraph {
    pub total_graph: Graph<usize, (), petgraph::Undirected>,
    pub chunk_graph: Graph<ChunkData, ()>,
    pub subdivision: usize,
    pub per_chunk: usize,
    pub tile_data: [RwLock<Vec<TileData>>; 21],
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GeographicalParams {
    pub metal_seed: i32,
    pub temp_seed: i32,
}

pub struct BoardInitialization {
    board_type: BoardInitializationType,
    subdivisions: u32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BoardInitializationType {
    Empty,
    Base(GeographicalParams),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BoardBuilder {
    subdivisions: usize,
    state: BoardInitializationType,
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

struct SurroundingEntry {
    edge: [u32; 2],
    avg_idx: u32,
}

fn add_chunk_triangle_indices(
    a: u32,
    b: u32,
    c: u32,
    unordered_edges: &mut HashSet<(u32, u32)>,
    original_points: &[Vec3A],
    new_points: &mut Vec<Vec3A>,
    per_face_indices: &mut Vec<i32>,
    surrounding_points: &mut HashMap<u32, ArrayVec<SurroundingEntry, 6>>
) {
    unordered_edges.extend(std::array::IntoIter::new([(a.min(b), a.max(b)), (b.min(c), b.max(c)), (c.min(a), c.max(a))]));

    let avg = (original_points[a as usize] + original_points[b as usize] + original_points[c as usize]) / 3.0;
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

fn strip_unfinished_edges<'a>(
    surrounding_points: &'a mut HashMap<u32, ArrayVec<SurroundingEntry, 6>>,
    edge_chunk_surrounding_points: &'a mut HashMap<u32, ArrayVec<SurroundingEntry, 6>>,
) -> impl Iterator<Item = (u32, ArrayVec<SurroundingEntry, 6>)> + 'a {
    surrounding_points
        .drain()
        .map(move |(k, v)| {
            if v.len() == 6 {
                Some((k, v))
            } else {
                match edge_chunk_surrounding_points.entry(k) {
                    std::collections::hash_map::Entry::Occupied(mut x) => { x.get_mut().extend(v); },
                    std::collections::hash_map::Entry::Vacant(x) => { x.insert(v); },
                }
                None
            }
        })
        .flatten()
}

fn make_point_transform(normalized_point: Vec3A) -> Transform {
    let x = (Quat::from_rotation_y(0.1) * normalized_point).normalize();
    let z = normalized_point.cross(x);

    Transform::from_matrix(Mat4::from_cols(x.extend(0.0), normalized_point.extend(0.0), z.extend(0.0), normalized_point.extend(1.0)))
}

pub fn idx_to_chunk(idx: usize, per_chunk: usize) -> (usize, usize) {
    let div = idx / per_chunk;
    if div > 19 {
        (20, idx - (20 * per_chunk))
    } else {
        (div, idx % per_chunk)
    }
}

impl BoardBuilder {
    pub fn create(&self, commands: &mut Commands) -> (Mesh, Vec<PerFaceData>, Entity) {
        let board = commands.spawn().id();

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
        let mut temp_out_indices = Vec::with_capacity(((original_points.len() - 12) * 6 + (12 * 5)) * 3);

        // Contains the source indices for processing each chunk.
        // Cleared each loop.
        let mut index_buffer = Vec::new();

        // Same idea as `surrounding_points`, however, when an entry
        // in `surrounding_points` is not full (has 6 edges), it is
        // inserted here or extends something that's already here.
        // This data builds the edges "chunk".
        let mut edge_chunk_surrounding_points = HashMap::<u32, ArrayVec<SurroundingEntry, 6>>::default();

        let mut cumulative_graph = Graph::new_undirected();

        let mut chunk_descriptors = Vec::with_capacity(21);
        let mut chunk_tile_data = ArrayVec::<Vec<TileData>, 21>::new();
        let mut entities = Vec::new();

        // Graph Edge creation stuff:
        //
        // Edges from center id to center id.
        let mut unordered_edges = HashSet::default();
        // Center id to node index.
        let mut old_center_to_node = HashMap::default();

        for chunk in 0..20 {
            index_buffer.clear();
            sphere.get_indices(chunk, &mut index_buffer);

            for triangle in index_buffer.chunks(3) {
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

            let inner = strip_unfinished_edges(&mut surrounding_points, &mut edge_chunk_surrounding_points);

            // Normal vector for the chunk.
            let mut chunk_normal = Vec3A::ZERO;

            // This is the tile data for this chunk.
            chunk_tile_data.push(Vec::new());

            // For every full hexagon we get...
            for (old_center, mut sides) in inner {
                let center = mid_points.len();
                let node = cumulative_graph.add_node(center);
                old_center_to_node.insert(old_center, node);
                assert_eq!(sides.len(), 6);

                let mut ordered_points = ArrayVec::<usize, 6>::new();

                let SurroundingEntry {
                    edge: [first, mut next],
                    avg_idx: f_data
                } = sides.pop().unwrap();

                ordered_points.push(f_data as usize);
                while sides.len() > 0 {
                    let SurroundingEntry { edge: [_, n], avg_idx: data } =
                        sides.remove(sides.iter().position(|SurroundingEntry { edge: [x, _], .. }| *x == next).unwrap());
                    next = n;
                    ordered_points.push(data as _);
                }
                assert_eq!(next, first);

                let avg = ordered_points
                    .iter()
                    .fold(Vec3A::ZERO, |prev, idx| prev + new_points[*idx])
                    .normalize();

                chunk_normal += avg;

                mid_points.push(avg);

                let mut iter = ordered_points.iter().copied().peekable();
                while let Some(a) = iter.next() {
                    use Index::*;

                    let b = *iter.peek().unwrap_or(&ordered_points[0]);
                    temp_out_indices.extend_from_slice(&[Mid(center as u32), All(a as u32), All(b as u32)]);
                }

                mid_face_indices.push(1);

                let entity = commands
                    .spawn()
                    .insert(GlobalTransform::default())
                    .id();

                commands
                    .entity(board.clone())
                    .push_children(&[entity]);

                entities.push(
                    (
                        // For each entity, we need to add:
                        entity,
                        // The node index
                        node,
                        // The per-face index which tells us the
                        // which material to index later.
                        center,
                    )
                );
            }
            
            chunk_normal = chunk_normal.normalize();

            let chunk_data = ChunkData {
                vector: chunk_normal.into(),
                chunk_id: chunk,
            };

            chunk_descriptors.push(chunk_data);
        }

        let per_chunk_len = entities.len() / 20;

        // Deal with the border/edge chunk.
        {
            chunk_tile_data.push(Vec::new());

            for (old_center, mut sides) in edge_chunk_surrounding_points {
                let center = mid_points.len();
                let node = cumulative_graph.add_node(center);
                old_center_to_node.insert(old_center, node);

                let mut ordered_points = ArrayVec::<usize, 6>::new();

                let SurroundingEntry {
                    edge: [first, mut next],
                    avg_idx: f_data
                } = sides.pop().unwrap();

                ordered_points.push(f_data as usize);
                while sides.len() > 0 {
                    let SurroundingEntry { edge: [_, n], avg_idx: data } =
                        sides.remove(sides.iter().position(|SurroundingEntry { edge: [x, _], .. }| *x == next).unwrap());
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
                    temp_out_indices.extend_from_slice(&[Mid(center as u32), All(a as u32), All(b as u32)]);
                }

                mid_face_indices.push(2);

                let entity = commands
                    .spawn()
                    .insert(GlobalTransform::default())
                    .id();

                commands
                    .entity(board.clone())
                    .push_children(&[entity]);

                entities.push(
                    (
                        // For each entity, we need to add:
                        entity,
                        // The node index
                        node,
                        // The per-face index which tells us
                        center,
                    )
                );
            }
        }

        for (edge_a, edge_b) in unordered_edges {
            cumulative_graph.add_edge(
                *old_center_to_node.get(&edge_a).unwrap(),
                *old_center_to_node.get(&edge_b).unwrap(),
                ()
            );
        }

        let geographical_parameters = if let BoardInitializationType::Base(x) = self.state { x } else { panic!() };

        let mut tile_datas = noise_gen::sample_all_noise(&mid_points,
            [
                // Whether it's dirt or metal
                noise_gen::NoiseParameters {
                    scale: 0.5,
                    lac: 0.1,
                    gain: 0.9,
                    min: 0.0,
                    max: 1.0,
                    seed: geographical_parameters.metal_seed,
                },
                // Whether it's terrain or lava
                noise_gen::NoiseParameters {
                    scale: 1.0,
                    lac: 1.0,
                    gain: 1.0,
                    min: -1.0,
                    max: 1.0,
                    seed: geographical_parameters.temp_seed,
                },
            ]
        );

        let biomes = crate::biome::make_biomes(&mut tile_datas);

        let per_face_data = crate::biome::BIOME_COLOURS.to_vec();

        let mut rng = rand::thread_rng();
        let hmap = &*crate::biome::BIOME_MAP;

        entities
            .iter()
            .map(|x| x.0)
            .zip(biomes.into_iter())
            .zip(tile_datas.into_iter())
            .enumerate()
            .for_each(|(idx, ((entity, biome), [_metal, temp ]))| {
                let biome_idx = rng.gen_range(hmap.get(&biome).unwrap().clone());
                // mid_face_indices[*face_idx] = biome;

                commands
                    .entity(entity)
                    .insert_bundle(
                        (
                            make_point_transform(mid_points[idx]),
                            FaceMaterialIdx(biome_idx),
                            OldFaceMaterialIdx(biome_idx),
                        )
                    );

                let (chunk_idx, _) = idx_to_chunk(idx, per_chunk_len);

                chunk_tile_data[chunk_idx].push(TileData {
                    layers: TileLayers {
                        base: entity,
                        unit: None
                    },
                    biome,
                    temperature: temp,
                });
            });


        let total_board = Arc::new(
            BoardGraph {
                total_graph: cumulative_graph,
                chunk_graph: Default::default(),
                subdivision: self.subdivisions,
                per_chunk: chunk_tile_data[0].len(),
                tile_data: chunk_tile_data
                    .into_iter()
                    .map(|x| RwLock::new(x))
                    .collect::<ArrayVec<_, 21>>()
                    .into_inner()
                    .unwrap(),
            }
        );

        entities
            .iter()
            .enumerate()
            .for_each(|(idx, (entity, node, face_idx))| {
                // mid_face_indices[idx] = 25;
                commands
                    .entity(*entity)
                    .insert(
                        BoardMember {
                            board: total_board.clone(),
                            graph_idx: *node,
                            data_idx: idx,
                            face_idx: idx + per_face_indices.len()
                        }
                    );
            });

        // Resolve real indices.
        let mid = new_points.len() as u32;
        let indices = temp_out_indices.into_iter().map(|x| x.resolve(mid)).collect::<Vec<_>>();
        // let len = per_face_indices.len();
        new_points.extend(mid_points.into_iter());
        per_face_indices.extend(mid_face_indices.into_iter());

        // entities
        //     .iter()
        //     .for_each(|(_, _, idx)| {
        //         let &idx = idx;
        //         per_face_indices[len + idx] = 4;
        //     });

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

        (mesh, per_face_data, board)
    }
}

impl BoardGraph {
    pub fn build(subdivisions: usize) -> BoardBuilder {
        BoardBuilder {
            subdivisions,
            state: BoardInitializationType::Base(GeographicalParams {
                metal_seed: 123,
                temp_seed: 30,
            }),
        }
    }
}
