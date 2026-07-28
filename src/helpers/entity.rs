use wgpu::Queue;

use crate::{
    app::AppUpdateDescriptor, controls::ControlsUpdateDescriptor, gui::GuiUpdateDescriptor,
    helpers::object::Object, renderer::RendererUpdateDescriptor, scene::SceneUpdateDescriptor,
};

#[allow(unused, dead_code)]
pub struct UpdateDescriptor {
    pub app: AppUpdateDescriptor,
    pub controls: ControlsUpdateDescriptor,
    pub scene: SceneUpdateDescriptor,
    pub renderer: Option<RendererUpdateDescriptor>,
    pub gui: GuiUpdateDescriptor,
}

pub trait Entity {
    fn update(&mut self, queue: &Queue, update_descriptor: &UpdateDescriptor) -> Result<(), ()>;
    fn get_object(&self) -> &Object;
}
