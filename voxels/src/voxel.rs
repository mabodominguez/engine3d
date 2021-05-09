use crate::geom::*;
use crate::coordinates::*;
pub const VOXEL_HALFWIDTH: f32 = 2.0; // Size of a voxel (halfwidth)
pub const CHUNK_SIZE: usize = 16; // Size of lenght, width, and height of a chunk


pub struct Chunk {
    // Array that holds the vector info. It dimensions are CHUNK_SIZE^3
    pub data: [[[u8; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    pub voxels: Vec<BBox>,
}

impl Chunk {
    pub fn create_bboxes(&mut self, i:usize) {
        let (x,y,z) = index_to_world(i);
        let voxel_width = 2.0 * VOXEL_HALFWIDTH;
        for i in 0..CHUNK_SIZE {
            for j in 0..CHUNK_SIZE {
                for k in 0..CHUNK_SIZE {
                    let voxel_x = (x  + CHUNK_SIZE * i) as f32  + VOXEL_HALFWIDTH;
                    let voxel_y = (y  + CHUNK_SIZE * j) as f32 + VOXEL_HALFWIDTH;
                    let voxel_z = (z  + CHUNK_SIZE * k) as f32 + VOXEL_HALFWIDTH;
                    let voxel_pos = Pos3{x:voxel_x, y:voxel_y, z:voxel_z};
                    self.voxels.push(BBox{center:voxel_pos, halfwidth:VOXEL_HALFWIDTH});
                }
            }
        }
    }
    pub fn bboxes_generated(&self) -> bool {
        return self.voxels.len() > 0;
    }
    pub fn data_at(&self, v:usize) -> u8 {
        return self.data[v / (CHUNK_SIZE * CHUNK_SIZE)][(v % (CHUNK_SIZE * CHUNK_SIZE)) / (CHUNK_SIZE)][v % CHUNK_SIZE];
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    #[allow(dead_code)]
    pub model: [[f32; 4]; 4],
}

impl InstanceRaw {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We don't have to do this in code though.
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float4,
                },
            ],
        }
    }
}
