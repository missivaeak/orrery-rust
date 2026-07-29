use cgmath::{
    Deg, InnerSpace, Matrix4, Quaternion, Rad, Rotation3, SquareMatrix, Transform, Vector3,
    VectorSpace, num_traits::*,
};

use wgpu::{
    BufferDescriptor, BufferUsages, Device, Queue,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    helpers::{
        constants::EARTH_RADIUS,
        entity::{Entity, UpdateDescriptor},
        math::it_mat4,
        mesh::{Mesh, MeshData},
        object::{Object, ObjectFragmentUniform, ObjectVertexUniform},
        vertex::Vertex,
    },
    renderer::RenderGroupType,
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
        let scale = Vector3::new(1.0, 1.0, 1.0) * EARTH_RADIUS;
        let axis = Vector3::new(1.0, -1.0, 0.0).normalize();
        let angle = Rad((1.0 / 3.0.sqrt()).acos());
        let rotation = Quaternion::from_axis_angle(axis, angle);

        let faces = vec![
            Face::new(Vector3::unit_x()),
            Face::new(Vector3::unit_x() * -1.0),
            Face::new(Vector3::unit_y()),
            Face::new(Vector3::unit_y() * -1.0),
            Face::new(Vector3::unit_z()),
            Face::new(Vector3::unit_z() * -1.0),
        ];
        let model_mat = Matrix4::from_translation(Vector3::new(-7.0, -7.0, -3.0));

        let meshes: Vec<Mesh> = faces
            .iter()
            .map(|face| {
                let MeshData {
                    vertices: _vertices,
                    indices,
                } = face.get_mesh_data(Vector3::unit_y(), -Vector3::unit_y());
                let vertex_buffer = device.create_buffer(&BufferDescriptor {
                    label: Some("Vertex Buffer"),
                    size: 14272000,
                    usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                let index_buffer = device.create_buffer(&BufferDescriptor {
                    label: Some("Vertex Buffer"),
                    size: 383376,
                    usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });

                Mesh {
                    vertex_buffer,
                    index_buffer,
                    index_length: indices.len() as u32,
                }
            })
            .collect();

        let vertex_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Object Vertex Uniform Buffer"),
            contents: bytemuck::bytes_of(&ObjectVertexUniform {
                model_mat: model_mat.into(),
                normal_mat: it_mat4(model_mat).into(),
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

        let object = Object {
            render_group_type: RenderGroupType::Lit,
            vertex_uniform_buffer,
            fragment_uniform_buffer,
            meshes,
        };

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

    /// Maximum and minium subdivision depth.
    pub max_depth: u32,
    pub min_depth: u32,
}

impl Face {
    pub fn new(normal: Vector3<f32>) -> Self {
        Self {
            normal,
            tangent: normal.yzx(),
            max_depth: 10,
            min_depth: 2,
        }
    }

    fn get_mesh_data(
        &self,
        camera_position: Vector3<f32>,
        camera_direction: Vector3<f32>,
    ) -> MeshData {
        let binormal = self.normal.cross(self.tangent);
        // let colour = ilerp(self.normal, -1.0, 1.0).xyzz();
        let mut mesh_data = MeshData {
            vertices: Vec::with_capacity(4),
            indices: Vec::with_capacity(6),
        };

        self.descend_node(
            [
                (self.normal - self.tangent - binormal),
                (self.normal + self.tangent - binormal),
                (self.normal + self.tangent + binormal),
                (self.normal - self.tangent + binormal),
            ],
            0,
            camera_position,
            camera_direction,
            &mut mesh_data,
        );

        mesh_data
    }

    fn descend_node(
        &self,
        positions: [Vector3<f32>; 4],
        depth: u32,
        camera_position: Vector3<f32>,
        camera_direction: Vector3<f32>,
        mesh_data: &mut MeshData,
    ) {
        let top_left = positions[0].normalize();
        let top_right = positions[1].normalize();
        let bottom_right = positions[2].normalize();
        let bottom_left = positions[3].normalize();
        let side_length = (top_left - top_right).magnitude();
        let middle = positions[0].lerp(positions[2], 0.5).normalize();
        let distance = (camera_position - middle).magnitude2();
        let area = side_length.pow(2);
        let screen_space_factor = area / distance;
        let threshold: f32 = 0.005;

        let horizon_cos = 1.0 / camera_position.magnitude();

        if depth > 0
            && [top_left, top_right, bottom_right, bottom_left]
                .iter()
                .all(|corner| corner.dot(camera_position.normalize()) < horizon_cos)
        {
            return;
        }

        if depth < self.min_depth || (depth < self.max_depth && screen_space_factor > threshold) {
            let top = positions[0].lerp(positions[1], 0.5).normalize();
            let right = positions[1].lerp(positions[2], 0.5).normalize();
            let bottom = positions[2].lerp(positions[3], 0.5).normalize();
            let left = positions[3].lerp(positions[0], 0.5).normalize();
            self.descend_node(
                [top_left, top, middle, left],
                depth + 1,
                camera_position,
                camera_direction,
                mesh_data,
            );
            self.descend_node(
                [top, top_right, right, middle],
                depth + 1,
                camera_position,
                camera_direction,
                mesh_data,
            );
            self.descend_node(
                [middle, right, bottom_right, bottom],
                depth + 1,
                camera_position,
                camera_direction,
                mesh_data,
            );
            self.descend_node(
                [left, middle, bottom, bottom_left],
                depth + 1,
                camera_position,
                camera_direction,
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

        for position in [top_left, top_right, bottom_right, bottom_left] {
            let position4 = position.extend(1.0);
            mesh_data.vertices.push(Vertex {
                position: position4.into(),
                normal: position4.into(),
                uv: position4.into(),
                colour: [1.0, 1.0, 1.0, 1.0],
            });
        }

        mesh_data.indices.push(vertex_count);
        mesh_data.indices.push(vertex_count + 1);
        mesh_data.indices.push(vertex_count + 2);
        mesh_data.indices.push(vertex_count + 2);
        mesh_data.indices.push(vertex_count + 3);
        mesh_data.indices.push(vertex_count);
    }
}

impl Entity for CubePlanet {
    fn update(&mut self, queue: &Queue, update_descriptor: &UpdateDescriptor) -> Result<(), ()> {
        let dt = update_descriptor.app.delta_time.as_secs_f32();

        // 90 degrees per second
        let speed = 10.0;

        let delta_rotation = Quaternion::from_angle_z(Deg(speed * dt));

        self.rotation = delta_rotation * self.rotation;

        let model_mat = Matrix4::from_translation(self.translation)
            * Matrix4::from(self.rotation)
            * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);

        // let mut meshes = Vec::with_capacity(6);

        // pull out vector3 out of self.quad_tree_roots here
        // populate meshes with vertices and indices

        let (camera_position, camera_direction) = if update_descriptor.gui.lod_probe_enabled {
            (
                Vector3 {
                    x: EARTH_RADIUS * 1.1,
                    y: 0.0,
                    z: 0.0,
                },
                -Vector3::unit_x(),
            )
        } else {
            (
                update_descriptor.controls.camera_position,
                update_descriptor.controls.camera_direction,
            )
        };

        if let Some(model_mat_i) = model_mat.invert() {
            for (i, mesh_data) in self
                .faces
                .iter()
                // .map(|face| face.get_mesh_data(update_descriptor.camera_position))
                .map(|face| {
                    face.get_mesh_data(
                        model_mat_i.transform_vector(camera_position),
                        model_mat_i.transform_vector(camera_direction),
                    )
                })
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
        } else {
            Err(())
        }
    }

    fn get_object(&self) -> &Object {
        &self.object
    }
}

#[allow(unused)]
fn map_cube_to_sphere(cube_point: Vector3<f32>) -> Vector3<f32> {
    let x_2: f32 = cube_point.x.pow(2);
    let y_2: f32 = cube_point.y.pow(2);
    let z_2: f32 = cube_point.z.pow(2);

    let sphere_cube = Vector3 {
        x: cube_point.x * (1.0 - (y_2 + z_2) / 2.0 + (y_2 * z_2) / 3.0).sqrt(),
        y: cube_point.y * (1.0 - (z_2 + x_2) / 2.0 + (z_2 * x_2) / 3.0).sqrt(),
        z: cube_point.z * (1.0 - (x_2 + y_2) / 2.0 + (x_2 * y_2) / 3.0).sqrt(),
    };

    // println!("{:?}", sphere_cube.magnitude());

    sphere_cube
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
