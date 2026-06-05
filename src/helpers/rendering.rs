use bytemuck::{Pod, Zeroable};
use cgmath::{Vector2, Vector3, Vector4};
use wgpu::{Buffer, BufferAddress, VertexBufferLayout};

use crate::renderer::RenderGroupType;

pub struct Object {
    pub render_group_type: RenderGroupType,
    pub vertex_uniform_buffer: Buffer,
    pub fragment_uniform_buffer: Buffer,
    pub meshes: Vec<Mesh>,
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
}

pub struct Mesh {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_length: u32,
}
