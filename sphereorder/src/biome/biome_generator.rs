//!
//! Biomes are controlled by the following
//! parameters:
//!
//! - Altitude
//!   - Very high altitude will cause temperature drops
//!   - Mid range altitude will cause increase in wetness
//! - Temperature
//!   - Extreme temperatures will decrease wetness
//!   - Mid range temperatures will increase wetness
//! - Slope
//!   - High slope will cause very minimal wetness
//!   - High slope will cause mineral spawns more often
//! - Wetness
//!
//!
//! And combinations of them:
//!
//! - High altitude low temperature will cause low wetness
//! - Low slope with high temperature and wetness will cause more wetness
//! - Low altitude with low slope will cause increase in wetness
//!

use crate::biome::biomes::Biome;

fn smoothstep(mut x: f32, e0: f32, e1: f32) -> f32 {
    x = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);

    x * x * x * (x * (x * 6.0 - 15.0) + 10.0)
}

pub fn simulate_rules(height_wet_temp: &mut [[f32; 3]], slopes: &[f32]) {
    let mut min_wet = f32::INFINITY;
    let mut max_wet = f32::NEG_INFINITY;

    let mut min_temp = f32::INFINITY;
    let mut max_temp = f32::NEG_INFINITY;

    height_wet_temp
        .iter()
        .copied()
        .for_each(|[_, wet, temp]| {
            min_wet = wet.min(min_wet);
            max_wet = wet.max(max_wet);

            min_temp = temp.min(min_temp);
            max_temp = temp.max(max_temp);
        });

    let scl_wet = 1.0 / (max_wet - min_wet);
    let scl_temp = 2.0 / (max_temp - min_temp);

    height_wet_temp
        .iter_mut()
        .zip(slopes.iter())
        .for_each(|([rel_height, wetness, temp], &slope)| {
            *wetness = (*wetness - min_wet) * scl_wet;
            *temp = ((*temp - min_temp) * scl_temp) - 1.0;
            let rel_height = *rel_height;
            // - Altitude
            //   - Very high altitude will cause temperature drops
            *temp *= 0.7 * smoothstep(rel_height, 0.6, 0.1) + 0.3;
            //   - Mid range altitude will cause decrease in wetness
            *wetness *= 1.0 + 0.2 * smoothstep(rel_height * rel_height, 0.085, 0.001);
            // - Temperature
            //   - Extreme temperatures will decrease wetness
            //   - Mid range temperatures will increase wetness
            *wetness *= 0.75 + 0.5 * smoothstep(*temp * *temp, 0.2, 0.0);
            // - Slope
            //   - High slope will cause very minimal wetness
            *wetness *= 0.1 + 0.9 * smoothstep(slope, 0.1, 0.05);
        })
}

pub fn make_biomes(height_wet_temp: &mut [[f32; 3]], slopes: &[f32]) -> Vec<Biome> {
    height_wet_temp
        .iter()
        .copied()
        .zip(
            slopes
                .iter()
                .copied()
        )
        .map(|([height, wet, temp], slope)| Biome::new(height, slope, temp, wet))
        .collect()
}
