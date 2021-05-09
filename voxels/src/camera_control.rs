use crate::camera::Camera;
use crate::geom::*;
use winit::event::*;
use crate::Events;
use std::f32::consts::PI;

pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    pub center_x: i32,
    pub center_y: i32,
    window_width: i32,
    window_height: i32,
    offset_x: i32,
    offset_y: i32,
}

impl CameraController {
    pub fn new(speed: f32, center_x: i32, center_y: i32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            center_x: center_x,
            center_y: center_y,
            window_width: center_x * 2,
            window_height: center_y * 2,
            offset_x: 0,
            offset_y: 0,
        }
    }

    pub fn process_events(&mut self, events: &Events) -> bool {
        //mouse movement
        self.offset_x = events.mouse_delta().0 as i32;
        self.offset_y = events.mouse_delta().1 as i32;
        //key presses
        self.is_forward_pressed = events.key_held(VirtualKeyCode::W);
        self.is_left_pressed = events.key_held(VirtualKeyCode::A);
        self.is_backward_pressed = events.key_held(VirtualKeyCode::S);
        self.is_right_pressed = events.key_held(VirtualKeyCode::D);

        true
    }

    pub fn update_camera(&mut self, camera: &mut Camera) {
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Prevents glitching when camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {
             camera.eye += forward_norm * self.speed;
             camera.target += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
             camera.eye -= forward_norm * self.speed;
             camera.target -= forward_norm * self.speed;
        }

        // let right = forward_norm.cross(camera.up);

        // Redo radius calc in case the up/ down is pressed.
        let forward = camera.target - camera.eye;
        // let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            // Rescale the distance between the target and eye so
            // that it doesn't change. The eye therefore still
            // lies on the circle made by the target and eye.
            // camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            //camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }

        // mouse stuff
        let full_circle = 2.0 * PI;
        if self.offset_x != 0 {
            let rotation_mag = self.offset_x as f32 / self.window_width as f32 * full_circle; // maybe include self speed
            let mut new_forward = camera.target - camera.eye;
            new_forward.x = forward.x * rotation_mag.cos() - forward.z * rotation_mag.sin();
            new_forward.z = forward.x * rotation_mag.sin() + forward.z * rotation_mag.cos();
            camera.target = camera.eye + new_forward.normalize();
            self.offset_x = 0;
        }
        if self.offset_y != 0 {
            let rotation_mag = -1.0 * self.offset_y as f32 / self.window_height as f32 * full_circle; // maybe include self speed
            let mut new_forward = camera.target - camera.eye;
            let horizontal_mag = PI - (new_forward.x.powf(2.0) + new_forward.z.powf(2.0)).powf(0.5); // TODO: Why does PI work here
            let current_rotation = (new_forward.y / horizontal_mag).atan();
            let mut new_rotation = current_rotation + rotation_mag;
            if new_rotation < -1.0 * PI {
                new_rotation = -1.0 * PI;
            } else if new_rotation > PI {
                new_rotation = PI;
            }
            new_forward.y = horizontal_mag * new_rotation;
            camera.target = camera.eye + new_forward.normalize();
            self.offset_y = 0;
        }
    }
}
