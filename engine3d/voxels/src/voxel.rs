use crate::geom::{Mat4, Pos3, BBox};
use crate::InstanceRaw;
use cgmath::EuclideanSpace;
pub const VOXEL_HALFWIDTH: f32 = 1.0;  // Size of a voxel (halfwidth)
pub const CHUNK_SIZE: usize = 8; // Size of lenght, width, and height of a chunk

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Material { // Enumeration to determine the material of a voxel. Is useful for differntiating them
    Grass,
    Dirt,
    Iron,
}

impl Material {
    pub fn strength(&self) -> i32 { // Possibly useful function to determine how much time it takes to break a block
        match *self {
            Material::Grass => 1,
            Material::Dirt => 2,
            Material::Iron => 3,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Voxel { // A voxel holds position and material info
    pub center: Pos3,
    pub material: Material,
}

impl Voxel {
    pub fn to_raw(&self) -> InstanceRaw { // Turns vector position into gpu-friendly data
        InstanceRaw { 
            model: (Mat4::from_translation(self.center.to_vec()) * Mat4::from_scale(VOXEL_HALFWIDTH)).into(),
        }
    }
    pub fn get_bbox(&self) -> BBox {
        return BBox{center:self.center, halfwidth:VOXEL_HALFWIDTH};
    }
}

pub struct Chunk{ // Array that holds the vector info. It dimensions are CHUNK_SIZE^3 
    // Holds a position and the data (which is just numbers)
    pub origin: Pos3,
    pub data:  [[[usize; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]
}

// Function to create voxels from the info matrix in a chunk
pub fn voxels_from_chunk(chunk: & Chunk) -> Vec<Voxel>{
    let mut voxels: Vec<Voxel> = vec!();
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let x_pos = (x as f32 * VOXEL_HALFWIDTH/0.5 ) + chunk.origin.x;
                let y_pos = (y as f32 * VOXEL_HALFWIDTH/0.5 ) + chunk.origin.y;
                let z_pos = (z as f32 * VOXEL_HALFWIDTH/0.5 ) + chunk.origin.z;
                let material = match chunk.data[x][y][z] {
                    0 => Material::Dirt,
                    1 => Material::Iron,
                    _ => Material::Grass
                };
                voxels.push(
                    Voxel {
                        center: Pos3::new(x_pos, y_pos, z_pos),
                        material,
                    }
                )
            }
        }
    }
    return  voxels
}