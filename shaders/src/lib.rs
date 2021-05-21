pub mod render_graph;

mod entity;
mod light;
mod material;

pub use entity::*;
pub use light::*;
pub use material::*;

pub mod prelude {
    pub use crate::{entity::*, light::LowPolyPointLight, material::LowPolyMaterial};
}

use bevy::app::prelude::*;
use bevy::asset::{AddAsset, Assets, Handle};
use bevy::ecs::system::IntoSystem;
use bevy::render::shader;
use render_graph::add_pbr_graph;

pub const ATTRIBUTE_PER_FACE_INDEX: &'static str = "Per_Face_Index";

/// NOTE: this isn't PBR yet. consider this name "aspirational" :)
#[derive(Default)]
pub struct LowPolyPBRPlugin;

impl Plugin for LowPolyPBRPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<LowPolyMaterial>()
            .register_type::<LowPolyPointLight>()
            .init_resource::<LowPolyAmbientLight>()
            .add_system_to_stage(
                CoreStage::PostUpdate,
                shader::asset_shader_defs_system::<LowPolyMaterial>.system(),
            );
        add_pbr_graph(app.world_mut());

        // add default StandardMaterial
        let mut materials = app
            .world_mut()
            .get_resource_mut::<Assets<LowPolyMaterial>>()
            .unwrap();
        materials.set_untracked(
            Handle::<LowPolyMaterial>::default(),
            LowPolyMaterial {
                unlit: true,
                ..Default::default()
            },
        );
    }
}
