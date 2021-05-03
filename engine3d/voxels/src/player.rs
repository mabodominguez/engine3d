use crate::geom::*;
use cgmath::*;
use crate::camera::Camera;
use winit::event::*;

pub struct Player {
    pub hitbox:BBox,
    pub vx: f32,
    pub vy: f32,
    pub facing_direction: Vec3,
    pub speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl Player {
    pub fn new(hitbox:BBox) -> Self {
        Self {
            hitbox: hitbox,
            vx: 0.0,
            vy: 0.0,
            facing_direction: cgmath::vec3(0.0, 0.0, 0.0),
            speed: 0.5,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }
    pub fn get_pos(&self) -> Pos3 {
        return self.hitbox.center;
    }
    pub fn change_pos(&mut self, x: f32, y: f32, z: f32) {
        self.hitbox.center.x += x;
        self.hitbox.center.y += y;
        self.hitbox.center.z += z;
    }
    pub fn update(&mut self, camera: &mut Camera) {
        let mut forward = camera.target - camera.eye;
        forward.y = 0.0;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();
        let right = forward_norm.cross(camera.up);
        if self.is_forward_pressed {
            let movement = forward_norm * self.speed;
            println!("{}", movement.x);
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
                    _ => false,
                }
            }
            _ => false,
        }
    }
}