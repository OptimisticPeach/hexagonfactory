use bevy::asset::Handle;
use bevy::core::{Pod, Zeroable};
use bevy::reflect::TypeUuid;
use bevy::render::{color::Color, renderer::RenderResources, shader::ShaderDefs, texture::Texture};

/// A material with "standard" properties used in PBR lighting
/// Standard property values with pictures here https://google.github.io/filament/Material%20Properties.pdf
#[derive(Debug, RenderResources, ShaderDefs, TypeUuid)]
#[uuid = "490ba3eb-9794-477b-ac46-b1edb5398758"]
pub struct LowPolyMaterial {
    #[shader_def]
    pub base_color_texture: Option<Handle<Texture>>,
    #[shader_def]
    pub metallic_roughness_texture: Option<Handle<Texture>>,
    #[shader_def]
    pub normal_map: Option<Handle<Texture>>,
    #[render_resources(ignore)]
    #[shader_def]
    pub double_sided: bool,
    #[shader_def]
    pub occlusion_texture: Option<Handle<Texture>>,
    #[shader_def]
    pub emissive_texture: Option<Handle<Texture>>,
    #[render_resources(ignore)]
    #[shader_def]
    pub unlit: bool,
    // Per face materials.
    #[render_resources(buffer)]
    pub per_face_data: Vec<PerFaceData>,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct PerFaceData {
    /// Doubles as diffuse albedo for non-metallic, specular for metallic and a mix for everything
    /// in between If used together with a base_color_texture, this is factored into the final
    /// base color as `base_color * base_color_texture_value`
    pub colour: [f32; 4],

    // Use a color for user friendliness even though we technically don't use the alpha channel
    // Might be used in the future for exposure correction in HDR
    pub emissive: [f32; 4],
    /// Linear perceptual roughness, clamped to [0.089, 1.0] in the shader
    /// Defaults to minimum of 0.089
    /// If used together with a roughness/metallic texture, this is factored into the final base
    /// color as `roughness * roughness_texture_value`
    pub roughness: f32,

    /// From [0.0, 1.0], dielectric to pure metallic
    /// If used together with a roughness/metallic texture, this is factored into the final base
    /// color as `metallic * metallic_texture_value`
    pub metallic: f32,
    /// Specular intensity for non-metals on a linear scale of [0.0, 1.0]
    /// defaults to 0.5 which is mapped to 4% reflectance in the shader
    pub reflectance: f32,

    pub flags: u32,
}

pub const METALLIC_ROUGHNESS_MAP: u32 = 0x0000000000000001;
pub const NORMAL_MAP: u32 = 0x0000000000000002;
pub const EMISSIVE_MAP: u32 = 0x0000000000000004;

impl Default for PerFaceData {
    fn default() -> Self {
        PerFaceData {
            colour: Color::CYAN.into(),
            // This is the minimum the roughness is clamped to in shader code
            // See https://google.github.io/filament/Filament.html#materialsystem/parameterization/
            // It's the minimum floating point value that won't be rounded down to 0 in the
            // calculations used. Although technically for 32-bit floats, 0.045 could be
            // used.
            roughness: 0.089,
            // Few materials are purely dielectric or metallic
            // This is just a default for mostly-dielectric
            metallic: 0.01,
            reflectance: 0.5,
            emissive: Color::BLACK.into(),
            flags: 0,
        }
    }
}

unsafe impl Pod for PerFaceData {}
unsafe impl Zeroable for PerFaceData {}

impl Default for LowPolyMaterial {
    fn default() -> Self {
        LowPolyMaterial {
            base_color_texture: None,
            // Minimum real-world reflectance is 2%, most materials between 2-5%
            // Expressed in a linear scale and equivalent to 4% reflectance see https://google.github.io/filament/Material%20Properties.pdf
            metallic_roughness_texture: None,
            normal_map: None,
            double_sided: false,
            occlusion_texture: None,
            emissive_texture: None,
            unlit: false,
            per_face_data: vec![],
        }
    }
}

impl From<Handle<Texture>> for LowPolyMaterial {
    fn from(texture: Handle<Texture>) -> Self {
        LowPolyMaterial {
            base_color_texture: Some(texture),
            ..Default::default()
        }
    }
}
