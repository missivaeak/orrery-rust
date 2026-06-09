use wgpu::{
    BufferUsages, Device,
    util::{BufferInitDescriptor, DeviceExt},
};
use winit::dpi::LogicalSize;

use crate::{
    controls::Controls,
    helpers::{
        math::{self, create_model, it_mat4},
        rendering::{GlobalFragmentUniform, GlobalVertexUniform, Object, ObjectVertexUniform},
    },
    primitives::{
        cube::get_cube_mesh_data, cubesphere::create_cubesphere_meshes,
        octasphere::create_octasphere_mesh_data, sphere::sphere_data, surface::surface_data,
    },
};

pub struct Scene {
    global_vertex_uniform: GlobalVertexUniform,
    global_fragment_uniform: GlobalFragmentUniform,
    objects: Vec<Object>,
}

impl Scene {
    pub fn new(device: &Device, controls: &Controls, size: LogicalSize<f32>) -> Self {
        let view_mat = controls.get_view_mat();
        let projection_mat = math::create_projection(size.width / size.height, true);
        let mut objects = Vec::new();
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

        objects.push(Object::from_mesh_datas(
            device,
            vec![get_cube_mesh_data()],
            None,
            None,
        ));
        objects.push(Object::from_mesh_datas(
            device,
            vec![sphere_data(1.1, 70, 70)],
            None,
            None,
        ));
        objects.push(Object::from_mesh_datas(
            device,
            vec![surface_data()],
            None,
            None,
        ));
        objects.push(Object::from_meshes(
            device,
            create_cubesphere_meshes(device, 2),
            None,
            None,
        ));
        objects.push(Object::from_mesh_datas(
            device,
            create_octasphere_mesh_data(5.0, 5),
            None,
            None,
        ));

        Self {
            global_vertex_uniform,
            global_fragment_uniform,
            objects,
        }
    }

    pub fn update(&mut self, device: &Device, view_mat: [[f32; 4]; 4], time_elapsed: f32) {
        self.global_vertex_uniform.view_mat = view_mat;

        let rotating_mat = create_model(
            (-0.8, -6.0, 2.4).into(),
            // (2.4, -0.5, 0.0).into(),
            (time_elapsed.sin(), time_elapsed.cos(), time_elapsed.cos()).into(),
            (1.0, 1.0, 1.0).into(),
        );

        self.objects[0].vertex_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Object Vertex Uniform Buffer"),
            contents: bytemuck::bytes_of(&ObjectVertexUniform {
                model_mat: rotating_mat.into(),
                normal_mat: it_mat4(rotating_mat).into(),
            }),
            usage: BufferUsages::UNIFORM,
        });
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

    pub fn get_objects(&self) -> &Vec<Object> {
        &self.objects
    }

    pub fn get_global_uniforms(&self) -> (&GlobalVertexUniform, &GlobalFragmentUniform) {
        (&self.global_vertex_uniform, &self.global_fragment_uniform)
    }
}
