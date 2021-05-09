use crate::coordinates::*;
use crate::geom::*;
use crate::voxel::*;

pub fn collide_x(hitbox: BBox, chunks: &Vec<Chunk>, x: f32) -> bool {
    let hitbox_center = hitbox.center;
    let halfwidth = hitbox.halfwidth;
    let border_check = cgmath::point3(
        hitbox_center.x + halfwidth * x.signum() + x,
        hitbox_center.y,
        hitbox_center.z,
    );
    let (i, (x, y, z)) = world_to_chunk(border_check);
    chunks[i].data[x][y][z] != 0
}

pub fn collide_y(hitbox: BBox, chunks: &Vec<Chunk>, y: f32) -> bool {
    let hitbox_center = hitbox.center;
    let halfwidth = hitbox.halfwidth;
    let border_check = cgmath::point3(
        hitbox_center.x,
        hitbox_center.y + halfwidth * y.signum() + y,
        hitbox_center.z,
    );
    let (i, (x, y, z)) = world_to_chunk(border_check);
    chunks[i].data[x][y][z] != 0
}

pub fn collide_z(hitbox: BBox, chunks: &Vec<Chunk>, z: f32) -> bool {
    let hitbox_center = hitbox.center;
    let halfwidth = hitbox.halfwidth;
    let border_check = cgmath::point3(
        hitbox_center.x,
        hitbox_center.y,
        hitbox_center.z + halfwidth * z.signum() + z,
    );
    let (i, (x, y, z)) = world_to_chunk(border_check);
    chunks[i].data[x][y][z] != 0
}
