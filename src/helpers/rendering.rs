use bytemuck::{Pod, Zeroable};
use cgmath::{InnerSpace, Matrix4, Vector2, Vector3, Vector4, VectorSpace};
use wgpu::{
    Buffer, BufferAddress, BufferUsages, Device, VertexBufferLayout,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    helpers::math::{it_mat4, slerp},
    renderer::RenderGroupType,
};

pub struct Object {
    pub render_group_type: RenderGroupType,
    pub vertex_uniform_buffer: Buffer,
    pub fragment_uniform_buffer: Buffer,
    pub meshes: Vec<Mesh>,
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

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 4],
    normal: [f32; 4],
    uv: [f32; 4],
    colour: [f32; 4],
}

impl Vertex {
    pub fn new(
        position: Vector3<f32>,
        normal: Vector3<f32>,
        uv: Vector2<f32>,
        colour: Vector4<f32>,
    ) -> Self {
        Self {
            position: [position.x, position.y, position.z, 1.0],
            normal: [normal.x, normal.y, normal.z, 1.0],
            uv: [uv.x, uv.y, 1.0, 1.0],
            colour: [colour.x, colour.y, colour.z, colour.w],
        }
    }
    const ATTRIBUTES: [wgpu::VertexAttribute; 4] =
        wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4, 3 => Float32x4];
    pub fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }

    pub fn lerp(&self, other: Vertex, t: f32) -> Vertex {
        let position = self.get_position().lerp(other.get_position(), t);
        let normal = self.get_normal().lerp(other.get_normal(), t).normalize();
        let uv = self.get_uv().lerp(other.get_uv(), t);
        let colour = self.get_colour().lerp(other.get_colour(), t);

        Vertex::new(position, normal, uv, colour)
    }

    pub fn slerp(&self, other: Vertex, t: f32) -> Vertex {
        let position = slerp(self.get_position(), other.get_position(), t);
        let normal = slerp(self.get_normal(), other.get_normal(), t);
        let uv = slerp(self.get_uv(), other.get_uv(), t);
        let colour = slerp(self.get_colour(), other.get_colour(), t);

        Vertex::new(position, normal, uv, colour)
    }

    pub fn get_position(&self) -> Vector3<f32> {
        Vector3::new(self.position[0], self.position[1], self.position[2])
    }

    pub fn get_normal(&self) -> Vector3<f32> {
        Vector3::new(self.normal[0], self.normal[1], self.normal[2])
    }

    pub fn get_uv(&self) -> Vector2<f32> {
        Vector2::new(self.uv[0], self.uv[1])
    }

    pub fn get_colour(&self) -> Vector4<f32> {
        Vector4::new(
            self.colour[0],
            self.colour[1],
            self.colour[2],
            self.colour[3],
        )
    }
}

pub struct Mesh {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_length: u32,
}

pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

pub fn get_mesh(mesh_data: &MeshData, device: &Device) -> Mesh {
    let MeshData { vertices, indices } = mesh_data;
    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(vertices),
        usage: BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(indices),
        usage: BufferUsages::INDEX,
    });

    Mesh {
        vertex_buffer,
        index_buffer,
        index_length: indices.len() as u32,
    }
}

pub fn get_object(
    device: &Device,
    model_mat: Matrix4<f32>,
    meshes: Vec<Mesh>,
    render_group_type: RenderGroupType,
) -> Object {
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
    Object {
        render_group_type,
        vertex_uniform_buffer,
        fragment_uniform_buffer,
        meshes,
    }
}
