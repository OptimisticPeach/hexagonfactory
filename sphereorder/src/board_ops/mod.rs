use bevy::app::{AppBuilder, Plugin};
use bevy::ecs::system::{Commands, Query, ResMut};
use crate::PlanetDesc;
use bevy::asset::Assets;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::Added;
use bevy::render::mesh::Mesh;
pub use select_tile::PlanetTileRaycastSet;
use shaders::LowPolyMaterial;
use bevy::ecs::schedule::{SystemSet, State};
use bevy::ecs::event::EventReader;
use crate::camera::LayerChangeEvent;
use crate::board_ops::load::LoadState;
use crate::board_ops::unload::UnloadState;
use bevy::transform::components::Parent;
use smallvec::SmallVec;
use std::ops::Deref;

mod changed_tiletype;
mod select_tile;
mod unload;
mod load;

pub struct BoardPlugin;
pub struct PlanetLayerOf;

pub struct LayerChild {
    layer_number: usize
}

impl LayerChild {
    pub fn subdivisions(&self) -> usize {
        self.layer_number * 3 + 8
    }
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Layers {
    layers: SmallVec<[Entity; 10]>,
}

impl Deref for Layers {
    type Target = [Entity];

    fn deref(&self) -> &[Entity] {
        &self.layers
    }
}

impl<Q: AsRef<[Entity]>> From<Q> for Layers {
    fn from(layers: Q) -> Self {
        Self {
            layers: layers.as_ref().iter().copied().collect(),
        }
    }
}

pub enum OpState {
    Pending,
    Finished,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum LayerLoadState {
    LoadUnload,
    Finished,
}

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(changed_tiletype::update_material_idx_system)
            .add_system(Self::add_new_planets);
            // .add_system_set(
            //     SystemSet::on_update(LayerLoadState::LoadUnload)
            //         .with_system(load::loader::<5>)
            //         .with_system(unload::unloader::<5>)
            // )
            // .add_system_set(
            //     SystemSet::on_exit(LayerLoadState::LoadUnload)
            //         .with_system(load::load_all)
            //         .with_system(unload::unload_all)
            // )
            // .add_system(Self::layer_event_watcher);
    }
}

impl BoardPlugin {
    fn add_new_planets(
        query: Query<(Entity, &PlanetDesc), Added<PlanetDesc>>,
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut planet_materials: ResMut<Assets<LowPolyMaterial>>,
    ) {
        for (new_planet, planet) in query.iter() {
            crate::BoardBuilder::from(*planet).create_on(
                &mut commands,
                new_planet,
                &mut *meshes,
                &mut *planet_materials,
            );
        }
    }

    // fn layer_event_watcher(
    //     mut events: EventReader<LayerChangeEvent>,
    //     mut state: ResMut<State<LayerLoadState>>,
    //     mut load: ResMut<LoadState>,
    //     mut unload: ResMut<UnloadState>,
    // ) {
    //     if let Some(event) = events.iter().next() {
    //
    //     }
    //
    //
    // }
}
