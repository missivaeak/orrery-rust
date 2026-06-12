use std::time::Duration;

use wgpu::{Device, Queue};
use winit::dpi::LogicalSize;

use crate::{
    controls::Controls,
    entities::{
        cube_sphere::CubeSphere, planet::Planet, pretty_cube::PrettyCube,
        pretty_sphere::PrettySphere, wavy_surface::WavySurface,
    },
    helpers::{
        entity::Entity,
        math::create_projection,
        object::{GlobalFragmentUniform, GlobalVertexUniform, Object},
    },
};

pub struct Scene {
    global_vertex_uniform: GlobalVertexUniform,
    global_fragment_uniform: GlobalFragmentUniform,
    entities: Vec<Box<dyn Entity>>,
}

impl Scene {
    pub fn new(device: &Device, controls: &Controls, size: LogicalSize<f32>) -> Self {
        let view_mat = controls.get_view_mat();
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

        entities.push(Box::new(PrettyCube::new(device)));
        entities.push(Box::new(CubeSphere::new(device)));
        entities.push(Box::new(Planet::new(device)));
        entities.push(Box::new(WavySurface::new(device)));
        entities.push(Box::new(PrettySphere::new(device)));

        Self {
            global_vertex_uniform,
            global_fragment_uniform,
            entities,
        }
    }

    pub fn update(
        &mut self,
        queue: &Queue,
        view_mat: [[f32; 4]; 4],
        total_time: Duration,
        delta_time: Duration,
    ) {
        self.global_vertex_uniform.view_mat = view_mat;

        for entity in self.entities.iter_mut() {
            if let Err(error) = entity.update(queue, total_time, delta_time) {
                println!("{:?}", error)
            }
        }

        // self.objects[1].vertex_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
        //     label: Some("Object Vertex Uniform Buffer sphere"),
        //     contents: bytemuck::bytes_of(&ObjectVertexUniform {
        //         model_mat: sphere_mat.into(),
        //         normal_mat: it_mat4(sphere_mat).into(),
        //     }),
        //     usage: BufferUsages::UNIFORM,
        // });

        // for object in self.objects.iter_mut() {
        //     object.renderable.vertex_uniform.model_mat = model_mat.into();
        //     object.renderable.vertex_uniform.normal_mat = it_mat4(model_mat).into();
        // }
    }

    pub fn get_objects_iter(&self) -> impl Iterator<Item = &Object> {
        self.entities.iter().map(|entity| entity.get_object())
    }

    pub fn get_global_uniforms(&self) -> (&GlobalVertexUniform, &GlobalFragmentUniform) {
        (&self.global_vertex_uniform, &self.global_fragment_uniform)
    }
}
