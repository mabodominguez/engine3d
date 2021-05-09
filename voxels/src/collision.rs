use crate::particle::Particle;
use crate::geom::*;

#[derive(Clone, Copy, Debug)]
pub struct Contact<T: Copy> {
    pub a: T,
    pub b: T,
    pub mtv: Vec3,
}

pub struct Contacts {
    pub block_player: Vec<Contact<usize>>,
    pub particle_block: Vec<Contact<usize>>,
}

impl Contacts {
    pub fn new() -> Self {
        Self {
            block_player: vec![],
            particle_block: vec![],
        }
    }
    fn sort(&mut self) {
        self.block_player
            .sort_unstable_by(|a, b| b.mtv.magnitude2().partial_cmp(&a.mtv.magnitude2()).unwrap());
        self.particle_block
            .sort_unstable_by(|a, b| b.mtv.magnitude2().partial_cmp(&a.mtv.magnitude2()).unwrap());
    }
    fn clear(&mut self) {
        self.block_player.clear();
        self.particle_block.clear();
    }
}

// fn restitute(voxels: &[Voxel], player: &mut BBox, particles: &mut [Particle], contacts: &mut Contacts) {
//     contacts.sort();
//     for c in contacts.block_player.iter_mut() {
//         if let Some(disp) = disp_box_box(&voxels[c.a].get_bbox(), player) {
//             let x = c.mtv.x;
//             let y = c.mtv.y;
//             let z = c.mtv.z;
//             if y <= x && y <= z {
//                 player.center.y += y / 2.0;
//                 c.mtv.x = 0.0;
//                 c.mtv.z = 0.0;
//             } else if x <= y && x <= z {
//                 player.center.x += x / 2.0;
//                 c.mtv.y = 0.0;
//                 c.mtv.z = 0.0;
//             } else {
//                 player.center.z += z / 2.0;
//                 c.mtv.x = 0.0;
//                 c.mtv.y = 0.0;
//             }
//         }
        
//     }
//     for c in contacts.particle_block.iter() {
//         //todo
//     }
// }

// pub fn update(voxels: &[Voxel], player: &mut BBox, particles: &mut [Particle], contacts: &mut Contacts) {
//     contacts.clear();
//     gather_contacts1(voxels, player, contacts);
//     restitute(voxels, player, particles, contacts);
// }

// fn gather_contacts1(voxels: &[Voxel], player: &mut BBox, into: &mut Contacts) { //probably check per chunk first
//     // collide player against voxels
//     for (ai, a) in voxels.iter().enumerate() {
//         if let Some(disp) = disp_box_box(&a.get_bbox(), player) {
//             into.block_player.push(Contact {
//                 a: ai,
//                 b: 0,
//                 mtv: disp,
//             });
//         }
//     }
// }