use image::{Rgb, ImageBuffer};
use glam::{Vec3A, Quat};

const SIZE: usize = 1000;
const SCALE: f32 = 20.0;
const BIAS: f32 = 0.5;
const SEED: i32 = 10;

fn main() {
    let mut img = ImageBuffer::new(SIZE as u32, 6 * SIZE as u32);

    let mut noise_values = noise_gen::sample_cube_noise(SIZE, [
        noise_gen::NoiseParameters {
            scale: SCALE,
            lac: 0.5,
            bias: BIAS,
            gain: 0.5,
            seed: SEED
        },
        noise_gen::NoiseParameters {
            scale: SCALE,
            lac: 0.5,
            bias: BIAS,
            gain: 0.5,
            seed: !SEED
        },
    ]);
    let noise_values = &mut noise_values[..SIZE * SIZE * 6];

    for (index, [ax, ay]) in noise_values.into_iter().enumerate() {
        let x = (index % SIZE) as u32;
        let y = (index / SIZE) as u32;

        let mut vector = (Quat::from_rotation_x(*ax) * Quat::from_rotation_y(*ay)).mul_vec3a(Vec3A::Z);

        vector *= 0.5;
        vector += Vec3A::splat(0.5);

        vector *= 255.0;
        let r = vector.x as u8;
        let g = vector.y as u8;
        let b = vector.z as u8;

        img.put_pixel(x, y, Rgb([r, g, b]));
    }

    img.save("assets/normal_map.png").unwrap();
}
