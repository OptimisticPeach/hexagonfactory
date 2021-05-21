mod lights_node;
mod pbr_pipeline;

use bevy::ecs::world::World;
pub use lights_node::*;
pub use pbr_pipeline::*;

/// the names of pbr graph nodes
pub mod node {
    pub const TRANSFORM: &str = "lp_transform";
    pub const STANDARD_MATERIAL: &str = "lp_standard_material";
    pub const LIGHTS: &str = "lp_lights";
}

/// the names of pbr uniforms
pub mod uniform {
    pub const LIGHTS: &str = "Lights";
}

use crate::prelude::LowPolyMaterial;
use bevy::asset::Assets;
use bevy::render::{
    pipeline::PipelineDescriptor,
    render_graph::{base, AssetRenderResourcesNode, RenderGraph},
    shader::Shader,
};

pub const MAX_POINT_LIGHTS: usize = 10;

pub(crate) fn add_pbr_graph(world: &mut World) {
    {
        let mut graph = world.get_resource_mut::<RenderGraph>().unwrap();
        graph.add_system_node(
            node::STANDARD_MATERIAL,
            AssetRenderResourcesNode::<LowPolyMaterial>::new(false),
        );

        graph.add_system_node(node::LIGHTS, LowPolyLightsNode::new(MAX_POINT_LIGHTS));

        // TODO: replace these with "autowire" groups
        graph
            .add_node_edge(node::STANDARD_MATERIAL, base::node::MAIN_PASS)
            .unwrap();
        graph
            .add_node_edge(node::LIGHTS, base::node::MAIN_PASS)
            .unwrap();
    }
    let pipeline = build_pbr_pipeline(&mut world.get_resource_mut::<Assets<Shader>>().unwrap());
    let mut pipelines = world
        .get_resource_mut::<Assets<PipelineDescriptor>>()
        .unwrap();
    pipelines.set_untracked(LOW_POLY_PBR_PIPELINE_HANDLE, pipeline);
}
