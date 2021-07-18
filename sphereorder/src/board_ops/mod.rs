use bevy::ecs::system::IntoSystem;
use bevy::app::{AppBuilder, Plugin};
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
}
