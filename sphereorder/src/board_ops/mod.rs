use bevy::app::{AppBuilder, Plugin};
use bevy::ecs::system::{Commands, Query, ResMut};

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

mod changed_tiletype;
mod select_tile;

use crate::PlanetDesc;
use bevy::asset::Assets;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::Added;
use bevy::render::mesh::Mesh;
pub use select_tile::PlanetTileRaycastSet;
use shaders::LowPolyMaterial;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(changed_tiletype::update_material_idx_system)
            .add_system(Self::add_new_planets);
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
}
