use wgpu::{
    Buffer, BufferUsages, Device,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::helpers::vertex::Vertex;

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
