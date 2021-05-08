use crate::geom::*;
use cgmath::*;
use crate::camera::Camera;
use winit::event::*;

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
            speed: 0.2,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
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
            self.vy -= 0.005;//gravity
            if (self.vy <= -0.2) { //terminal velocity
                self.vy = -0.2;
            }
        } else {
            self.vy = 0.0;
        }
        if (self.jump_pressed && self.can_jump) {
            self.vy = 0.15;
            self.can_jump = false;
        }
        self.change_pos(0.0, self.vy, 0.0);

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
        self.hitbox.center.y += self.vy;
        let player_pos = self.get_pos();
        let player_p3 = cgmath::point3(player_pos.x, player_pos.y, player_pos.z);

        // set camera pos to player pos
        let player_diff = player_p3 - camera.eye;
        camera.eye += player_diff;
        camera.target += player_diff;
    }
    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::Up => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Down => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::W => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::A => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::S => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::D => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Space => {
                        self.jump_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
}