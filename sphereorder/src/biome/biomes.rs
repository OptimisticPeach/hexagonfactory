//!
//! Biomes are controlled by the following
//! parameters:
//!
//! - Altitude
//! - Temperature
//! - Slope
//! - Wetness
//!
use Measure::*;
use Biome::*;
use shaders::PerFaceData;
use bevy::render::color::Color;
use std::ops::Range;
use bevy::utils::HashMap;

#[derive(Copy, Clone, Debug, PartialEq, Hash, Eq)]
pub enum Biome {
    Water,
    Jungle,
    Forest,
    Plains,
    Savannah,
    Steppes,
    MountainSide,
    Ice,
    Desert,
    Red,
}

lazy_static::lazy_static! {
    pub static ref BIOME_MAP: HashMap<Biome, Range<i32>> = std::array::IntoIter::new([
        (Savannah, 1..4),
        (Forest, 4..7),
        (Jungle, 7..10),
        (Ice, 10..13),
        (Desert, 13..15),
        (MountainSide, 15..17),
        (Water, 17..19),
        (Red, 0..1),
        (Plains, 19..22),
        (Steppes, 22..25),
    ])
    .collect();

    pub static ref BIOME_COLOURS: [PerFaceData; 25] = [
        PerFaceData {
            colour: Color::rgb_linear(1.0, 0.0, 1.0).as_linear_rgba_f32(),
            ..Default::default()
        },
        // Savannah
        PerFaceData {
            colour: Color::ORANGE.as_linear_rgba_f32(),
            reflectance: 0.1,
            roughness: 0.9,
            metallic: 0.7,
            ..Default::default()
        },
        PerFaceData {
            colour: Color::ORANGE.as_linear_rgba_f32(),
            reflectance: 0.0,
            roughness: 0.6,
            metallic: 0.2,
            ..Default::default()
        },
        PerFaceData {
            colour: Color::ORANGE.as_linear_rgba_f32(),
            reflectance: 0.2,
            roughness: 0.7,
            metallic: 0.7,
            ..Default::default()
        },
        // Forest
        PerFaceData {
            colour: Color::DARK_GREEN.as_linear_rgba_f32(),
            reflectance: 0.1,
            roughness: 0.9,
            metallic: 0.0,
            ..Default::default()
        },
        PerFaceData {
            colour: Color::DARK_GREEN.as_linear_rgba_f32(),
            reflectance: 0.4,
            roughness: 0.8,
            metallic: 0.05,
            ..Default::default()
        },
        PerFaceData {
            colour: Color::DARK_GREEN.as_linear_rgba_f32(),
            reflectance: 0.2,
            roughness: 0.7,
            metallic: 0.5,
            ..Default::default()
        },
        // Jungle
        PerFaceData {
            colour: Color::LIME_GREEN.as_linear_rgba_f32(),
            reflectance: 0.1,
            roughness: 0.9,
            metallic: 0.0,
            ..Default::default()
        },
        PerFaceData {
            colour: Color::GREEN.as_linear_rgba_f32(),
            reflectance: 0.4,
            roughness: 0.8,
            metallic: 0.05,
            ..Default::default()
        },
        PerFaceData {
            colour: Color::YELLOW_GREEN.as_linear_rgba_f32(),
            reflectance: 0.2,
            roughness: 0.7,
            metallic: 0.5,
            ..Default::default()
        },
        // Snow
        PerFaceData {
            colour: Color::ALICE_BLUE.as_linear_rgba_f32(),
            reflectance: 0.6,
            roughness: 0.3,
            metallic: 0.1,
            ..Default::default()
        },
        PerFaceData {
            colour: Color::ALICE_BLUE.as_linear_rgba_f32(),
            reflectance: 0.8,
            roughness: 0.2,
            metallic: 0.4,
            ..Default::default()
        },
        PerFaceData {
            colour: Color::WHITE.as_linear_rgba_f32(),
            reflectance: 0.9,
            roughness: 0.95,
            metallic: 0.2,
            ..Default::default()
        },
        // Desert
        PerFaceData {
            colour: (Color::YELLOW * 0.9).as_linear_rgba_f32(),
            reflectance: 0.9,
            roughness: 1.0,
            metallic: 0.1,
            ..Default::default()
        },
        PerFaceData {
            colour: (Color::YELLOW * 0.8).as_linear_rgba_f32(),
            reflectance: 0.8,
            roughness: 0.8,
            metallic: 0.3,
            ..Default::default()
        },
        // Stone
        PerFaceData {
            colour: Color::GRAY.as_linear_rgba_f32(),
            reflectance: 0.1,
            roughness: 0.8,
            metallic: 0.2,
            ..Default::default()
        },
        PerFaceData {
            colour: Color::GRAY.as_linear_rgba_f32(),
            reflectance: 0.0,
            roughness: 1.0,
            metallic: 0.1,
            ..Default::default()
        },
        // Water placeholder
        PerFaceData {
            colour: Color::BLUE.as_linear_rgba_f32(),
            reflectance: 1.0,
            roughness: 0.0,
            metallic: 0.1,
            ..Default::default()
        },
        PerFaceData {
            colour: (Color::BLUE * 0.9).as_linear_rgba_f32(),
            reflectance: 0.9,
            roughness: 0.05,
            metallic: 0.2,
            ..Default::default()
        },
        // Plains
        PerFaceData {
            colour: Color::rgb_linear(159.0 / 255.0, 247.0 / 255.0, 141.0 / 255.0).as_linear_rgba_f32(),
            reflectance: 0.1,
            roughness: 0.9,
            metallic: 0.7,
            ..Default::default()
        },
        PerFaceData {
            colour: Color::rgb_linear(159.0 / 255.0, 247.0 / 255.0, 141.0 / 255.0).as_linear_rgba_f32(),
            reflectance: 0.0,
            roughness: 0.6,
            metallic: 0.2,
            ..Default::default()
        },
        PerFaceData {
            colour: Color::rgb_linear(159.0 / 255.0, 247.0 / 255.0, 141.0 / 255.0).as_linear_rgba_f32(),
            reflectance: 0.2,
            roughness: 0.7,
            metallic: 0.7,
            ..Default::default()
        },
        // Steppes
        PerFaceData {
            colour: Color::rgb_linear(240.0 / 255.0, 230.0 / 255.0, 127.0 / 255.0).as_linear_rgba_f32(),
            reflectance: 0.1,
            roughness: 0.9,
            metallic: 0.7,
            ..Default::default()
        },
        PerFaceData {
            colour: Color::rgb_linear(240.0 / 255.0, 230.0 / 255.0, 127.0 / 255.0).as_linear_rgba_f32(),
            reflectance: 0.0,
            roughness: 0.6,
            metallic: 0.2,
            ..Default::default()
        },
        PerFaceData {
            colour: Color::rgb_linear(240.0 / 255.0, 230.0 / 255.0, 127.0 / 255.0).as_linear_rgba_f32(),
            reflectance: 0.2,
            roughness: 0.7,
            metallic: 0.7,
            ..Default::default()
        },
    ];
}

#[derive(Debug)]
enum Measure {
    Low,
    Mid,
    High,
}

impl Measure {
    pub fn height(x: f32) -> Self {
        if x < -0.01 {
            Low
        } else if x < 0.01 {
            Mid
        } else {
            High
        }
    }

    pub fn slope(x: f32) -> Self {
        if x < 0.0001 {
            Low
        } else if x < 0.1 {
            Mid
        } else {
            High
        }
    }

    pub fn temp(x: f32) -> Self {
        if x < -0.5 {
            Low
        } else if x < 0.5 {
            Mid
        } else {
            High
        }
    }

    pub fn humid(x: f32) -> Self {
        if x < 0.25 {
            Low
        } else if x < 0.75 {
            Mid
        } else {
            High
        }
    }
}

impl Biome {
    pub fn new(height: f32, slope: f32, temperature: f32, humidity: f32) -> Self {
        match (Measure::height(height), Measure::slope(slope), Measure::temp(temperature), Measure::humid(humidity)) {
            (Low, _, _, _) => Water,
            (Mid, _, High, High) => Jungle,
            (Mid, _, Mid, High) => Forest,
            (Mid | High, Mid | High, High, Low) => Desert,
            (Mid | High, Low, High, Low) => Savannah,
            (Mid | High, _, Low, Low) => Ice,
            (Mid | High, Low, Mid, Mid | Low) => Plains,
            (Mid | High, Mid, Mid, Mid) => Steppes,
            (Mid | High, High, _, _) => MountainSide,
            (High, _, _, _) => Ice,
            _ => {
                // println!("{:?}", x);
                Plains
            },
        }
    }
}
