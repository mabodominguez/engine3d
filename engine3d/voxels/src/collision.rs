use crate::Voxel;
use crate::Particle;
use crate::geom::*;

#[derive(Clone, Copy, Debug)]
pub struct Contact<T: Copy> {
    pub a: T,
    pub b: T,
    pub mtv: Vec3,
}

pub struct Contacts {
    pub player_block: Vec<Contact<usize>>,
    pub particle_block: Vec<Contact<usize>>,
}

impl Contacts {
    pub fn new() -> Self {
        Self {
            player_block: vec![],
            particle_block: vec![],
        }
    }
    fn sort(&mut self) {
        self.player_block
            .sort_unstable_by(|a, b| b.mtv.magnitude2().partial_cmp(&a.mtv.magnitude2()).unwrap());
        self.particle_block
            .sort_unstable_by(|a, b| b.mtv.magnitude2().partial_cmp(&a.mtv.magnitude2()).unwrap());
    }
    fn clear(&mut self) {
        self.player_block.clear();
        self.particle_block.clear();
    }
}

fn restitute(voxels: &[Voxel], player: &mut BBox, particles: &mut [Particle], contacts: &mut Contacts) {
    contacts.sort();
    for c in contacts.player_block.iter_mut() {
        let x = c.mtv.x;
        let y = c.mtv.y;
        let z = c.mtv.z;
        if (y <= x && y <= z) {
            player.center.y += y;
            c.mtv.x = 0.0;
            c.mtv.z = 0.0;
        } else if (x <= y && x <= z) {
            player.center.x += x;
            c.mtv.y = 0.0;
            c.mtv.z = 0.0;
        } else {
            player.center.z += z;
            c.mtv.x = 0.0;
            c.mtv.y = 0.0;
        }
    }
    for c in contacts.particle_block.iter() {
        //todo
    }
}

pub fn update(voxels: &[Voxel], player: &mut BBox, particles: &mut [Particle], contacts: &mut Contacts) {
    contacts.clear();
    gather_contacts1(voxels, player, contacts);
    restitute(voxels, player, particles, contacts);
}

fn gather_contacts1(voxels: &[Voxel], player: &mut BBox, into: &mut Contacts) { //probably check per chunk first
    // collide player against voxels
    for (ai, a) in voxels.iter().enumerate() {
        if let Some(disp) = disp_box_box(&a.get_bbox(), player) {
            into.player_block.push(Contact {
                a: ai,
                b: 0,
                mtv: disp,
            });
        }
    }
}