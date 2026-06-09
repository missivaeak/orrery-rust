use std::ops::{Add, Div, Mul};

use cgmath::{ElementWise, Vector2, Vector3, num_traits::ToPrimitive};
use wgpu::{
    BufferUsages, Device,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::helpers::rendering::{Mesh, Vertex};

pub fn create_cubesphere_meshes(device: &Device, resolution: usize) -> Vec<Mesh> {
    vec![
        create_face(device, Vector3::unit_x(), resolution + 1),
        create_face(device, Vector3::unit_x().mul(-1.0), resolution + 1),
        create_face(device, Vector3::unit_y(), resolution + 1),
        create_face(device, Vector3::unit_y().mul(-1.0), resolution + 1),
        create_face(device, Vector3::unit_z(), resolution + 1),
        create_face(device, Vector3::unit_z().mul(-1.0), resolution + 1),
    ]
}

fn create_face(device: &Device, normal: Vector3<f32>, resolution: usize) -> Mesh {
    let axis_a: Vector3<f32> = normal.yzx();
    let axis_b = normal.cross(axis_a);
    let mut vertices: Vec<Vertex> = Vec::with_capacity(resolution * resolution);
    let mut indices: Vec<u16> = Vec::with_capacity((resolution - 1) * (resolution - 1) * 6);

    if let Some(res_f32) = resolution.to_f32() {
        for y in 0..resolution {
            if let Some(y_f32) = y.to_f32() {
                for x in 0..resolution {
                    if let Some(x_f32) = x.to_f32() {
                        let vertex_index: u16 = (x + y * resolution) as u16;
                        let offset: Vector2<f32> = Vector2::new(x_f32, y_f32).div(res_f32 - 1.0);
                        let point: Vector3<f32> = map_cube_to_sphere(
                            normal
                                .add(axis_a.mul(2.0 * offset.x - 1.0))
                                .add(axis_b.mul(2.0 * offset.y - 1.0)),
                        );
                        let vertex = Vertex::new(
                            point,
                            point,
                            offset,
                            normal.xyz().add_element_wise(1.0).div(2.0).extend(1.0),
                        );
                        vertices.push(vertex);

                        if x != resolution - 1 && y != resolution - 1 {
                            indices.push(vertex_index);
                            indices.push(vertex_index + resolution as u16 + 1);
                            indices.push(vertex_index + resolution as u16);
                            indices.push(vertex_index);
                            indices.push(vertex_index + 1);
                            indices.push(vertex_index + resolution as u16 + 1);
                        }
                    }
                }
            }
        }
    }
    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: BufferUsages::INDEX,
    });

    Mesh {
        vertex_buffer,
        index_buffer,
        index_length: indices.len() as u32,
    }
}

fn map_cube_to_sphere(cube_point: Vector3<f32>) -> Vector3<f32> {
    let x_square = cube_point.x * cube_point.x;
    let y_square = cube_point.y * cube_point.y;
    let z_square = cube_point.z * cube_point.z;
    let x = cube_point.x * (1.0 - (y_square + z_square) / 2.0 + (y_square * z_square) / 3.0).sqrt();
    let y = cube_point.y * (1.0 - (z_square + x_square) / 2.0 + (z_square * x_square) / 3.0).sqrt();
    let z = cube_point.z * (1.0 - (y_square + x_square) / 2.0 + (y_square * x_square) / 3.0).sqrt();

    Vector3::new(x, y, z)
}
