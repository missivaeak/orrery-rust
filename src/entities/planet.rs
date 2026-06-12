use std::time::Duration;

use cgmath::{Deg, InnerSpace, Matrix4, Quaternion, Rotation3, Vector3};

use wgpu::{Device, Queue};

use crate::helpers::{
    entity::Entity,
    math::it_mat4,
    mesh::MeshData,
    object::{Object, ObjectOptions, ObjectVertexUniform},
    vertex::Vertex,
};

pub struct Planet {
    object: Object,
    translation: Vector3<f32>,
    scale: Vector3<f32>,
    rotation: Quaternion<f32>,
}

impl Planet {
    pub fn new(device: &Device) -> Self {
        let translation = Vector3::new(-7.0, -7.0, -3.0);
        let scale = Vector3::new(1.0, 1.0, 1.0);
        let rotation = Quaternion::new(1.0, 0.0, 0.0, 0.0);

        let object = Object::from_mesh_datas(
            device,
            create_octasphere_mesh_data(5.0, 3),
            ObjectOptions {
                model_mat: Matrix4::from_translation(translation),
                ..Default::default()
            },
        );
        Self {
            object,
            translation,
            scale,
            rotation,
        }
    }
}

impl Entity for Planet {
    fn update(
        &mut self,
        queue: &Queue,
        _total_time: Duration,
        delta_time: Duration,
    ) -> Result<(), ()> {
        let dt = delta_time.as_secs_f32();

        // 90 degrees per second
        let speed = 10.0;

        let delta_rotation = Quaternion::from_angle_y(Deg(speed * dt));

        self.rotation = delta_rotation * self.rotation;

        let model_mat = Matrix4::from_translation(self.translation)
            * Matrix4::from(self.rotation)
            * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);

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

struct Face {
    vectors: Vec<Vector3<f32>>,
    children: Option<Vec<Face>>,
}

pub fn create_octasphere_mesh_data(radius: f32, lod: usize) -> Vec<MeshData> {
    let forward = Vector3::unit_x();
    let back = Vector3::unit_x() * -1.0;
    let right = Vector3::unit_y();
    let left = Vector3::unit_y() * -1.0;
    let up = Vector3::unit_z();
    let down = Vector3::unit_z() * -1.0;
    let faces = vec![
        create_face(forward, right, up, lod),
        create_face(forward, right, down, lod),
        create_face(forward, left, up, lod),
        create_face(forward, left, down, lod),
        create_face(back, right, up, lod),
        create_face(back, right, down, lod),
        create_face(back, left, up, lod),
        create_face(back, left, down, lod),
    ];

    let mut mesh_datas = Vec::with_capacity(6);

    for face in faces {
        let mesh_data = get_mesh_data(&face, radius);
        mesh_datas.push(mesh_data);
    }

    mesh_datas
}

fn get_mesh_data(face: &Face, radius: f32) -> MeshData {
    let mut mesh_data = MeshData {
        vertices: Vec::with_capacity(3),
        indices: Vec::with_capacity(3),
    };

    fn populate_mesh_data(face: &Face, radius: f32, mesh_data: &mut MeshData) {
        if let Some(children) = &face.children {
            for child in children {
                populate_mesh_data(child, radius, mesh_data);
            }
            return;
        }

        let vertex_count = mesh_data.vertices.len();

        if mesh_data.vertices.capacity() < 3 {
            mesh_data.vertices.reserve(vertex_count);
            mesh_data.indices.reserve(vertex_count);
        }

        for (i, vector) in face.vectors.iter().cloned().enumerate() {
            let vertex = Vertex::new(
                vector * radius,
                vector,
                vector.truncate(),
                vector.extend(1.0),
            );
            mesh_data.vertices.push(vertex);
            mesh_data.indices.push((i + vertex_count) as u16);
        }
    }

    populate_mesh_data(face, radius, &mut mesh_data);

    mesh_data
}

fn create_face(v1: Vector3<f32>, v2: Vector3<f32>, v3: Vector3<f32>, lod: usize) -> Face {
    let mut face = Face {
        vectors: vec![v1, v2, v3],
        children: None,
    };

    fn populate_face_data(face: &mut Face, lod: usize, depth: usize) {
        if depth == lod {
            return;
        }

        let v0 = face.vectors[0];
        let v1 = face.vectors[1];
        let v2 = face.vectors[2];
        let v01 = (v0 + v1).normalize();
        let v12 = (v1 + v2).normalize();
        let v20 = (v2 + v0).normalize();

        face.children = Some(vec![
            Face {
                vectors: vec![v0, v01, v20],
                children: None,
            },
            Face {
                vectors: vec![v1, v12, v01],
                children: None,
            },
            Face {
                vectors: vec![v2, v20, v12],
                children: None,
            },
            Face {
                vectors: vec![v01, v12, v20],
                children: None,
            },
        ]);

        if let Some(children) = &mut face.children {
            for child in children {
                populate_face_data(child, lod, depth + 1);
            }
        }
    }

    populate_face_data(&mut face, lod, 0);

    face
}
