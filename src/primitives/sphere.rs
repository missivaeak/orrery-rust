use std::ops::Div;

use cgmath::{Deg, ElementWise, Vector2, Vector3};

use crate::helpers::{
    math::spherical_to_cartesian,
    rendering::{MeshData, Vertex},
};

pub fn sphere_data(radius: f32, u: usize, v: usize) -> MeshData {
    let mut positions: Vec<Vector3<f32>> = Vec::with_capacity(4 * (u - 1) * (v - 1));
    let mut normals: Vec<Vector3<f32>> = Vec::with_capacity(4 * (u - 1) * (v - 1));
    let mut uvs: Vec<Vector2<f32>> = Vec::with_capacity(4 * (u - 1) * (v - 1));

    for i in 0..u - 1 {
        for j in 0..v - 1 {
            let theta = i as f32 * 180.0 / (u as f32 - 1.0);
            let phi = j as f32 * 360.0 / (v as f32 - 1.0);
            let theta1 = (i as f32 + 1.0) * 180.0 / (u as f32 - 1.0);
            let phi1 = (j as f32 + 1.0) * 360.0 / (v as f32 - 1.0);
            let p0 = spherical_to_cartesian(radius, Deg(theta), Deg(phi));
            let p1 = spherical_to_cartesian(radius, Deg(theta1), Deg(phi));
            let p2 = spherical_to_cartesian(radius, Deg(theta1), Deg(phi1));
            let p3 = spherical_to_cartesian(radius, Deg(theta), Deg(phi1));

            // positions
            positions.push(p0);
            positions.push(p1);
            positions.push(p3);
            positions.push(p1);
            positions.push(p2);
            positions.push(p3);

            // normals
            normals.push(Vector3::new(p0[0] / radius, p0[1] / radius, p0[2] / radius));
            normals.push(Vector3::new(p1[0] / radius, p1[1] / radius, p1[2] / radius));
            normals.push(Vector3::new(p3[0] / radius, p3[1] / radius, p3[2] / radius));
            normals.push(Vector3::new(p1[0] / radius, p1[1] / radius, p1[2] / radius));
            normals.push(Vector3::new(p2[0] / radius, p2[1] / radius, p2[2] / radius));
            normals.push(Vector3::new(p3[0] / radius, p3[1] / radius, p3[2] / radius));

            // uvs
            uvs.push(Vector2::new(1.0, 1.0));
            uvs.push(Vector2::new(1.0, 1.0));
            uvs.push(Vector2::new(1.0, 1.0));
            uvs.push(Vector2::new(1.0, 1.0));
            uvs.push(Vector2::new(1.0, 1.0));
            uvs.push(Vector2::new(1.0, 1.0));
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
