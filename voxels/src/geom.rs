pub use cgmath::prelude::*;
pub type Vec3 = cgmath::Vector3<f32>;
pub type Pos3 = cgmath::Point3<f32>;
pub type Pos2 = cgmath::Point2<f32>;
pub type Mat4 = cgmath::Matrix4<f32>;

pub fn dist_3d(p1: Pos3, p2: Pos3) -> f32{
    return ((p1.x - p2.x).powf(2.0) + (p1.y - p2.y).powf(2.0) + (p1.z - p2.z).powf(2.0)).powf(0.5);
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Sphere {
    pub c: Pos3,
    pub r: f32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Plane {
    pub n: Vec3,
    pub d: f32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BBox {
    pub center: Pos3,
    pub halfwidth: f32, // Width, height, depth
}

pub fn touching_box_box(b1: &BBox, b2: &BBox) -> bool {
    if (b1.halfwidth + b2.halfwidth) < (b1.center.x - b2.center.x).abs() {
        return false;
    }
    if (b1.halfwidth + b2.halfwidth) < (b1.center.y - b2.center.y).abs() {
        return false;
    }
    if (b1.halfwidth + b2.halfwidth).abs() < (b1.center.z - b2.center.z).abs() {
        return false;
    }
    return true;
}

pub fn touching_box_sphere(b: &BBox, s: &Sphere) -> bool {
    if s.c.distance2(b.center) <= (s.r + b.halfwidth).powi(2) {
        return true;
    } else if s.c.distance2(b.center) <= (s.r + b.halfwidth).powi(2) {
        return true;
    } else if s.c.distance2(b.center) <= (s.r + b.halfwidth).powi(2) {
        return true;
    } else {
        return true;
    }
}

pub fn disp_box_box(b1: &BBox, b2: &BBox) -> Option<Vec3> {
    let mut x_overlap = 0.0;
    let mut y_overlap = 0.0;
    let mut z_overlap = 0.0;
    if !((b1.halfwidth + b2.halfwidth) < (b1.center.x - b2.center.x).abs()) {
        x_overlap = (b1.halfwidth + b2.halfwidth) - (b1.center.x - b2.center.x).abs();
        if b1.center.x < b2.center.x {
            x_overlap = x_overlap * -1.0;
        }
    }
    if !((b1.halfwidth + b2.halfwidth) < (b1.center.y - b2.center.y).abs()) {
        y_overlap = (b1.halfwidth + b2.halfwidth) - (b1.center.y - b2.center.y).abs();
        if b1.center.y < b2.center.y {
            y_overlap = y_overlap * -1.0;
        }
    }
    if !((b1.halfwidth + b2.halfwidth).abs() < (b1.center.z - b2.center.z).abs()) {
        z_overlap = (b1.halfwidth + b2.halfwidth) - (b1.center.z - b2.center.z).abs();
        if b1.center.z < b2.center.z {
            z_overlap = z_overlap * -1.0;
        }
    }

    // Get Disp
    if x_overlap == 0.0 || y_overlap == 0.0 || z_overlap == 0.0 {
        None
    } else {
        Some(Vec3::new(x_overlap / 2.0, y_overlap / 2.0, z_overlap / 2.0))
    }
}

pub fn disp_box_sphere(b: &BBox, s: &Sphere) -> Option<Vec3> {
    let mut x_overlap = 0.0;
    let mut y_overlap = 0.0;
    let mut z_overlap = 0.0;
    if s.c.distance2(b.center) <= (s.r + b.halfwidth).powi(2) {
        x_overlap = (s.r + b.halfwidth) - (s.c.x - b.center.x).abs()
    }
    if s.c.distance2(b.center) <= (s.r + b.halfwidth).powi(2) {
        y_overlap = (s.r + b.halfwidth) - (s.c.y - b.center.y).abs()
    }
    if s.c.distance2(b.center) <= (s.r + b.halfwidth).powi(2) {
        z_overlap = (s.r + b.halfwidth) - (s.c.z - b.center.z).abs()
    }
    if x_overlap == 0.0 && y_overlap == 0.0 && z_overlap == 0.0 {
        None
    } else {
        Some(Vec3::new(x_overlap / 2.0, y_overlap / 2.0, z_overlap / 2.0)) // SOMETHING FUNNY IS GOING ON (DIVIDE BY 2 MAYBE?)
    }
}

/// Are s1 and s2 touching?
pub fn touching_sphere_sphere(s1: &Sphere, s2: &Sphere) -> bool {
    // Is the (squared) distance between the centers less than the
    // (squared) sum of the radii?
    s2.c.distance2(s1.c) <= (s1.r + s2.r).powi(2)
}
/// What's the offset I'd need to push s1 and s2 out of each other?
pub fn disp_sphere_sphere(s1: &Sphere, s2: &Sphere) -> Option<Vec3> {
    let offset = s2.c - s1.c;
    let distance = offset.magnitude();
    if distance < s1.r + s2.r {
        // Make sure we don't divide by 0
        let distance = if distance == 0.0 { 1.0 } else { distance };
        // How much combined radius is "left over"?
        let disp_mag = (s1.r + s2.r) - distance;
        // Normalize offset and multiply by the amount to push
        Some(offset * (disp_mag / distance))
    } else {
        None
    }
}

pub fn touching_sphere_plane(s: &Sphere, p: &Plane) -> bool {
    // Find the distance of the sphere's center to the plane
    (s.c.dot(p.n) - p.d).abs() <= s.r
}
pub fn disp_sphere_plane(s: &Sphere, p: &Plane) -> Option<Vec3> {
    // Find the distance of the sphere's center to the plane
    let dist = s.c.dot(p.n) - p.d;
    if dist.abs() <= s.r {
        // If we offset from the sphere position opposite the normal,
        // we'll end up hitting the plane at `dist` units away.  So
        // the displacement is just the plane's normal * dist.
        Some(p.n * (s.r - dist))
    } else {
        None
    }
}

pub fn disp_box_plane(b: &BBox, p: &Plane) -> Option<Vec3> {
    // Find the distance of the sphere's center to the plane
    let dist = b.center.dot(p.n) - p.d;
    if dist.abs() <= b.halfwidth {
        // If we offset from the sphere position opposite the normal,
        // we'll end up hitting the plane at `dist` units away.  So
        // the displacement is just the plane's normal * dist.
        return Some(p.n * (b.halfwidth - dist) * 1.03);
    } else {
        None
    }
}
