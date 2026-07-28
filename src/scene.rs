use cgmath::Matrix4;
use wgpu::{Device, Queue};
use winit::dpi::LogicalSize;

use crate::{
    controls::Controls,
    entities::{cube_planet::CubePlanet, pretty_sphere::PrettySphere},
    helpers::{
        entity::{Entity, UpdateDescriptor},
        math::{create_projection, create_view},
        object::{GlobalFragmentUniform, GlobalVertexUniform, Object},
    },
};

pub struct Scene {
    global_vertex_uniform: GlobalVertexUniform,
    global_fragment_uniform: GlobalFragmentUniform,
    entities: Vec<Box<dyn Entity>>,
    projection_mat: Matrix4<f32>,
}

#[allow(unused, dead_code)]
pub struct SceneUpdateDescriptor {
    pub projection_mat: Matrix4<f32>,
}

impl Scene {
    pub fn new(device: &Device, controls: &Controls, size: LogicalSize<f32>) -> Self {
        let view_mat = create_view(controls.camera_position, controls.camera_direction);
        let projection_mat = create_projection(size.width / size.height, true);
        let mut entities: Vec<Box<dyn Entity>> = Vec::new();
        let global_vertex_uniform = GlobalVertexUniform {
            projection_mat: projection_mat.into(),
            view_mat: view_mat.into(),
        };
        let global_fragment_uniform = GlobalFragmentUniform {
            camera_position: (
                controls.camera_position.x,
                controls.camera_position.y,
                controls.camera_position.z,
                1.0,
            )
                .into(),
            light_position: (3.0, 0.0, 5.0, 1.0).into(),
            light_colour: (1.0, 1.0, 1.0, 1.0).into(),
            specular_colour: (1.0, 1.0, 1.0, 1.0).into(),
            ambient_intensity: 0.1,
            diffuse_intensity: 0.8,
            specular_intensity: 0.4,
            specular_gloss: 30.0,
        };

        entities.push(Box::new(CubePlanet::new(device)));
        entities.push(Box::new(PrettySphere::new(device)));

        Self {
            global_vertex_uniform,
            global_fragment_uniform,
            entities,
            projection_mat,
        }
    }

    pub fn update(&mut self, queue: &Queue, update_descriptor: &UpdateDescriptor) {
        self.global_vertex_uniform.view_mat = update_descriptor.controls.view_mat.into();
        self.global_vertex_uniform.projection_mat = update_descriptor.scene.projection_mat.into();

        for entity in self.entities.iter_mut() {
            if let Err(error) = entity.update(queue, update_descriptor) {
                println!("{:?}", error)
            }
        }
    }

    pub fn get_update_descriptor(&self) -> SceneUpdateDescriptor {
        SceneUpdateDescriptor {
            projection_mat: self.projection_mat,
        }
    }

    pub fn get_objects_iter(&self) -> impl Iterator<Item = &Object> {
        self.entities.iter().map(|entity| entity.get_object())
    }

    pub fn get_global_uniforms(&self) -> (&GlobalVertexUniform, &GlobalFragmentUniform) {
        (&self.global_vertex_uniform, &self.global_fragment_uniform)
    }
}
