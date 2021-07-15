use bevy::ecs::system::{Query, ResMut, IntoSystem};
use bevy::asset::{Assets, Handle};
use crate::{BoardMember, FaceMaterialIdx, OldFaceMaterialIdx};
use bevy::transform::components::Parent;
use bevy::render::mesh::{Mesh, VertexAttributeValues};
use bevy::app::{AppBuilder, Plugin};
use bevy::ecs::query::Changed;
use bevy_mod_raycast::DefaultRaycastingPlugin;

pub struct BoardPlugin;

mod changed_tiletype;
mod select_tile;

pub use select_tile::PlanetTileRaycastSet;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_system(changed_tiletype::update_material_idx_system.system())
            .add_plugin(DefaultRaycastingPlugin::<PlanetTileRaycastSet>::default());
    }

    fn name(&self) -> &str {
        "SphereOrder Board"
    }
}
