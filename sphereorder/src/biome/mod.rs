mod biomes;

pub use biomes::*;

pub fn make_base_biomes(metal_temp: &[[f32; 2]]) -> Vec<biomes::Biome> {
    metal_temp
        .iter()
        .map(|[metal, temp]| biomes::Biome::new_base(*temp, *metal))
        .collect()
}

pub fn make_sky_biomes(land: &[[f32; 1]]) -> Vec<biomes::Biome> {
    land
        .iter()
        .map(|[land]| biomes::Biome::new_sky(*land))
        .collect()
}

pub fn make_space_biomes(land: &[[f32; 1]]) -> Vec<biomes::Biome> {
    land
        .iter()
        .map(|[land]| biomes::Biome::new_space(*land))
        .collect()
}
