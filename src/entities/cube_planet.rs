use cgmath::{
    Deg, ElementWise, InnerSpace, Matrix4, Quaternion, Rad, Rotation3, Vector2, Vector3,
    num_traits::{ToPrimitive, *},
};

use wgpu::{
    BufferUsages, Device, Queue,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::helpers::{
    entity::{Entity, UpdateDescriptor},
    math::it_mat4,
    mesh::{Mesh, MeshData},
    object::{Object, ObjectOptions, ObjectVertexUniform},
    vertex::Vertex,
};

use std::ops::{Add, Div, Mul};

pub struct CubePlanet {
    object: Object,
    translation: Vector3<f32>,
    scale: Vector3<f32>,
    rotation: Quaternion<f32>,
    faces: Vec<Face>,
}

impl CubePlanet {
    pub fn new(device: &Device) -> Self {
        let translation = Vector3::new(0.0, 0.0, 0.0);
        let scale = Vector3::new(1.0, 1.0, 1.0);
        let axis = Vector3::new(1.0, -1.0, 0.0).normalize();
        let angle = Rad((1.0 / 3.0.sqrt()).acos());
        let rotation = Quaternion::from_axis_angle(axis, angle);

        let radius = 1.0;
        let faces = vec![
            Face::new(Vector3::unit_x(), radius),
            Face::new(Vector3::unit_x() * -1.0, radius),
            Face::new(Vector3::unit_y(), radius),
            Face::new(Vector3::unit_y() * -1.0, radius),
            Face::new(Vector3::unit_z(), radius),
            Face::new(Vector3::unit_z() * -1.0, radius),
        ];

        let mesh_datas = faces.iter().map(|face| face.get_mesh_data()).collect();

        let object = Object::from_mesh_datas(
            device,
            mesh_datas,
            ObjectOptions {
                model_mat: Matrix4::from_translation(Vector3::new(-7.0, -7.0, -3.0)),
                ..Default::default()
            },
        );

        Self {
            object,
            translation,
            scale,
            rotation,
            faces,
        }
    }
}

struct Face {
    /// Vectors describing the plane of this cube face.
    pub normal: Vector3<f32>,
    pub tangent: Vector3<f32>,

    /// Radius of the planet.
    pub radius: f32,

    /// Maximum subdivision depth.
    pub max_depth: u32,
}

impl Face {
    pub fn new(normal: Vector3<f32>, radius: f32) -> Self {
        Self {
            normal,
            tangent: normal.yzx(),
            radius,
            max_depth: 5,
        }
    }

    fn get_mesh_data(&self) -> MeshData {
        let binormal = self.normal.cross(self.tangent);
        let mut vertices: Vec<Vertex> = Vec::with_capacity(4);
        let mut indices: Vec<u16> = Vec::with_capacity(6);

        vertices.push(Vertex::new(
            self.normal - self.tangent - binormal,
            self.normal,
            Vector2::new(-self.radius, -self.radius),
            self.normal.xyzz(),
        ));

        vertices.push(Vertex::new(
            self.normal + self.tangent - binormal,
            self.normal,
            Vector2::new(self.radius, -self.radius),
            self.normal.xyzz(),
        ));

        vertices.push(Vertex::new(
            self.normal + self.tangent + binormal,
            self.normal,
            Vector2::new(self.radius, self.radius),
            self.normal.xyzz(),
        ));

        vertices.push(Vertex::new(
            self.normal - self.tangent + binormal,
            self.normal,
            Vector2::new(-self.radius, self.radius),
            self.normal.xyzz(),
        ));

        indices.push(0);
        indices.push(1);
        indices.push(2);
        indices.push(2);
        indices.push(3);
        indices.push(0);

        MeshData { vertices, indices }
    }
}

impl Entity for CubePlanet {
    fn update(&mut self, queue: &Queue, update_descriptor: &UpdateDescriptor) -> Result<(), ()> {
        let dt = update_descriptor.delta_time.as_secs_f32();

        // 90 degrees per second
        let speed = 100.0;

        let delta_rotation = Quaternion::from_angle_z(Deg(speed * dt));

        self.rotation = delta_rotation * self.rotation;

        let model_mat = Matrix4::from_translation(self.translation)
            * Matrix4::from(self.rotation)
            * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);

        // let mut meshes = Vec::with_capacity(6);

        // pull out vector3 out of self.quad_tree_roots here
        // populate meshes with vertices and indices

        // self.object.meshes = meshes;

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

// pub fn create_cubesphere_meshes(device: &Device, resolution: usize) -> Vec<Mesh> {
//     vec![
//         // Ups
//         create_face(device, Vector3::unit_x(), resolution + 1),
//         create_face(device, Vector3::unit_y(), resolution + 1),
//         create_face(device, Vector3::unit_z(), resolution + 1),
//         // Downs
//         create_face(device, Vector3::unit_x().mul(-1.0), resolution + 1),
//         create_face(device, Vector3::unit_y().mul(-1.0), resolution + 1),
//         create_face(device, Vector3::unit_z().mul(-1.0), resolution + 1),
//     ]
// }
//
// fn create_face(device: &Device, normal: Vector3<f32>, resolution: usize) -> Mesh {
//     let axis_a: Vector3<f32> = normal.yzx();
//     let axis_b = normal.cross(axis_a);
//     let mut vertices: Vec<Vertex> = Vec::with_capacity(resolution * resolution);
//     let mut indices: Vec<u16> = Vec::with_capacity((resolution - 1) * (resolution - 1) * 6);
//
//     if let Some(res_f32) = resolution.to_f32() {
//         for y in 0..resolution {
//             if let Some(y_f32) = y.to_f32() {
//                 for x in 0..resolution {
//                     if let Some(x_f32) = x.to_f32() {
//                         let vertex_index: u16 = (x + y * resolution) as u16;
//                         let offset: Vector2<f32> = Vector2::new(x_f32, y_f32).div(res_f32 - 1.0);
//                         let point: Vector3<f32> = map_cube_to_sphere(
//                             normal
//                                 .add(axis_a.mul(2.0 * offset.x - 1.0))
//                                 .add(axis_b.mul(2.0 * offset.y - 1.0)),
//                         );
//                         let vertex = Vertex::new(
//                             point,
//                             point,
//                             offset,
//                             normal.xyz().add_element_wise(1.0).div(2.0).extend(1.0),
//                         );
//                         vertices.push(vertex);
//
//                         if x != resolution - 1 && y != resolution - 1 {
//                             indices.push(vertex_index);
//                             indices.push(vertex_index + resolution as u16 + 1);
//                             indices.push(vertex_index + resolution as u16);
//                             indices.push(vertex_index);
//                             indices.push(vertex_index + 1);
//                             indices.push(vertex_index + resolution as u16 + 1);
//                         }
//                     }
//                 }
//             }
//         }
//     }
//     let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
//         label: Some("Vertex Buffer"),
//         contents: bytemuck::cast_slice(&vertices),
//         usage: BufferUsages::VERTEX,
//     });
//     let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
//         label: Some("Vertex Buffer"),
//         contents: bytemuck::cast_slice(&indices),
//         usage: BufferUsages::INDEX,
//     });
//
//     Mesh {
//         vertex_buffer,
//         index_buffer,
//         index_length: indices.len() as u32,
//     }
// }
//
// fn map_cube_to_sphere(cube_point: Vector3<f32>) -> Vector3<f32> {
//     let x_square = cube_point.x * cube_point.x;
//     let y_square = cube_point.y * cube_point.y;
//     let z_square = cube_point.z * cube_point.z;
//     let x = cube_point.x * (1.0 - (y_square + z_square) / 2.0 + (y_square * z_square) / 3.0).sqrt();
//     let y = cube_point.y * (1.0 - (z_square + x_square) / 2.0 + (z_square * x_square) / 3.0).sqrt();
//     let z = cube_point.z * (1.0 - (y_square + x_square) / 2.0 + (y_square * x_square) / 3.0).sqrt();
//
//     Vector3::new(x, y, z)
// }
