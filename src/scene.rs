use std::time::Instant;

use bytemuck::{Pod, Zeroable};
use cgmath::{Vector2, Vector3};
use wgpu::{
    Buffer, BufferAddress, BufferUsages, Device, VertexBufferLayout,
    util::{BufferInitDescriptor, DeviceExt},
};
use winit::dpi::LogicalSize;

use crate::{
    controls::Controls,
    helpers::math::{self, create_model, it_mat4},
    primitives::{
        cube::{cube_normals, cube_positions, cube_uvs},
        sphere::sphere_data,
    },
    renderer::RenderGroupType,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 4],
    normal: [f32; 4],
    uv: [f32; 4],
}

impl Vertex {
    pub fn new(position: Vector3<f32>, normal: Vector3<f32>, uv: Vector2<f32>) -> Self {
        Self {
            position: [position.x, position.y, position.z, 1.0],
            normal: [normal.x, normal.y, normal.z, 1.0],
            uv: [uv.x, uv.y, 1.0, 1.0],
        }
    }
    const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4];
    pub fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

pub struct Object {
    pub render_group_type: RenderGroupType,
    pub vertex_uniform_buffer: Buffer,
    pub fragment_uniform_buffer: Buffer,
    pub buffer: Buffer,
    pub buffer_length: u32,
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GlobalVertexUniform {
    pub view_mat: [[f32; 4]; 4],
    pub projection_mat: [[f32; 4]; 4],
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GlobalFragmentUniform {
    pub camera_position: [f32; 4],
    pub light_position: [f32; 4],
    pub light_colour: [f32; 4],
    pub specular_colour: [f32; 4],
    pub ambient_intensity: f32,
    pub diffuse_intensity: f32,
    pub specular_intensity: f32,
    pub specular_gloss: f32,
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ObjectVertexUniform {
    pub model_mat: [[f32; 4]; 4],
    pub normal_mat: [[f32; 4]; 4],
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ObjectFragmentUniform {
    pub colour: [f32; 4],
}

pub struct Scene {
    initial_timestamp: Instant,
    last_timestamp: Instant,
    global_vertex_uniform: GlobalVertexUniform,
    global_fragment_uniform: GlobalFragmentUniform,
    objects: Vec<Object>,
}

impl Scene {
    pub fn new(device: &Device, size: LogicalSize<f32>) -> Self {
        let timestamp = Instant::now();

        let controls = Controls::new();
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

        objects.push(get_cube_object(device));
        objects.push(get_sphere_object(device));

        Self {
            global_vertex_uniform,
            global_fragment_uniform,
            objects,
            last_timestamp: timestamp,
            initial_timestamp: timestamp,
        }
    }

    pub fn update(&mut self, device: &Device, view_mat: [[f32; 4]; 4], time_elapsed: f32) {
        self.global_vertex_uniform.view_mat = view_mat;

        let model_mat = create_model(
            (-0.8, -6.0, 2.4).into(),
            // (2.4, -0.5, 0.0).into(),
            (time_elapsed.sin(), time_elapsed.cos(), time_elapsed.cos()).into(),
            (1.0, 1.0, 1.0).into(),
        );

        // let sphere_mat = create_model(
        //     (0.0, 0.0, 0.0).into(),
        //     // (2.4, -0.5, 0.0).into(),
        //     (time_elapsed.sin(), time_elapsed.cos(), time_elapsed.cos()).into(),
        //     (1.0, 1.0, 1.0).into(),
        // );
        self.objects[0].vertex_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Object Vertex Uniform Buffer"),
            contents: bytemuck::bytes_of(&ObjectVertexUniform {
                model_mat: model_mat.into(),
                normal_mat: it_mat4(model_mat).into(),
            }),
            usage: BufferUsages::UNIFORM,
        });
        // self.objects[1].renderable.vertex_uniform.model_mat = sphere_mat.into();
        // self.objects[1].renderable.vertex_uniform.normal_mat = it_mat4(sphere_mat).into();

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

fn get_cube_object(device: &Device) -> Object {
    let positions = cube_positions();
    let normals = cube_normals();
    let uvs = cube_uvs();
    let vertices = {
        let mut data: Vec<Vertex> = Vec::with_capacity(positions.len());
        for i in 0..positions.len() {
            data.push(Vertex::new(positions[i], normals[i], uvs[i]));
        }
        data
    };
    let model_mat = math::create_model(
        (0.0, 0.0, 0.0).into(),
        (0.0, 0.0, 0.0).into(),
        (1.0, 1.0, 1.0).into(),
    );
    let vertex_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Object Vertex Uniform Buffer"),
        contents: bytemuck::bytes_of(&ObjectVertexUniform {
            model_mat: model_mat.into(),
            normal_mat: it_mat4(model_mat).into(),
        }),
        usage: BufferUsages::UNIFORM,
    });
    let fragment_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Object Fragment Uniform Buffer"),
        contents: bytemuck::bytes_of(&ObjectFragmentUniform {
            colour: (0.5, 0.0, 1.0, 1.0).into(),
        }),
        usage: BufferUsages::UNIFORM,
    });
    let buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: BufferUsages::VERTEX,
    });
    Object {
        render_group_type: RenderGroupType::Lit,
        vertex_uniform_buffer,
        fragment_uniform_buffer,
        buffer_length: vertices.len() as u32,
        buffer,
    }
}

fn get_sphere_object(device: &Device) -> Object {
    let (positions, normals, uvs) = sphere_data(1.1, 15, 30);
    let vertices = {
        let mut data: Vec<Vertex> = Vec::with_capacity(positions.len());
        for i in 0..positions.len() {
            data.push(Vertex::new(positions[i], normals[i], uvs[i]));
        }
        data
    };
    let model_mat = math::create_model(
        (0.0, 0.0, 0.0).into(),
        (0.0, 0.0, 0.0).into(),
        (1.0, 1.0, 1.0).into(),
    );
    let vertex_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Object Vertex Uniform Buffer"),
        contents: bytemuck::bytes_of(&ObjectVertexUniform {
            model_mat: model_mat.into(),
            normal_mat: it_mat4(model_mat).into(),
        }),
        usage: BufferUsages::UNIFORM,
    });
    let fragment_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Object Fragment Uniform Buffer"),
        contents: bytemuck::bytes_of(&ObjectFragmentUniform {
            colour: (0.5, 0.0, 1.0, 1.0).into(),
        }),
        usage: BufferUsages::UNIFORM,
    });
    let buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: BufferUsages::VERTEX,
    });

    Object {
        render_group_type: RenderGroupType::Lit,
        buffer_length: vertices.len() as u32,
        buffer,
        vertex_uniform_buffer,
        fragment_uniform_buffer,
    }
}
