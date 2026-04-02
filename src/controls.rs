use cgmath::{Matrix4, Point3, Vector3};
use winit::{
    event::KeyEvent,
    keyboard::{Key, NamedKey, SmolStr},
};

use crate::math::create_view;

pub struct Controls {
    pub position: Point3<f32>,
    pub look_direction: Point3<f32>,
    velocity: Vector3<f32>,
    acceleration: Vector3<f32>,
    gravity: Vector3<f32>,
    max_speed: f32,
    breaking_fixed: f32,
    breaking_factor: f32,
}

impl Controls {
    pub fn new() -> Self {
        Self {
            position: (3.0, 3.0, 1.5).into(),
            look_direction: (0.0, 0.0, 0.0).into(),
            velocity: (0.0, 0.0, 0.0).into(),
            acceleration: (0.0, 0.0, 0.0).into(),
            gravity: (0.0, 0.0, 0.0).into(),
            max_speed: 4.0,
            breaking_fixed: 0.1,
            breaking_factor: 2.0,
        }
    }

    pub fn get_view_mat(&self) -> Matrix4<f32> {
        create_view(self.position, self.look_direction)
    }

    pub fn handle_key_input(&mut self, key_event: KeyEvent) {
        match key_event {
            KeyEvent {
                logical_key: Key::Character(ref c),
                ..
            } if c == "w" => {
                self.position.x += 0.1;
            }
            _ => {}
        }
    }
}
