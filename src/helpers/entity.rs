use std::time::Duration;

use wgpu::Queue;

use crate::helpers::object::Object;

pub trait Entity {
    fn update(
        &mut self,
        queue: &Queue,
        total_time: Duration,
        delta_time: Duration,
    ) -> Result<(), ()>;
    fn get_object(&self) -> &Object;
}
