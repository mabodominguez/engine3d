use crate::geom::*;
use cgmath::*;
use crate::camera::Camera;
use crate::Events;
use winit::event::*;
use crate::collision::Contact;

pub struct Player {
    pub hitbox:BBox,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub facing_direction: Vec3,
    pub speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed:bool,
    is_down_pressed:bool,
    is_gravity_pressed:bool,
    do_gravity:bool,
    jump_pressed:bool,
    pub can_jump:bool,
    pub x_pos_blocked: bool,
    pub x_neg_blocked: bool,
    pub y_pos_blocked: bool,
    pub y_neg_blocked: bool,
    pub z_pos_blocked: bool,
    pub z_neg_blocked: bool,
}

impl Player {
    pub fn new(hitbox:BBox) -> Self {
        Self {
            hitbox: hitbox,
            vx: 0.0,
            vy: 0.0,
            vz: 0.0,
            facing_direction: cgmath::vec3(0.0, 0.0, 0.0),
            speed: 1.0,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed:false,
            is_down_pressed:false,
            is_gravity_pressed:false,
            do_gravity:true,
            jump_pressed: false,
            can_jump: false,
            x_pos_blocked: false,
            x_neg_blocked: false,
            y_pos_blocked: false,
            y_neg_blocked: false,
            z_pos_blocked: false,
            z_neg_blocked: false,
        }
    }
    pub fn get_pos(&self) -> Pos3 {
        return self.hitbox.center;
    }
    pub fn change_pos(&mut self, x: f32, y: f32, z: f32) {
        if (x > 0.0 && !self.x_pos_blocked) || (x < 0.0 && !self.x_neg_blocked) {
            self.hitbox.center.x += x;
        }
        if (y > 0.0 && !self.y_pos_blocked) || (y < 0.0 && !self.y_neg_blocked) {
            self.hitbox.center.y += y;
        }
        if (z > 0.0 && !self.z_pos_blocked) || (z < 0.0 && !self.z_neg_blocked) {
            self.hitbox.center.z += z;
        }
    }
    pub fn process_contacts(&mut self, contacts:&Vec<Contact<usize>>) {
        self.reset_blocked();
        for Contact { mtv: disp, .. } in contacts.iter() {
            if disp.x > 0.0 {
                self.x_neg_blocked = true;
            }
            if disp.x < 0.0 {
                self.x_pos_blocked = true;
            }
            if disp.y > 0.0 {
                self.y_neg_blocked = true;
                self.can_jump = true;
            }
            if disp.y < 0.0 {
                self.y_pos_blocked = true;
            }
            if disp.z > 0.0 {
                self.z_neg_blocked = true;
            }
            if disp.z < 0.0 {
                self.z_pos_blocked = true;
            }
        }
    }
    pub fn reset_blocked(&mut self) {
        self.x_pos_blocked = false;
        self.x_neg_blocked = false;
        self.y_pos_blocked = false;
        self.y_neg_blocked = false;
        self.z_pos_blocked = false;
        self.z_neg_blocked = false;
    }
    pub fn update(&mut self, camera: &mut Camera) {
        //change position based on velocity
        
        if (!self.y_neg_blocked) {
                self.vy -= 0.005;
            if (self.vy <= -0.1) { //terminal velocity
                self.vy = -0.1;
            }
        } else {
            self.vy = 0.0;
        }
        if (self.jump_pressed && self.can_jump) {
            self.vy = 0.15;
            self.can_jump = false;
        }

        if self.do_gravity {
            self.change_pos(0.0, self.vy, 0.0);
        }
        

        //change camera position to player position
        let mut forward = camera.target - camera.eye;
        forward.y = 0.0;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();
        let right = forward_norm.cross(camera.up);
        if self.is_forward_pressed {
            let movement = forward_norm * self.speed;
            self.change_pos(movement.x, 0.0, movement.z);
        }
        if self.is_backward_pressed {
            let movement = -1.0 * forward_norm * self.speed;
            self.change_pos(movement.x, 0.0, movement.z);
        }
        if self.is_right_pressed {
            let movement = right * self.speed;
            self.change_pos(movement.x, 0.0, movement.z);
        }
        if self.is_left_pressed {
            let movement = -1.0 * right * self.speed;
            self.change_pos(movement.x, 0.0, movement.z);
        }
        if self.is_up_pressed {
            self.change_pos(0.0, self.speed, 0.0);
        }
        if self.is_down_pressed {
            self.change_pos(0.0, -1.0 * self.speed, 0.0);
        }
        if self.is_gravity_pressed {
            self.do_gravity = !self.do_gravity;
        }
        self.hitbox.center.y += self.vy;
        let player_pos = self.get_pos();
        let player_p3 = cgmath::point3(player_pos.x, player_pos.y, player_pos.z);

        // set camera pos to player pos
        let player_diff = player_p3 - camera.eye;
        camera.eye += player_diff;
        camera.target += player_diff;
    }
    pub fn process_events(&mut self, events: &Events) -> bool {
        self.is_forward_pressed = events.key_held(VirtualKeyCode::W);
        self.is_left_pressed = events.key_held(VirtualKeyCode::A);
        self.is_backward_pressed = events.key_held(VirtualKeyCode::S);
        self.is_right_pressed = events.key_held(VirtualKeyCode::D);
        self.is_up_pressed = events.key_held(VirtualKeyCode::R);
        self.is_down_pressed = events.key_held(VirtualKeyCode::F);
        self.is_gravity_pressed = events.key_pressed(VirtualKeyCode::G);
        self.jump_pressed = events.key_held(VirtualKeyCode::Space);
        true
    }
}