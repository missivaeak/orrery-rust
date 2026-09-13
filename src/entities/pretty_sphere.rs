use cgmath::{Matrix4, Quaternion, Vector3};
use wgpu::{Device, Queue};

use crate::{
    helpers::{
        asset_library::AssetLibrary,
        constants::EARTH_RADIUS,
        entity::{Entity, UpdateDescriptor},
        math::it_mat4,
        object::{Object, ObjectOptions, ObjectVertexUniform},
        texture::TextureType,
    },
    primitives::sphere::sphere_data,
    renderer::RenderGroupType,
};

pub struct PrettySphere {
    object: Object,
    translation: Vector3<f32>,
    scale: Vector3<f32>,
    rotation: Quaternion<f32>,
}

impl PrettySphere {
    pub fn new(device: &Device, asset_library: &AssetLibrary) -> Self {
        let translation = Vector3::new(EARTH_RADIUS + 1.0, 0.0, 0.0);
        let scale = Vector3::new(500.0, 500.0, 500.0);
        let rotation = Quaternion::new(1.0, 0.0, 0.0, 0.0);

        let mut options = ObjectOptions::default();
        options.render_group_type = RenderGroupType::Lit;
        
        // Try to get skybox texture from asset library for cubemap rendering
        options.texture = asset_library.get_texture("skybox");
        options.texture_type = asset_library
            .get_texture_type("skybox")
            .unwrap_or(TextureType::Texture2D);

        let object = Object::from_mesh_datas(
            device,
            vec![sphere_data(0.1, 20, 10)],
            options,
        );
        Self {
            object,
            translation,
            scale,
            rotation,
        }
    }
}

impl Entity for PrettySphere {
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
