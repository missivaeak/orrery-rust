use cgmath::{Matrix4, Quaternion, Vector3};
use wgpu::{Device, Queue};

use crate::{
    helpers::{
        entity::{Entity, UpdateDescriptor},
        math::it_mat4,
        object::{Object, ObjectOptions, ObjectVertexUniform},
    },
    primitives::cube::get_cube_mesh_data,
};

pub struct PrettyCube {
    object: Object,
    translation: Vector3<f32>,
    scale: Vector3<f32>,
    rotation: Quaternion<f32>,
}

impl PrettyCube {
    pub fn new(device: &Device) -> Self {
        let translation = Vector3::new(-7.0, 0.0, 0.0);
        let scale = Vector3::new(1.0, 1.0, 1.0);
        let rotation = Quaternion::new(1.0, 0.0, 0.0, 0.0);
        let object =
            Object::from_mesh_datas(device, vec![get_cube_mesh_data()], ObjectOptions::default());
        Self {
            object,
            translation,
            scale,
            rotation,
        }
    }
}

impl Entity for PrettyCube {
    fn update(&mut self, queue: &Queue, _update_descriptor: &UpdateDescriptor) -> Result<(), ()> {
        let model_mat = Matrix4::from_translation(self.translation)
            * Matrix4::from(self.rotation)
            * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);

        queue.write_buffer(
            &self.object.vertex_uniform_buffer,
            0,
            bytemuck::bytes_of(&ObjectVertexUniform {
                model_mat: model_mat.into(),
                normal_mat: it_mat4(model_mat).into(),
            }),
        );
        Ok(())
    }

    fn get_object(&self) -> &Object {
        &self.object
    }
}
