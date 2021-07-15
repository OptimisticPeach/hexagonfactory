mod biomes;

pub use biomes::*;

pub fn make_biomes(metal_temp: &mut [[f32; 2]]) -> Vec<biomes::Biome> {
    metal_temp
        .iter()
        .map(|[metal, temp]| biomes::Biome::new(*temp, *metal))
        .collect()
}
