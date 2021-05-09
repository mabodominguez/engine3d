use crate::voxel::*;
use rand;

extern crate noise;
use noise::{Add, NoiseFn, Perlin, Seedable, Turbulence};

pub const WORLD_DIMS: (usize, usize, usize) = (10, 5, 10); // The number of chunks that you want to load in 3D space
pub const WORLD_MAX: usize = WORLD_DIMS.0 * WORLD_DIMS.1 * WORLD_DIMS.2;
pub const RENDER_RADIUS: (usize, usize) = (1, 1);

const PERLIN_STEP_2D: f64 = 0.03;
const PERLIN_STEP_TOP: f64 = 0.15;
const PERLIN_STEP_MID: f64 = 0.07;

pub type Pos3 = cgmath::Point3<f32>;

pub fn make_world() -> Vec<Chunk> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    // Iterate through the world chunks, and in each chunk place a random voxel
    let mut chunks: Vec<Chunk> = Vec::new();

    let seed: u32 = rng.gen();
    let noise_1 = Perlin::new().set_seed(seed);
    let turbulence = Turbulence::new(noise_1);
    let perlin3 = Perlin::new().set_seed(seed);

    let perlin2: Add<[f64; 2]> = Add::new(&turbulence, &noise_1);

    for cx in 0..WORLD_DIMS.0 {
        for cy in 0..WORLD_DIMS.1 {
            for cz in 0..WORLD_DIMS.2 {
                if cy == 0 {
                    chunks.push(make_bottom_layer((cx as f64, cy as f64, cz as f64),
                    &perlin3,));
                }
                else if cy >= (WORLD_DIMS.1 - 1) {
                    chunks.push(make_air_layer());
                }
                else if cy >= (WORLD_DIMS.1 - 2) {
                    chunks.push(make_top_layer(
                        (cx as f64, cy as f64, cz as f64),
                        &perlin2,
                        &perlin3,
                    ));
                } else {
                    chunks.push(make_mid_layer( (cx as f64, cy as f64, cz as f64),  &perlin3));
                }
            }
        }
    }
    return chunks;
}

fn make_top_layer(
    (cx, cy, cz): (f64, f64, f64),
    noise: &Add<[f64; 2]>,
    noise_3_d: &Perlin,
) -> Chunk {

    // Array that we'll copy into chunks
    let mut data: [[[u8; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE] =
        [[[0; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

    for x in 0..CHUNK_SIZE as usize {
        let world_x = cx * (CHUNK_SIZE as f64) + (x as f64);
        for y in 0..CHUNK_SIZE as usize {
            let world_y = cy * (CHUNK_SIZE as f64) + (y as f64);
            for z in 0..CHUNK_SIZE as usize {
                // Get chunk coordinates into world coordinate
                let world_z = cz * (CHUNK_SIZE as f64) + (z as f64);

                // Get the height value from noise
                // Noise originally ranges from -2.0 to 2.0, so we adjust it to be between 0 and 1
                let noise_val =
                    (noise.get([world_x * PERLIN_STEP_2D, world_z * PERLIN_STEP_2D]) + 2.0) / 4.0;
                let height = (noise_val * CHUNK_SIZE as f64).floor()
                    + ((WORLD_DIMS.1 - 2) * CHUNK_SIZE) as f64;

                // If our coordinate is taller than the height, we place air
                // Otherwise we place a dirt block
                if world_y > height {
                    data[x][y][z] = 0;
                } else {
                    if world_y == height {
                        data[x][y][z] = 1;
                    } else {
                        data[x][y][z] = 2;
                        if world_y <= height - 3.0 {
                            let noise_val_3_d = (noise_3_d.get([
                                world_x * PERLIN_STEP_TOP,
                                world_y * PERLIN_STEP_TOP,
                                world_z * PERLIN_STEP_TOP,
                            ]) + 1.0)
                                / 2.0;
                            let material = (noise_val_3_d + 2.0).round() as u8;
                            data[x][y][z] = material;
                        }
                    }
                }
            }
        }
    }

    return Chunk { data, voxels:vec![] };
}


fn make_air_layer() -> Chunk{
    let mut data: [[[u8; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE] =
        [[[0; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                data[x][y][z] = 0;
            }
        }
    }
    Chunk { data, voxels:vec![]}
}


fn make_mid_layer(
    (cx, cy, cz): (f64, f64, f64),
    noise_3_d: &Perlin,
) -> Chunk{
    // Array that we'll copy into chunks
    let mut data: [[[u8; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE] =
        [[[0; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

    for x in 0..CHUNK_SIZE as usize {
        let world_x = cx * (CHUNK_SIZE as f64) + (x as f64);
        for y in 0..CHUNK_SIZE as usize {
            let world_y = cy * (CHUNK_SIZE as f64) + (y as f64);
            for z in 0..CHUNK_SIZE as usize {
                // Get chunk coordinates into world coordinate
                let world_z = cz * (CHUNK_SIZE as f64) + (z as f64);

                // Get noise
                // Noise adjusted to range from 0 - 1 
                let noise_val = (noise_3_d.get([
                    world_x * PERLIN_STEP_MID,
                    world_y * PERLIN_STEP_MID,
                    world_z * PERLIN_STEP_MID,
                ]) + 1.0) / 2.0;
                let material = mid_material(noise_val);
                data[x][y][z] = material;
            }
        }
    }
    Chunk { data , voxels:vec![]}
}



fn make_bottom_layer(
    (cx, cy, cz): (f64, f64, f64),
    noise_3_d: &Perlin,
) -> Chunk{
    // Array that we'll copy into chunks
    let mut data: [[[u8; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE] =
        [[[0; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

    for x in 0..CHUNK_SIZE as usize {
        let world_x = cx * (CHUNK_SIZE as f64) + (x as f64);
        for y in 0..CHUNK_SIZE as usize {
            let world_y = cy * (CHUNK_SIZE as f64) + (y as f64);
            for z in 0..CHUNK_SIZE as usize {
                // Get chunk coordinates into world coordinate
                let world_z = cz * (CHUNK_SIZE as f64) + (z as f64);

                // Get noise
                // Noise adjusted to range from 0 - 1 
                if y == 0{
                    data[x][y][z] = 7;
                } else {
                    let noise_val = (noise_3_d.get([
                        world_x * PERLIN_STEP_MID,
                        world_y * PERLIN_STEP_MID,
                        world_z * PERLIN_STEP_MID,
                    ]) + 1.0) / 2.0;
                    let material = bottom_material(noise_val);
                    data[x][y][z] = material;
                }
            }
        }
    }
    Chunk { data , voxels:vec![]}
}

// ROCK -> 3
// IRON -> 4 
// GOLD -> 5
// DIAMOND -> 6
// BEDROCK -> 7
fn mid_material(noise: f64) -> u8 {
    if noise <= 0.3 {
        3
    } else if noise <= 0.33 {
        4
    } else if noise <= 0.44 {
        3
    } else if noise <= 0.55 {
        0
    } else if noise <= 0.7 {
        3
    } else if noise <= 0.73 {
        5
    } else {
        3
    }
}

fn bottom_material(noise: f64) -> u8 {
    if noise <= 0.01 {
        7
    } else if noise <= 0.3 {
        3
    } else if noise <= 0.32 {
        4
    } else if noise <= 0.44 {
        3
    } else if noise <= 0.55 {
        0
    } else if noise <= 0.7 {
        3
    } else if noise <= 0.72 {
        5
    } else if noise <= 0.98 {
        3
    } else {
        7
    }
}