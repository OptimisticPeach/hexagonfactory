use bevy::math::Vec3A;
use simdeez::sse41::*;
use arrayvec::ArrayVec;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct NoiseParameters {
    pub scale: f32,
    pub lac: f32,
    pub bias: f32,
    pub gain: f32,
    pub seed: i32,
}

fn align_buffer(buf: &mut Vec<f32>, lanes: usize) -> &mut [f32] {
    (0..(lanes - (buf.len() % lanes))).for_each(|_| buf.push(0.0));

    let ptr = buf.as_ptr() as usize;
    let width = lanes * 4;
    let need = (ptr % width) / 4;

    (0..need).for_each(|_| buf.insert(0, 0.0));

    &mut buf[need..]
}

fn generate_cube_inputs<S: Simd>(size: usize) -> (Vec<S::Vf32>, Vec<S::Vf32>, Vec<S::Vf32>) {
    let inputs = size * size * 6;

    let sides = [
        //x+
        (-Vec3A::Z, -Vec3A::Y, Vec3A::X),
        //x-
        (Vec3A::Z, -Vec3A::Y, -Vec3A::X),
        //y+
        (Vec3A::X, Vec3A::Z, Vec3A::Y),
        //y-
        (Vec3A::X, -Vec3A::Z, -Vec3A::Y),
        //z+
        (Vec3A::X, -Vec3A::Y, Vec3A::Z),
        //z-
        (-Vec3A::X, -Vec3A::Y, -Vec3A::Z),
    ];

    let mut xs = vec![vec![]; 6];
    let mut ys = vec![vec![]; 6];
    let mut zs = vec![vec![]; 6];

    rayon::scope(|scope| {
        let mut xs = &mut xs[..];
        let mut ys = &mut ys[..];
        let mut zs = &mut zs[..];

        let interval = inputs / 6;

        for i in 0..6 {
            let (xf, xr) = xs.split_first_mut().unwrap();
            xs = xr;
            let (yf, yr) = ys.split_first_mut().unwrap();
            ys = yr;
            let (zf, zr) = zs.split_first_mut().unwrap();
            zs = zr;

            let range = (i * interval)..((i + 1) * interval);

            scope.spawn(move |_| {
                for i in range {
                    let x = i % size;
                    let y = i / size;

                    let fi = x as f32 / size as f32;
                    let fi = 2.0 * fi - 1.0;
                    let fj = (y % size) as f32 / size as f32;
                    let fj = 2.0 * fj - 1.0;
                    let index = (y / size) as usize;
                    let (x, y, depth) = sides[index];

                    let mut input_vector = depth + fi * x + fj * y;
                    input_vector = input_vector.normalize();

                    // input_vector = input_vector * 0.5 + Vec3A::ONE * 0.5;

                    xf.push(input_vector.x);
                    yf.push(input_vector.y);
                    zf.push(input_vector.z);
                }
            });
        }
    });

    let mut xs_f32 = xs.into_iter().flatten().collect();
    let mut ys_f32 = ys.into_iter().flatten().collect();
    let mut zs_f32 = zs.into_iter().flatten().collect();

    let xs_f32 = align_buffer(&mut xs_f32, S::VF32_WIDTH);
    let ys_f32 = align_buffer(&mut ys_f32, S::VF32_WIDTH);
    let zs_f32 = align_buffer(&mut zs_f32, S::VF32_WIDTH);

    // sanity checks
    assert_eq!(xs_f32.len() % S::VF32_WIDTH, 0);
    assert_eq!(ys_f32.len() % S::VF32_WIDTH, 0);
    assert_eq!(zs_f32.len() % S::VF32_WIDTH, 0);

    assert_eq!(xs_f32.as_ptr() as usize % (S::VF32_WIDTH * 4), 0);
    assert_eq!(ys_f32.as_ptr() as usize % (S::VF32_WIDTH * 4), 0);
    assert_eq!(zs_f32.as_ptr() as usize % (S::VF32_WIDTH * 4), 0);

    assert!(xs_f32.len() == ys_f32.len() && xs_f32.len() == zs_f32.len());

    let mut xs = Vec::with_capacity(xs_f32.len() / S::VF32_WIDTH);
    let mut ys = Vec::with_capacity(ys_f32.len() / S::VF32_WIDTH);
    let mut zs = Vec::with_capacity(zs_f32.len() / S::VF32_WIDTH);

    xs_f32
        .chunks(S::VF32_WIDTH)
        .for_each(|x| {
            unsafe {
                xs.push(S::load_ps(&x[0]));
            }
        });

    ys_f32
        .chunks(S::VF32_WIDTH)
        .for_each(|x| {
            unsafe {
                ys.push(S::load_ps(&x[0]));
            }
        });

    zs_f32
        .chunks(S::VF32_WIDTH)
        .for_each(|x| {
            unsafe {
                zs.push(S::load_ps(&x[0]));
            }
        });

    (xs, ys, zs)
}

fn generate_inputs<S: Simd>(vectors: &[Vec3A]) -> (Vec<S::Vf32>, Vec<S::Vf32>, Vec<S::Vf32>) {
    let mut xs_f32 = vectors.iter().map(|w| w.x).collect::<Vec<_>>();
    let mut ys_f32 = vectors.iter().map(|w| w.y).collect::<Vec<_>>();
    let mut zs_f32 = vectors.iter().map(|w| w.z).collect::<Vec<_>>();

    let xs_f32 = align_buffer(&mut xs_f32, S::VF32_WIDTH);
    let ys_f32 = align_buffer(&mut ys_f32, S::VF32_WIDTH);
    let zs_f32 = align_buffer(&mut zs_f32, S::VF32_WIDTH);

    // sanity checks
    assert_eq!(xs_f32.len() % S::VF32_WIDTH, 0);
    assert_eq!(ys_f32.len() % S::VF32_WIDTH, 0);
    assert_eq!(zs_f32.len() % S::VF32_WIDTH, 0);

    assert_eq!(xs_f32.as_ptr() as usize % (S::VF32_WIDTH * 4), 0);
    assert_eq!(ys_f32.as_ptr() as usize % (S::VF32_WIDTH * 4), 0);
    assert_eq!(zs_f32.as_ptr() as usize % (S::VF32_WIDTH * 4), 0);

    assert!(xs_f32.len() == ys_f32.len() && xs_f32.len() == zs_f32.len());

    let mut xs = Vec::with_capacity(xs_f32.len() / S::VF32_WIDTH);
    let mut ys = Vec::with_capacity(ys_f32.len() / S::VF32_WIDTH);
    let mut zs = Vec::with_capacity(zs_f32.len() / S::VF32_WIDTH);

    xs_f32.chunks(S::VF32_WIDTH).for_each(|x| unsafe {
        xs.push(S::load_ps(&x[0]));
    });

    ys_f32.chunks(S::VF32_WIDTH).for_each(|x| unsafe {
        ys.push(S::load_ps(&x[0]));
    });

    zs_f32.chunks(S::VF32_WIDTH).for_each(|x| unsafe {
        zs.push(S::load_ps(&x[0]));
    });

    (xs, ys, zs)
}

unsafe fn generate_many_array_values<S: Simd, const N: usize, const U: usize>(value: f32) -> [[S::Vf32; N]; U] {
    let val = S::set1_ps(value);
    let val = (0..N)
        .map(|_| val)
        .collect::<ArrayVec<S::Vf32, { N }>>()
        .into_inner()
        .unwrap();
    (0..U)
        .map(|_| val)
        .collect::<ArrayVec<[S::Vf32; N], { U }>>()
        .into_inner()
        .unwrap()
}

unsafe fn generate_noise<S: Simd, const N: usize>(
    xs: Vec<S::Vf32>,
    ys: Vec<S::Vf32>,
    zs: Vec<S::Vf32>,
    parameters: [NoiseParameters; N],
) -> Vec<[f32; N]> {
    let mut outputs = vec![vec![]; 6];

    let mut mins = generate_many_array_values::<S, { N }, 6>(f32::INFINITY);
    let mut maxs = generate_many_array_values::<S, { N }, 6>(f32::NEG_INFINITY);

    rayon::scope(|scope| {
        let inc = xs.len() / 6;
        let ranges = (0..5)
            .map(|i| (i * inc)..((i + 1) * inc))
            .chain(std::iter::once((5 * inc)..xs.len()))
            .collect::<Vec<_>>();

        for (
            (range, output),
            (mins, maxs)
        ) in ranges
            .into_iter()
            .zip(outputs.iter_mut())
            .zip(
                mins
                    .iter_mut()
                    .zip(maxs.iter_mut())
            ) {
            let range = range;
            let xs = &xs[range.clone()];
            let ys = &ys[range.clone()];
            let zs = &zs[range.clone()];
            let parameters = &parameters;

            (scope).spawn(move |_| {
                let output = output;
                let mins = mins;
                let maxs = maxs;
                let mut results = ArrayVec::<S::Vf32, { N }>::new();
                for i in 0..xs.len() {
                    for result in 0..N {
                        let value = unsafe {
                            let scale = S::set1_ps(parameters[result].scale);
                            let lac = S::set1_ps(parameters[result].lac);
                            let gain = S::set1_ps(parameters[result].gain);
                            let seed = parameters[result].seed;

                            let x = S::mul_ps(scale, xs[i]);
                            let y = S::mul_ps(scale, ys[i]);
                            let z = S::mul_ps(scale, zs[i]);

                            let value: S::Vf32 =
                                simdnoise::simplex::fbm_3d::<S>(x, y, z, lac, gain, 10, seed);

                            value
                        };

                        unsafe {
                            mins[result] = S::min_ps(value, mins[result]);
                            maxs[result] = S::max_ps(value, maxs[result]);
                        }

                        results.push(value);
                    }
                    output.push(results.clone().into_inner().unwrap());
                    results.clear();
                }
            });
        }
    });

    let mut min = [f32::INFINITY; N];
    let mut max = [f32::NEG_INFINITY; N];

    for per_thread in 0..6 {
        for per_lane in 0..S::VF32_WIDTH {
            for per_param in 0..N {
                min[per_param] = min[per_param].min(mins[per_thread][per_param][per_lane]);
                max[per_param] = max[per_param].max(maxs[per_thread][per_param][per_lane]);
            }
        }
    }

    let factors = min.iter().zip(max.iter())
        .map(|(min, max)| 2.0 / (*max - *min))
        .collect::<ArrayVec<f32, N>>()
        .into_inner()
        .unwrap();

    let end_mins = min
        .iter()
        .copied()
        .map(|x| unsafe { S::set1_ps(x) })
        .collect::<ArrayVec<_, N>>()
        .into_inner()
        .unwrap();

    let end_factors = factors
        .iter()
        .copied()
        .map(|x| unsafe { S::set1_ps(x) })
        .collect::<ArrayVec<_, N>>()
        .into_inner()
        .unwrap();

    let one = S::set1_ps(1.0);

    let biases = parameters
        .iter()
        .map(|x| unsafe { S::set1_ps(x.bias) })
        .collect::<ArrayVec<_, N>>()
        .into_inner()
        .unwrap();

    outputs
        .into_iter()
        .flatten()
        .map(|mut x: [S::Vf32; N]| {
            for i in 0..N {
                x[i] = unsafe { S::sub_ps(x[i], end_mins[i]) };
                x[i] = unsafe { S::mul_ps(x[i], end_factors[i]) };
                x[i] = unsafe { S::sub_ps(x[i], one) };
                x[i] = unsafe { S::mul_ps(x[i], biases[i]) };
            }
            (0..S::VF32_WIDTH)
                .map(move |lane| {
                    (0..N)
                        .map(|param| x[param][lane])
                        .collect::<ArrayVec<_, N>>()
                        .into_inner()
                        .unwrap()
                })
        })
        .flatten()
        .collect::<Vec<_>>()
}

unsafe fn make_noise<S: Simd, const N: usize>(
    data: &[Vec3A],
    parameters: [NoiseParameters; N],
) -> Vec<[f32; N]> {
    let (x, y, z) = generate_inputs::<S>(data);
    generate_noise::<S, { N }>(x, y, z, parameters)
}

#[cfg(target_feature = "avx2")]
fn make_noise_compiletime<const N: usize>(
    data: &[Vec3A],
    parameters: [NoiseParameters; N],
) -> Vec<[f32; N]> {
    unsafe { make_noise::<Avx2, { N }>(data, parameters) }
}

#[cfg(all(target_feature = "sse4.1", not(target_feature = "avx2")))]
fn make_noise_compiletime<const N: usize>(
    data: &[Vec3A],
    parameters: [NoiseParameters; N],
) -> Vec<[f32; N]> {
    unsafe { make_noise::<Sse41, { N }>(data, parameters) }
}

#[cfg(all(
target_feature = "sse2",
not(any(target_feature = "sse4.1", target_feature = "avx2"))
))]
fn make_noise_compiletime<const N: usize>(
    data: &[Vec3A],
    parameters: [NoiseParameters; N],
) -> Vec<[f32; N]> {
    unsafe { make_noise::<Sse2, { N }>(data, parameters) }
}

#[cfg(not(any(
target_feature = "sse4.1",
target_feature = "avx2",
target_feature = "sse2"
)))]
fn make_noise_compiletime<const N: usize>(
    data: &[Vec3A],
    parameters: [NoiseParameters; N],
) -> Vec<[f32; N]> {
    unsafe { make_noise::<Scalar, { N }>(data, parameters) }
}

pub fn sample_all_noise<const N: usize>(
    inputs: &[Vec3A],
    parameters: [NoiseParameters; N],
) -> Vec<[f32; N]> {
    make_noise_compiletime(inputs, parameters)
}


unsafe fn make_cube_noise<S: Simd, const N: usize>(
    size: usize,
    parameters: [NoiseParameters; N],
) -> Vec<[f32; N]> {
    let (x, y, z) = generate_cube_inputs::<S>(size);
    generate_noise::<S, { N }>(x, y, z, parameters)
}

#[cfg(target_feature = "avx2")]
fn make_cube_noise_compiletime<const N: usize>(
    size: usize,
    parameters: [NoiseParameters; N],
) -> Vec<[f32; N]> {
    unsafe { make_cube_noise::<Avx2, { N }>(size, parameters) }
}

#[cfg(all(target_feature = "sse4.1", not(target_feature = "avx2")))]
fn make_cube_noise_compiletime<const N: usize>(
    size: usize,
    parameters: [NoiseParameters; N],
) -> Vec<[f32; N]> {
    unsafe { make_cube_noise::<Sse41, { N }>(size, parameters) }
}

#[cfg(all(
target_feature = "sse2",
not(any(target_feature = "sse4.1", target_feature = "avx2"))
))]
fn make_cube_noise_compiletime<const N: usize>(
    size: usize,
    parameters: [NoiseParameters; N],
) -> Vec<[f32; N]> {
    unsafe { make_cube_noise::<Sse2, { N }>(size, parameters) }
}

#[cfg(not(any(
target_feature = "sse4.1",
target_feature = "avx2",
target_feature = "sse2"
)))]
fn make_cube_noise_compiletime<const N: usize>(
    size: usize,
    parameters: [NoiseParameters; N],
) -> Vec<[f32; N]> {
    unsafe { make_cube_noise::<Scalar, { N }>(size, parameters) }
}

pub fn sample_cube_noise<const N: usize>(
    size: usize,
    parameters: [NoiseParameters; N],
) -> Vec<[f32; N]> {
    make_cube_noise_compiletime(size, parameters)
}
