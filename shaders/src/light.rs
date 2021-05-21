use bevy::core::{Pod, Zeroable};
use bevy::pbr::{AmbientLight, PointLight};
use bevy::transform::components::GlobalTransform;

pub type LowPolyPointLight = PointLight;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct LowPolyPointLightUniform {
    pub pos: [f32; 4],
    pub color: [f32; 4],
    // storing as a `[f32; 4]` for memory alignement
    pub light_params: [f32; 4],
}

unsafe impl Pod for LowPolyPointLightUniform {}
unsafe impl Zeroable for LowPolyPointLightUniform {}

impl LowPolyPointLightUniform {
    pub fn from(
        light: &LowPolyPointLight,
        global_transform: &GlobalTransform,
    ) -> LowPolyPointLightUniform {
        let (x, y, z) = global_transform.translation.into();

        // premultiply color by intensity
        // we don't use the alpha at all, so no reason to multiply only [0..3]
        let color: [f32; 4] = (light.color * light.intensity).into();

        LowPolyPointLightUniform {
            pos: [x, y, z, 1.0],
            color,
            light_params: [1.0 / (light.range * light.range), light.radius, 0.0, 0.0],
        }
    }
}

pub type LowPolyAmbientLight = AmbientLight;
