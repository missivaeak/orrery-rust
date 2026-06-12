use std::ops::Div;

use cgmath::{ElementWise, InnerSpace, Matrix4, Quaternion, Vector2, Vector3};
use egui::emath::inverse_lerp;

use crate::helpers::{
    entity::Entity,
    math::{Aabb2, Aabb3, it_mat4},
    mesh::MeshData,
    object::{Object, ObjectOptions, ObjectVertexUniform},
    vertex::Vertex,
};

use std::time::Duration;

use wgpu::{Device, Queue};

pub struct WavySurface {
    object: Object,
    translation: Vector3<f32>,
    scale: Vector3<f32>,
    rotation: Quaternion<f32>,
}

impl WavySurface {
    pub fn new(device: &Device) -> Self {
        let translation = Vector3::new(-50.0, -90.0, -30.0);
        let scale = Vector3::new(1.0, 1.0, 1.0);
        let rotation = Quaternion::new(1.0, 0.0, 0.0, 0.0);
        let object =
            Object::from_mesh_datas(device, vec![surface_data()], ObjectOptions::default());
        Self {
            object,
            translation,
            scale,
            rotation,
        }
    }
}

impl Entity for WavySurface {
    fn update(
        &mut self,
        queue: &Queue,
        _time_elapsed: Duration,
        _delta_time: Duration,
    ) -> Result<(), ()> {
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

pub fn sinc(x: f32, y: f32) -> [f32; 3] {
    let r = (x * x + y * y).sqrt();
    let z = if r == 0.0 { 1.0 } else { r.sin() / r };
    [x, y, z]
}

fn map_to_height(height_range: [f32; 2], height: f32) -> f32 {
    inverse_lerp(height_range[0]..=height_range[1], height).expect("Failed to map to height range")
}

pub fn surface_data() -> MeshData {
    let aabb = Aabb2 {
        x_min: -8.0,
        x_max: 8.0,
        y_min: -8.0,
        y_max: 8.0,
    };
    let x_count = 30;
    let y_count = 30;
    let (grid, height_range) = get_surface_grid(&sinc, aabb, x_count, y_count, 40.0, -2.00);
    let mut positions: Vec<Vector3<f32>> = Vec::with_capacity(4 * (x_count - 1) * (y_count - 1));
    let mut normals: Vec<Vector3<f32>> = Vec::with_capacity(4 * (x_count - 1) * (y_count - 1));
    let mut uvs: Vec<Vector2<f32>> = Vec::with_capacity(4 * (x_count - 1) * (y_count - 1));
    for i in 0..x_count - 1 {
        for j in 0..y_count - 1 {
            let v0: Vector3<f32> = grid[i][j].into();
            let v1: Vector3<f32> = grid[i][j + 1].into();
            let v2: Vector3<f32> = grid[i + 1][j + 1].into();
            let v3: Vector3<f32> = grid[i + 1][j].into();

            let diff02 = Vector3::new(v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]);
            let diff12 = Vector3::new(v2[0] - v1[0], v2[1] - v1[1], v2[2] - v1[2]);
            let diff03 = Vector3::new(v3[0] - v0[0], v3[1] - v0[1], v3[2] - v0[2]);
            let normal1 = (diff02.cross(diff12)).normalize();
            let normal2 = (diff02.cross(diff03)).normalize();

            positions.push(v0);
            positions.push(v1);
            positions.push(v2);
            normals.push(normal1);
            normals.push(normal1);
            normals.push(normal1);
            uvs.push(Vector2::new(0.0, map_to_height(height_range, v0.z)));
            uvs.push(Vector2::new(0.0, map_to_height(height_range, v1.z)));
            uvs.push(Vector2::new(0.0, map_to_height(height_range, v2.z)));

            positions.push(v2);
            positions.push(v3);
            positions.push(v0);
            normals.push(normal2);
            normals.push(normal2);
            normals.push(normal2);
            uvs.push(Vector2::new(0.0, map_to_height(height_range, v2.z)));
            uvs.push(Vector2::new(0.0, map_to_height(height_range, v3.z)));
            uvs.push(Vector2::new(0.0, map_to_height(height_range, v0.z)));
        }
    }

    let vertices = {
        let mut data: Vec<Vertex> = Vec::with_capacity(positions.len());
        for i in 0..positions.len() {
            data.push(Vertex::new(
                positions[i],
                normals[i],
                uvs[i],
                normals[i].add_element_wise(1.0).div(2.0).extend(1.0),
            ));
        }
        data
    };
    let indices = {
        let mut data: Vec<u16> = Vec::with_capacity(positions.len());
        for i in 0..positions.len() {
            data.push(i as u16);
        }
        data
    };

    MeshData { vertices, indices }
}

pub fn get_surface_grid(
    func: &dyn Fn(f32, f32) -> [f32; 3],
    aabb: Aabb2,
    x_count: usize,
    y_count: usize,
    scale: f32,
    aspect: f32,
) -> (Vec<Vec<[f32; 3]>>, [f32; 2]) {
    let Aabb2 {
        x_min,
        x_max,
        y_min,
        y_max,
    } = aabb;
    let x_offset: f32 = (x_max - x_min) / (x_count as f32 - 1.0);
    let y_offset: f32 = (y_max - y_min) / (y_count as f32 - 1.0);

    let mut z_min: f32 = 0.0;
    let mut z_max: f32 = 0.0;
    let mut grid: Vec<Vec<[f32; 3]>> = vec![vec![Default::default(); y_count]; x_count];

    for (i, col) in grid.iter_mut().enumerate() {
        let x = x_min + i as f32 * x_offset;

        for (j, point) in col.iter_mut().enumerate() {
            let y = y_min + j as f32 * y_offset;
            *point = func(x, y);
            z_min = if point[2] < z_min { point[2] } else { z_min };
            z_max = if point[2] > z_max { point[2] } else { z_max };
        }
    }
    let z_min_scaled = z_min - (1.0 - aspect) * (z_max - z_min);
    let z_max_scaled = z_max + (1.0 - aspect) * (z_max - z_min);
    let aabb = Aabb3 {
        x_min,
        x_max,
        y_min,
        y_max,
        z_min: z_min_scaled,
        z_max: z_max_scaled,
    };

    for col in &mut grid {
        for point in col {
            *point = normalise(*point, &aabb, scale);
        }
    }

    let c_min = normalise([0.0, 0.0, z_min_scaled], &aabb, scale);
    let c_max = normalise([0.0, 0.0, z_max_scaled], &aabb, scale);

    (grid, [c_min[2], c_max[2]])
}

fn normalise(point: [f32; 3], aabb: &Aabb3, scale: f32) -> [f32; 3] {
    let Aabb3 {
        x_min,
        x_max,
        y_min,
        y_max,
        z_min,
        z_max,
    } = aabb;
    [
        scale * ((point[0] - x_min) / (x_max - x_min)),
        scale * ((point[1] - y_min) / (y_max - y_min)),
        scale * ((point[2] - z_min) / (z_max - z_min)),
    ]
}
