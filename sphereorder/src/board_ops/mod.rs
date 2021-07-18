use bevy::ecs::system::{Query, Commands, ResMut};
use bevy::app::{AppBuilder, Plugin};
use bevy_mod_raycast::DefaultRaycastingPlugin;

pub struct BoardPlugin;

mod changed_tiletype;
mod select_tile;

pub use select_tile::PlanetTileRaycastSet;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::Added;
use crate::PlanetDesc;
use shaders::{LowPolyPBRBundle, LowPolyMaterial};
use bevy::asset::Assets;
use bevy::render::mesh::Mesh;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_system(changed_tiletype::update_material_idx_system)
            .add_system(Self::add_new_planets)
            .add_plugin(DefaultRaycastingPlugin::<PlanetTileRaycastSet>::default());
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
            let(mesh, per_face_data) = crate::BoardBuilder::from(*planet).create_on(&mut commands, new_planet);
            commands
                .entity(new_planet)
                .insert_bundle(
                    LowPolyPBRBundle {
                        mesh: meshes.add(mesh),
                        material: planet_materials.add(LowPolyMaterial {
                            per_face_data,
                            ..Default::default()
                        }),
                        ..Default::default()
                    }
                );
        }
    }
}
