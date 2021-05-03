use crate::geom::*;
use crate::{Cube, Marble, Wall}; //something happening here

#[derive(Clone, Copy, Debug)]
pub struct Contact<T: Copy> {
    pub a: T,
    pub b: T,
    pub mtv: Vec3,
}

pub struct Contacts {
    pub wm: Vec<Contact<usize>>,
    pub mm: Vec<Contact<usize>>,
    pub cm: Vec<Contact<usize>>,
    pub cc: Vec<Contact<usize>>,
    pub wc: Vec<Contact<usize>>,
}

impl Contacts {
    pub fn new() -> Self {
        Self {
            wm: vec![],
            mm: vec![],
            cm: vec![],
            cc: vec![],
            wc: vec![],
        }
    }
    fn sort(&mut self) {
        self.wm
            .sort_unstable_by(|a, b| b.mtv.magnitude2().partial_cmp(&a.mtv.magnitude2()).unwrap());
        self.mm
            .sort_unstable_by(|a, b| b.mtv.magnitude2().partial_cmp(&a.mtv.magnitude2()).unwrap());
    }
    fn clear(&mut self) {
        self.wm.clear();
        self.mm.clear();
    }
}

fn restitute(walls: &[Wall], marbles: &mut [Marble], cubes: &mut [Cube], contacts: &mut Contacts) {
    contacts.sort();
    // Lots of marbles on the floor...
    for c in contacts.wm.iter() {
        let a = c.a;
        let b = c.b;
        // Are they still touching?  This way we don't need to track disps or anything
        // at the expense of some extra collision checks
        if let Some(disp) = disp_sphere_plane(&marbles[a].body, &walls[b].body) {
            // We can imagine we're instantaneously applying a
            // velocity change to pop the object just above the floor.
            marbles[a].body.c += disp;
            // It feels a little weird to be adding displacement (in
            // units) to velocity (in units/frame), but we'll roll
            // with it.  We're not exactly modeling a normal force
            // here but it's something like that.
            marbles[a].velocity += disp;
        }
    }

    for c in contacts.wc.iter() {
        let a = c.a;
        let b = c.b;
        // Are they still touching?  This way we don't need to track disps or anything
        // at the expense of some extra collision checks
        if let Some(disp) = disp_box_plane(&cubes[a].body, &walls[b].body) {
            // We can imagine we're instantaneously applying a
            // velocity change to pop the object just above the floor.
            cubes[a].body.center += disp;
            // It feels a little weird to be adding displacement (in
            // units) to velocity (in units/frame), but we'll roll
            // with it.  We're not exactly modeling a normal force
            // here but it's something like that.
            cubes[a].velocity += disp;
        }
    }
    // That can bump into each other in perfectly elastic collisions!
    for c in contacts.mm.iter() {
        let a = c.a;
        let b = c.b;
        // Just split the difference.  In crowded situations this will
        // cause issues, but those will always be hard to solve with
        // this kind of technique.
        if let Some(disp) = disp_sphere_sphere(&marbles[a].body, &marbles[b].body) {
            let a_mass = marbles[a].mass / (marbles[a].mass + marbles[b].mass);
            let b_mass = 1 as f32 - a_mass;
            marbles[a].body.c -= disp / (2.0 * a_mass);
            marbles[a].velocity -= disp / (2.0 * a_mass);
            marbles[b].body.c += disp / (2.0 * b_mass);
            marbles[b].velocity += disp / (2.0 * b_mass);
        }
    }

    // // That can bump into each other in perfectly elastic collisions!
    for c in contacts.cm.iter() {
        let a = c.a;
        let b = c.b;
        // Just split the difference.  In crowded situations this will
        // cause issues, but those will always be hard to solve with
        // this kind of technique.
        if let Some(disp) = disp_box_sphere(&cubes[a].body, &marbles[b].body) {
            let a_mass = cubes[a].mass / (cubes[a].mass + marbles[b].mass);
            let b_mass = 1 as f32 - a_mass;
            cubes[a].body.center -= disp / (2.0 * a_mass);
            cubes[a].velocity -= disp / (2.0 * a_mass);
            marbles[b].body.c += disp / (2.0 * b_mass);
            marbles[b].velocity += disp / (2.0 * b_mass);
        }
    }

    for c in contacts.cc.iter() {
        let a = c.a;
        let b = c.b;
        // Just split the difference.  In crowded situations this will
        // cause issues, but those will always be hard to solve with
        // this kind of technique.
        if let Some(disp) = disp_box_box(&cubes[a].body, &cubes[b].body) {
            let a_mass = cubes[a].mass / (cubes[a].mass + cubes[b].mass);
            let b_mass = 1 as f32 - a_mass;
            cubes[a].body.center -= disp / (2.0 * a_mass);
            cubes[a].velocity -= disp / (2.0 * a_mass);
            cubes[b].body.center += disp / (2.0 * b_mass);
            cubes[b].velocity += disp / (2.0 * b_mass);
        }
    }

    // That can bump into each other in perfectly elastic collisions!
    for c in contacts.mm.iter() {
        let a = c.a;
        let b = c.b;
        // Just split the difference.  In crowded situations this will
        // cause issues, but those will always be hard to solve with
        // this kind of technique.
        if let Some(disp) = disp_sphere_sphere(&marbles[a].body, &marbles[b].body) {
            let a_mass = marbles[a].mass / (marbles[a].mass + marbles[b].mass);
            let b_mass = 1 as f32 - a_mass;
            marbles[a].body.c -= disp / (2.0 * a_mass);
            marbles[a].velocity -= disp / (2.0 * a_mass);
            marbles[b].body.c += disp / (2.0 * b_mass);
            marbles[b].velocity += disp / (2.0 * b_mass);
        }
    }
}

pub fn update(walls: &[Wall], marbles: &mut [Marble], cubes: &mut [Cube], contacts: &mut Contacts) {
    contacts.clear();
    gather_contacts1(walls, marbles, contacts);
    gather_contacts2(walls, cubes, contacts);
    gather_contacts3(cubes, marbles, contacts);
    restitute(walls, marbles, cubes, contacts);
}

fn gather_contacts1(statics: &[Wall], dynamics: &[Marble], into: &mut Contacts) {
    // collide mobiles against mobiles
    for (ai, a) in dynamics.iter().enumerate() {
        for (bi, b) in dynamics[(ai + 1)..].iter().enumerate() {
            let bi = ai + 1 + bi;
            if let Some(disp) = disp_sphere_sphere(&a.body, &b.body) {
                into.mm.push(Contact {
                    a: ai,
                    b: bi,
                    mtv: disp,
                });
            }
        }
    }
    // collide mobiles against walls
    for (bi, b) in statics.iter().enumerate() {
        for (ai, a) in dynamics.iter().enumerate() {
            if let Some(disp) = disp_sphere_plane(&a.body, &b.body) {
                into.wm.push(Contact {
                    a: ai,
                    b: bi,
                    mtv: disp,
                });
            }
        }
    }
}

fn gather_contacts2(statics: &[Wall], dynamics: &[Cube], into: &mut Contacts) {
    // collide mobiles against mobiles
    for (ai, a) in dynamics.iter().enumerate() {
        for (bi, b) in dynamics[(ai + 1)..].iter().enumerate() {
            let bi = ai + 1 + bi;
            if let Some(disp) = disp_box_box(&a.body, &b.body) {
                into.cc.push(Contact {
                    a: ai,
                    b: bi,
                    mtv: disp,
                });
            }
        }
    }
    //collide mobiles against walls
    for (bi, b) in statics.iter().enumerate() {
        for (ai, a) in dynamics.iter().enumerate() {
            if let Some(disp) = disp_box_plane(&a.body, &b.body) {
                into.wc.push(Contact {
                    a: ai,
                    b: bi,
                    mtv: disp,
                });
            }
        }
    }
}

fn gather_contacts3(cubes: &[Cube], marbles: &[Marble], into: &mut Contacts) {
    // collide mobiles against mobiles
    for (ai, a) in cubes.iter().enumerate() {
        for (bi, b) in marbles.iter().enumerate() {
            if let Some(disp) = disp_box_sphere(&a.body, &b.body) {
                into.cm.push(Contact {
                    a: ai,
                    b: bi,
                    mtv: disp,
                });
            }
        }
    }
}
