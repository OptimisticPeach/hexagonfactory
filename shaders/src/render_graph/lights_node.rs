use crate::{
    light::{LowPolyAmbientLight, LowPolyPointLight, LowPolyPointLightUniform},
    render_graph::uniform,
};
use bevy::core::{Pod, Zeroable};
use bevy::ecs::{
    system::{BoxedSystem, IntoSystem, Local, Query, Res, ResMut},
    world::World,
};
use bevy::render::{
    render_graph::{CommandQueue, Node, ResourceSlots, SystemNode},
    renderer::{
        BufferId, BufferInfo, BufferMapMode, BufferUsage, RenderContext, RenderResourceBinding,
        RenderResourceBindings, RenderResourceContext,
    },
};
use bevy::transform::prelude::*;

/// A Render Graph [Node] that write light data from the ECS to GPU buffers
#[derive(Debug, Default)]
pub struct LowPolyLightsNode {
    command_queue: CommandQueue,
    max_point_lights: usize,
}

impl LowPolyLightsNode {
    pub fn new(max_lights: usize) -> Self {
        LowPolyLightsNode {
            max_point_lights: max_lights,
            command_queue: CommandQueue::default(),
        }
    }
}

impl Node for LowPolyLightsNode {
    fn update(
        &mut self,
        _world: &World,
        render_context: &mut dyn RenderContext,
        _input: &ResourceSlots,
        _output: &mut ResourceSlots,
    ) {
        self.command_queue.execute(render_context);
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct LowPolyLightCount {
    // storing as a `[u32; 4]` for memory alignement
    pub num_lights: [u32; 4],
}

unsafe impl Pod for LowPolyLightCount {}
unsafe impl Zeroable for LowPolyLightCount {}

impl SystemNode for LowPolyLightsNode {
    fn get_system(&self) -> BoxedSystem {
        let system = lights_node_system.system().config(|config| {
            config.0 = Some(LowPolyLightsNodeSystemState {
                command_queue: self.command_queue.clone(),
                max_point_lights: self.max_point_lights,
                light_buffer: None,
                staging_buffer: None,
            })
        });
        Box::new(system)
    }
}

/// Local "lights node system" state
#[derive(Debug, Default)]
pub struct LowPolyLightsNodeSystemState {
    light_buffer: Option<BufferId>,
    staging_buffer: Option<BufferId>,
    command_queue: CommandQueue,
    max_point_lights: usize,
}

pub fn lights_node_system(
    mut state: Local<LowPolyLightsNodeSystemState>,
    render_resource_context: Res<Box<dyn RenderResourceContext>>,
    ambient_light_resource: Res<LowPolyAmbientLight>,
    // TODO: this write on RenderResourceBindings will prevent this system from running in parallel
    // with other systems that do the same
    mut render_resource_bindings: ResMut<RenderResourceBindings>,
    query: Query<(&LowPolyPointLight, &GlobalTransform)>,
) {
    let state = &mut state;
    let render_resource_context = &**render_resource_context;

    // premultiply ambient brightness
    let ambient_light: [f32; 4] =
        (ambient_light_resource.color * ambient_light_resource.brightness).into();
    let ambient_light_size = std::mem::size_of::<[f32; 4]>();
    let point_light_count = query.iter().len().min(state.max_point_lights);
    let size = std::mem::size_of::<LowPolyPointLightUniform>();
    let light_count_size = ambient_light_size + std::mem::size_of::<LowPolyLightCount>();
    let point_light_array_size = size * point_light_count;
    let point_light_array_max_size = size * state.max_point_lights;
    let current_point_light_uniform_size = light_count_size + point_light_array_size;
    let max_light_uniform_size = light_count_size + point_light_array_max_size;

    if let Some(staging_buffer) = state.staging_buffer {
        if point_light_count == 0 {
            return;
        }

        render_resource_context.map_buffer(staging_buffer, BufferMapMode::Write);
    } else {
        let buffer = render_resource_context.create_buffer(BufferInfo {
            size: max_light_uniform_size,
            buffer_usage: BufferUsage::UNIFORM | BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
            ..Default::default()
        });
        render_resource_bindings.set(
            uniform::LIGHTS,
            RenderResourceBinding::Buffer {
                buffer,
                range: 0..max_light_uniform_size as u64,
                dynamic_index: None,
            },
        );
        state.light_buffer = Some(buffer);

        let staging_buffer = render_resource_context.create_buffer(BufferInfo {
            size: max_light_uniform_size,
            buffer_usage: BufferUsage::COPY_SRC | BufferUsage::MAP_WRITE,
            mapped_at_creation: true,
        });
        state.staging_buffer = Some(staging_buffer);
    }

    let staging_buffer = state.staging_buffer.unwrap();
    render_resource_context.write_mapped_buffer(
        staging_buffer,
        0..current_point_light_uniform_size as u64,
        &mut |data, _renderer| {
            // ambient light
            data[0..ambient_light_size].copy_from_slice(bevy::core::cast_slice(&ambient_light));

            // light count
            data[ambient_light_size..light_count_size].copy_from_slice(bevy::core::cast_slice(&[
                point_light_count as u32,
                0,
                0,
                0,
            ]));

            // light array
            for ((point_light, global_transform), slot) in query.iter().zip(
                data[light_count_size..current_point_light_uniform_size].chunks_exact_mut(size),
            ) {
                slot.copy_from_slice(bevy::core::bytes_of(&LowPolyPointLightUniform::from(
                    &point_light,
                    &global_transform,
                )));
            }
        },
    );
    render_resource_context.unmap_buffer(staging_buffer);
    let light_buffer = state.light_buffer.unwrap();
    state.command_queue.copy_buffer_to_buffer(
        staging_buffer,
        0,
        light_buffer,
        0,
        max_light_uniform_size as u64,
    );
}
