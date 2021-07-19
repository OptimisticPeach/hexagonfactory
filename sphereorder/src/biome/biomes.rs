use bevy::render::color::Color;
use bevy::utils::HashMap;
use shaders::PerFaceData;
use std::ops::Range;
use Biome::*;
use Measure::*;

#[derive(Copy, Clone, Debug, PartialEq, Hash, Eq)]
pub enum Biome {
    Dirt,
    Metal,
    Lava,
    Ice,
    Cloud,
    Asteroid,
    Platform,
    Empty,
}

lazy_static::lazy_static! {
    pub static ref SPACE_BIOME_MAP: HashMap<Biome, Range<i32>> = std::array::IntoIter::new([
        (Empty, 1..2),
        (Asteroid, 2..5),
        (Platform, 5..6),
    ]).collect();

    pub static ref SPACE_BIOME_COLOURS: [PerFaceData; 6] = [
        PerFaceData {
            colour: Color::rgb_linear(1.0, 0.0, 1.0).as_linear_rgba_f32(),
            ..Default::default()
        },
        // Empty
        PerFaceData {
            colour: [0.0; 4],
            ..Default::default()
        },
        // Asteroid
        PerFaceData {
            colour: Color::rgb_u8(153, 129, 61).as_linear_rgba_f32(),
            reflectance: 0.1,
            roughness: 0.9,
            metallic: 0.05,
            ..Default::default()
        },
        PerFaceData {
            colour: Color::rgb_u8(125, 96, 42).as_linear_rgba_f32(),
            reflectance: 0.0,
            roughness: 0.6,
            metallic: 0.0,
            ..Default::default()
        },
        PerFaceData {
            colour: Color::rgb_u8(122, 92, 37).as_linear_rgba_f32(),
            reflectance: 0.07,
            roughness: 0.7,
            metallic: 0.0,
            ..Default::default()
        },
        // Platform
        PerFaceData {
            colour: Color::GRAY.as_linear_rgba_f32(),
            reflectance: 0.0,
            roughness: 1.0,
            metallic: 0.1,
            ..Default::default()
        },
    ];

    pub static ref SKY_BIOME_MAP: HashMap<Biome, Range<i32>> = std::array::IntoIter::new([
        (Empty, 1..2),
        (Cloud, 2..4),
        (Platform, 4..5),
    ]).collect();

    pub static ref SKY_BIOME_COLOURS: [PerFaceData; 5] = [
        PerFaceData {
            colour: Color::rgb_linear(1.0, 0.0, 1.0).as_linear_rgba_f32(),
            ..Default::default()
        },
        // Empty
        PerFaceData {
            colour: [0.0; 4],
            ..Default::default()
        },
        // Cloud
        PerFaceData {
            colour: Color::hex("f9f4e880").unwrap().as_linear_rgba_f32(),
            reflectance: 0.0,
            roughness: 1.0,
            metallic: 0.0,
            ..Default::default()
        },
        PerFaceData {
            colour: Color::hex("f4ebde80").unwrap().as_linear_rgba_f32(),
            reflectance: 0.0,
            roughness: 1.0,
            metallic: 0.0,
            ..Default::default()
        },
        // Platform
        PerFaceData {
            colour: Color::hex("80808080").unwrap().as_linear_rgba_f32(),
            reflectance: 0.0,
            roughness: 1.0,
            metallic: 0.1,
            ..Default::default()
        },
    ];

    pub static ref BASE_BIOME_MAP: HashMap<Biome, Range<i32>> = std::array::IntoIter::new([
        (Empty, 1..2),
        (Dirt, 2..5),
        (Metal, 5..8),
        (Lava, 8..11),
        (Ice, 11..14),
    ])
    .collect();

    pub static ref BASE_BIOME_COLOURS: [PerFaceData; 14] = [
        PerFaceData {
            colour: Color::rgb_linear(1.0, 0.0, 1.0).as_linear_rgba_f32(),
            ..Default::default()
        },
        // Empty
        PerFaceData {
            colour: [0.0; 4],
            ..Default::default()
        },
        // Dirt
        PerFaceData {
            colour: Color::rgb_u8(153, 129, 61).as_linear_rgba_f32(),
            reflectance: 0.1,
            roughness: 0.9,
            metallic: 0.05,
            ..Default::default()
        },
        PerFaceData {
            colour: Color::rgb_u8(125, 96, 42).as_linear_rgba_f32(),
            reflectance: 0.0,
            roughness: 0.6,
            metallic: 0.0,
            ..Default::default()
        },
        PerFaceData {
            colour: Color::rgb_u8(122, 92, 37).as_linear_rgba_f32(),
            reflectance: 0.07,
            roughness: 0.7,
            metallic: 0.0,
            ..Default::default()
        },
        // Metal
        PerFaceData {
            colour: Color::GRAY.as_linear_rgba_f32(),
            reflectance: 0.8,
            roughness: 0.7,
            metallic: 0.9,
            ..Default::default()
        },
        PerFaceData {
            colour: Color::GRAY.as_linear_rgba_f32(),
            reflectance: 0.5,
            roughness: 0.9,
            metallic: 0.5,
            ..Default::default()
        },
        PerFaceData {
            colour: Color::DARK_GRAY.as_linear_rgba_f32(),
            reflectance: 0.8,
            roughness: 0.7,
            metallic: 0.9,
            ..Default::default()
        },
        // Lava
        PerFaceData {
            colour: Color::RED.as_linear_rgba_f32(),
            reflectance: 0.1,
            roughness: 0.9,
            metallic: 0.0,
            emissive: Color::RED.as_linear_rgba_f32(),
            ..Default::default()
        },
        PerFaceData {
            colour: Color::ORANGE.as_linear_rgba_f32(),
            reflectance: 0.3,
            roughness: 0.8,
            metallic: 0.0,
            emissive: Color::ORANGE.as_linear_rgba_f32(),
            ..Default::default()
        },
        PerFaceData {
            colour: Color::ORANGE_RED.as_linear_rgba_f32(),
            reflectance: 0.2,
            roughness: 0.7,
            metallic: 0.0,
            emissive: Color::ORANGE_RED.as_linear_rgba_f32(),
            ..Default::default()
        },
        // Ice
        PerFaceData {
            colour: Color::hex("60DCFF").unwrap().as_linear_rgba_f32(),
            reflectance: 0.6,
            roughness: 0.3,
            metallic: 0.1,
            ..Default::default()
        },
        PerFaceData {
            colour: Color::hex("65E5FF").unwrap().as_linear_rgba_f32(),
            reflectance: 0.8,
            roughness: 0.2,
            metallic: 0.4,
            ..Default::default()
        },
        PerFaceData {
            colour: Color::hex("1FAAFF").unwrap().as_linear_rgba_f32(),
            reflectance: 0.9,
            roughness: 0.95,
            metallic: 0.2,
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
    pub fn temp(x: f32) -> Self {
        if x < -0.5 {
            Low
        } else if x < 0.5 {
            Mid
        } else {
            High
        }
    }

    pub fn metal(x: f32) -> Self {
        if x > 0.75 {
            High
        } else {
            Mid
        }
    }

    pub fn land(x: f32) -> Self {
        if x < 0.25 {
            Low
        } else {
            High
        }
    }
}

impl Biome {
    pub fn new_base(temperature: f32, metal: f32) -> Self {
        match (Measure::temp(temperature), Measure::metal(metal)) {
            (High, _) => Lava,
            (_, High) => Metal,
            (Low, _) => Ice,
            _ => Dirt,
        }
    }
    pub fn new_sky(land: f32) -> Self {
        match Measure::land(land) {
            Low => Empty,
            _ => Cloud,
        }
    }
    pub fn new_space(land: f32) -> Self {
        match Measure::land(land) {
            Low => Empty,
            _ => Asteroid,
        }
    }
}
