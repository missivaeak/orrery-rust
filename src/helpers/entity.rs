use std::time::Duration;

use cgmath::{Matrix4, Vector3};
use wgpu::Queue;

use crate::helpers::object::Object;

#[allow(unused, dead_code)]
pub struct UpdateDescriptor {
    pub delta_time: Duration,
    pub total_time: Duration,
    pub camera_position: Vector3<f32>,
    pub view_mat: Matrix4<f32>,
    pub projection_mat: Option<Matrix4<f32>>,
}

pub trait Entity {
    fn update(&mut self, queue: &Queue, update_descriptor: &UpdateDescriptor) -> Result<(), ()>;
    fn get_object(&self) -> &Object;
}
