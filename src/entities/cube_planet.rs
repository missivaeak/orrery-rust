use cgmath::{
    Deg, InnerSpace, Matrix4, Quaternion, Rad, Rotation3, SquareMatrix, Transform, Vector2,
    Vector3, Vector4, num_traits::*,
};

use wgpu::{Device, Queue};

use crate::helpers::{
    entity::{Entity, UpdateDescriptor},
    math::{ilerp, it_mat4},
    mesh::MeshData,
    object::{Object, ObjectOptions, ObjectVertexUniform},
    vertex::Vertex,
};

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

        let mesh_datas = faces
            .iter()
            .map(|face| face.get_mesh_data(Vector3::unit_y()))
            .collect();

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
            max_depth: 7,
        }
    }

    fn get_mesh_data(&self, camera_position: Vector3<f32>) -> MeshData {
        let binormal = self.normal.cross(self.tangent);
        let colour = ilerp(self.normal, -1.0, 1.0).xyzz();
        let mut mesh_data = MeshData {
            vertices: Vec::with_capacity(4),
            indices: Vec::with_capacity(6),
        };

        self.descend_node(
            [
                Vertex::new(
                    self.normal - self.tangent - binormal,
                    self.normal,
                    Vector2::new(-self.radius, -self.radius),
                    colour,
                ),
                Vertex::new(
                    self.normal + self.tangent - binormal,
                    self.normal,
                    Vector2::new(self.radius, -self.radius),
                    colour,
                ),
                Vertex::new(
                    self.normal + self.tangent + binormal,
                    self.normal,
                    Vector2::new(self.radius, self.radius),
                    colour,
                ),
                Vertex::new(
                    self.normal - self.tangent + binormal,
                    self.normal,
                    Vector2::new(-self.radius, self.radius),
                    colour,
                ),
            ],
            0,
            camera_position,
            &mut mesh_data,
        );

        for vertex in mesh_data.vertices.iter_mut() {
            let position = Vector3::new(vertex.position[0], vertex.position[1], vertex.position[2])
                .normalize();

            vertex.position[0] = position.x;
            vertex.position[1] = position.y;
            vertex.position[2] = position.z;

            let normal = Vector3::new(vertex.position[0], vertex.position[1], vertex.position[2])
                .normalize();

            vertex.normal[0] = normal.x;
            vertex.normal[1] = normal.y;
            vertex.normal[2] = normal.z;
        }

        mesh_data
    }

    fn descend_node(
        &self,
        vertices: [Vertex; 4],
        depth: u32,
        camera_position: Vector3<f32>,
        mesh_data: &mut MeshData,
    ) {
        let top_left = vertices[0];
        let top_right = vertices[1];
        let bottom_right = vertices[2];
        let bottom_left = vertices[3];
        let middle = top_left.lerp(bottom_right, 0.5);
        let distance = (camera_position - middle.get_position()).magnitude();
        if depth < self.max_depth && distance < 0.8 {
            let top = top_left.lerp(top_right, 0.5);
            let right = bottom_right.lerp(top_right, 0.5);
            let bottom = bottom_left.lerp(bottom_right, 0.5);
            let left = top_left.lerp(bottom_left, 0.5);
            self.descend_node(
                [top_left, top, middle, left],
                depth + 1,
                camera_position,
                mesh_data,
            );
            self.descend_node(
                [top, top_right, right, middle],
                depth + 1,
                camera_position,
                mesh_data,
            );
            self.descend_node(
                [middle, right, bottom_right, bottom],
                depth + 1,
                camera_position,
                mesh_data,
            );
            self.descend_node(
                [left, middle, bottom, bottom_left],
                depth + 1,
                camera_position,
                mesh_data,
            );

            return;
        }

        let vertex_count = mesh_data.vertices.len() as u16;

        if mesh_data.vertices.capacity() - mesh_data.vertices.len() < 4 {
            mesh_data
                .vertices
                .reserve(mesh_data.vertices.capacity() * 4);
        }

        if mesh_data.indices.capacity() - mesh_data.indices.len() < 6 {
            mesh_data.indices.reserve(mesh_data.indices.capacity() * 4);
        }

        for vertex in vertices.iter() {
            mesh_data.vertices.push(*vertex);
        }

        mesh_data.indices.push(vertex_count + 0);
        mesh_data.indices.push(vertex_count + 1);
        mesh_data.indices.push(vertex_count + 2);
        mesh_data.indices.push(vertex_count + 2);
        mesh_data.indices.push(vertex_count + 3);
        mesh_data.indices.push(vertex_count + 0);
    }
}

impl Entity for CubePlanet {
    fn update(&mut self, queue: &Queue, update_descriptor: &UpdateDescriptor) -> Result<(), ()> {
        let dt = update_descriptor.delta_time.as_secs_f32();

        // 90 degrees per second
        let speed = 50.0;

        let delta_rotation = Quaternion::from_angle_z(Deg(speed * dt));

        self.rotation = delta_rotation * self.rotation;

        let model_mat = Matrix4::from_translation(self.translation)
            * Matrix4::from(self.rotation)
            * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);

        // let mut meshes = Vec::with_capacity(6);

        // pull out vector3 out of self.quad_tree_roots here
        // populate meshes with vertices and indices

        let camera_position_local = model_mat
            .invert()
            .expect("Failed to invert cube_planet model matrix")
            .transform_vector(Vector3::unit_y());

        for (i, mesh_data) in self
            .faces
            .iter()
            // .map(|face| face.get_mesh_data(update_descriptor.camera_position))
            .map(|face| face.get_mesh_data(camera_position_local))
            .enumerate()
        {
            let MeshData { vertices, indices } = mesh_data;
            queue.write_buffer(
                &self.object.meshes[i].vertex_buffer,
                0,
                bytemuck::cast_slice(&vertices),
            );
            queue.write_buffer(
                &self.object.meshes[i].index_buffer,
                0,
                bytemuck::cast_slice(&indices),
            );
            self.object.meshes[i].index_length = indices.len() as u32;
        }

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
