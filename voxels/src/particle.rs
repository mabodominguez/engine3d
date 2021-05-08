use crate::InstanceRaw;
use crate::geom::{Mat4, Vec3, Pos3, Sphere};
use super::{DT};
use cgmath::EuclideanSpace;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Particle {
    pub body: Sphere,
    pub velocity: Vec3,
    pub lifetime: f32,
    pub visible: bool,
}

impl Particle {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (Mat4::from_translation(self.body.c.to_vec()) * Mat4::from_scale(self.body.r))
                .into(),
        }
    }
    fn update(&mut self, g: f32) {
        if self.visible {
            self.velocity += Vec3::new(0.0, -g, 0.0) * DT;
            self.body.c += self.velocity * DT;
            self.lifetime -= DT;
        } else {
            self.body.c = Pos3{x:0.0, y: 0.0, z: 0.0};
        }
        
        if self.lifetime <= 0.0 {
            self.visible = false;
        }
    }

    fn reset(&mut self) {
        self.body.c = Pos3{x:10.0, y: 10.0, z: 10.0};
        self.body.r = 0.5;
        self.lifetime = 5.0;
        self.visible = true;
    }
}