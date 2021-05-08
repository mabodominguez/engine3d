use crate::voxel::*;
//use crate::particle::Particle;
//use crate::camera::Camera;
//use crate::camera_control::CameraController;
//use crate::collision::Contacts;
use cgmath::prelude::*;
pub type Pos3 = cgmath::Point3<f32>;
pub type Pos2 = cgmath::Point2<f32>;
pub type Mat4 = cgmath::Matrix4<f32>;

pub struct Game {
    camera_pos: Pos3,
    chunks: Vec<Chunk>,
}
