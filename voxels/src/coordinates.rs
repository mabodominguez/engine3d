use crate::voxel::*;
use crate::world_gen::WORLD_DIMS; // This is the dimensions of the world. I have yet to upload worldgen, so just redefine this constant

type Pos3 = cgmath::Point3<f32>;

pub fn world_to_chunk(coords: Pos3) -> (usize, (usize, usize, usize)) {
    //println!("X:{}, Y:{}, Z:{}", coords.x,coords.y,coords.z);
    let x = coords.x.floor() as i64;
    let y = coords.y.floor() as i64;
    let z = coords.z.floor() as i64;
    let chunk_scale = (CHUNK_SIZE as f32 * (VOXEL_HALFWIDTH * 2.0)) as i64;
    let vox_scale = (VOXEL_HALFWIDTH * 2.0) as i64;
    // X
    let world_x = (x / chunk_scale) as usize;
    let chunk_x = ((x % chunk_scale) / vox_scale) as usize;
    // Y
    let world_y = (y / chunk_scale) as usize;
    let chunk_y = ((y % chunk_scale) / vox_scale) as usize;
    // Z
    let world_z = (z / chunk_scale) as usize;
    let chunk_z = ((z % chunk_scale) / vox_scale) as usize;

    // Chunk index
    let mut i = world_x * WORLD_DIMS.1 * WORLD_DIMS.2;
    i += world_y * WORLD_DIMS.2;
    i += world_z;
    return (i, (chunk_x, chunk_y, chunk_z));
}

pub fn index_to_world(index: usize) -> (usize, usize, usize) {
    let x = index / (WORLD_DIMS.2 * WORLD_DIMS.1);
    let y = (index % (WORLD_DIMS.2 * WORLD_DIMS.1)) / WORLD_DIMS.2;
    let z = index % WORLD_DIMS.2;
    (x, y, z)
}
