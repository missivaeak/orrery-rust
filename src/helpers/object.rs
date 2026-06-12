use bytemuck::{Pod, Zeroable};
use cgmath::Matrix4;
use wgpu::{
    Buffer, BufferUsages, Device,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    helpers::{
        math::{create_model, it_mat4},
        mesh::{Mesh, MeshData, get_mesh},
    },
    renderer::RenderGroupType,
};

pub struct Object {
    pub render_group_type: RenderGroupType,
    pub vertex_uniform_buffer: Buffer,
    pub fragment_uniform_buffer: Buffer,
    pub meshes: Vec<Mesh>,
}

pub struct ObjectOptions {
    pub model_mat: Matrix4<f32>,
    pub render_group_type: RenderGroupType,
}

impl Default for ObjectOptions {
    fn default() -> Self {
        Self {
            model_mat: create_model(
                (5.0, 0.0, 0.0).into(),
                (0.0, 0.0, 0.0).into(),
                (1.0, 1.0, 1.0).into(),
            ),
            render_group_type: RenderGroupType::Lit,
        }
    }
}

impl Object {
    pub fn from_meshes(device: &Device, meshes: Vec<Mesh>, options: ObjectOptions) -> Self {
        let vertex_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Object Vertex Uniform Buffer"),
            contents: bytemuck::bytes_of(&ObjectVertexUniform {
                model_mat: options.model_mat.into(),
                normal_mat: it_mat4(options.model_mat).into(),
            }),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let fragment_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Object Fragment Uniform Buffer"),
            contents: bytemuck::bytes_of(&ObjectFragmentUniform {
                colour: (0.5, 0.0, 1.0, 1.0).into(),
            }),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        Self {
            render_group_type: options.render_group_type,
            vertex_uniform_buffer,
            fragment_uniform_buffer,
            meshes,
        }
    }

    pub fn from_mesh_datas(
        device: &Device,
        mesh_datas: Vec<MeshData>,
        options: ObjectOptions,
    ) -> Self {
        let meshes: Vec<Mesh> = mesh_datas
            .iter()
            .map(|mesh_data| get_mesh(mesh_data, device))
            .collect();
        Self::from_meshes(device, meshes, options)
    }
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
