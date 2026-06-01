use cgmath::{InnerSpace, Matrix4, Point3, Quaternion, Rad, Rotation, Rotation3, Vector3, Zero};
use winit::{
    event::{ElementState, KeyEvent, MouseButton},
    keyboard::{Key, NamedKey},
};

use crate::helpers::math::create_view;
pub enum InputEventResult {
    Ok,
    RequestClose,
    RequestLockCursor,
    RequestUnlockCursor,
    CameraMoved,
}

pub struct Controls {
    pub camera_position: Point3<f32>,
    pub camera_direction: Vector3<f32>,
    velocity: Vector3<f32>,
    acceleration: Vector3<f32>,
    movement_acceleration: f32,
    max_velocity: f32,
    breaking_fixed: f32,
    breaking_factor: f32,
    right_pressed: bool,
    left_pressed: bool,
    forward_pressed: bool,
    back_pressed: bool,
    up_pressed: bool,
    down_pressed: bool,
    camera_controlled: bool,
}

impl Controls {
    pub fn new() -> Self {
        Self {
            camera_position: (3.0, 3.0, 1.5).into(),
            camera_direction: Vector3::new(-3.0, -3.0, -1.5).normalize(),
            velocity: (0.0, 0.0, 0.0).into(),
            acceleration: (0.0, 0.0, 0.0).into(),
            movement_acceleration: 1.0,
            max_velocity: 0.0001,
            breaking_fixed: 0.1,
            breaking_factor: 0.1,
            right_pressed: false,
            left_pressed: false,
            forward_pressed: false,
            back_pressed: false,
            up_pressed: false,
            down_pressed: false,
            camera_controlled: false,
        }
    }

    pub fn get_view_mat(&self) -> Matrix4<f32> {
        create_view(self.camera_position, self.camera_direction)
    }

    fn get_movement_acceleration(&self) -> Vector3<f32> {
        let Self {
            right_pressed,
            left_pressed,
            forward_pressed,
            back_pressed,
            up_pressed,
            down_pressed,
            camera_direction,
            ..
        } = self;

        let local_acceleration = Vector3::new(
            ((*right_pressed as i32) - (*left_pressed as i32)) as f32,
            ((*forward_pressed as i32) - (*back_pressed as i32)) as f32,
            ((*up_pressed as i32) - (*down_pressed as i32)) as f32,
        );
        let right_direction = camera_direction.cross(Vector3::unit_z());
        let right_direction = if right_direction.magnitude2() > 0.0 {
            right_direction.normalize()
        } else {
            Vector3::unit_x() // fallback
        };

        let movement_acceleration = (right_direction * local_acceleration.x)
            + (camera_direction * local_acceleration.y)
            + (Vector3::unit_z() * local_acceleration.z);

        if movement_acceleration.is_zero() {
            movement_acceleration
        } else {
            movement_acceleration.normalize_to(self.movement_acceleration)
        }
    }

    pub fn update(&mut self) {
        let mut speed = self.velocity.magnitude();
        let breaking = self.breaking_fixed + (speed * self.breaking_factor);

        if speed < breaking {
            self.velocity = Vector3::zero();
            speed = 0.0;
        } else {
            self.velocity = self.velocity.normalize_to(speed - breaking);
            speed -= breaking;
        }

        self.velocity += self.acceleration;

        if speed > self.max_velocity {
            self.velocity = self.velocity.normalize_to(self.max_velocity);
        }

        self.camera_position += self.velocity;
    }

    fn update_key_state(&mut self, key_character: &str, state: ElementState) {
        match key_character {
            "E" => self.right_pressed = state == ElementState::Pressed,
            "A" => self.left_pressed = state == ElementState::Pressed,
            "Ä" => self.forward_pressed = state == ElementState::Pressed,
            "O" => self.back_pressed = state == ElementState::Pressed,
            " " => self.up_pressed = state == ElementState::Pressed,
            "J" => self.down_pressed = state == ElementState::Pressed,
            _ => (),
        }

        self.acceleration = self.get_movement_acceleration()
    }

    pub fn handle_key_input(&mut self, key_event: KeyEvent) -> InputEventResult {
        match key_event {
            KeyEvent {
                logical_key: Key::Named(NamedKey::Escape),
                ..
            } => InputEventResult::RequestClose,
            KeyEvent {
                logical_key: Key::Character(ref character),
                state,
                ..
            } => {
                let key_character_string = character.to_uppercase();
                let key_character = key_character_string.as_str();
                if matches!(key_character, "Ä" | "O" | "E" | "A" | "J") {
                    self.update_key_state(key_character, state);
                }
                InputEventResult::Ok
            }
            KeyEvent {
                logical_key: Key::Named(named_key),
                state,
                ..
            } => {
                if let Some(key_name) = named_key.to_text()
                    && matches!(key_name, " " | "F1")
                {
                    self.update_key_state(key_name, state);
                }

                InputEventResult::Ok
            }
            _ => InputEventResult::Ok,
        }
    }

    pub fn handle_mouse_input(&mut self, button: MouseButton) -> InputEventResult {
        match button {
            MouseButton::Left => {
                self.camera_controlled = !self.camera_controlled;
                println!("{:?}", self.camera_controlled);
                if self.camera_controlled {
                    return InputEventResult::RequestLockCursor;
                }
                InputEventResult::RequestUnlockCursor
            }
            _ => InputEventResult::Ok,
        }
    }

    pub fn handle_mouse_move(&mut self, (x_delta, y_delta): (f64, f64)) -> InputEventResult {
        if self.camera_controlled {
            let sensitivity = 0.002;

            let yaw = -(x_delta as f32) * sensitivity;
            let pitch = -(y_delta as f32) * sensitivity;

            let up = Vector3::unit_z();
            let right = self.camera_direction.cross(up).normalize();

            let yaw_rot = Quaternion::from_axis_angle(up, Rad(yaw));
            let pitch_rot = Quaternion::from_axis_angle(right, Rad(pitch));

            self.camera_direction = (yaw_rot * pitch_rot)
                .rotate_vector(self.camera_direction)
                .normalize();

            return InputEventResult::CameraMoved;
        }

        InputEventResult::Ok
    }
}
