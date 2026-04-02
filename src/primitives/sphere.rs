use cgmath::{Deg, Vector2, Vector3};

use crate::math::spherical_to_cartesian;

type SphereData = (Vec<Vector3<f32>>, Vec<Vector3<f32>>, Vec<Vector2<f32>>);

pub fn sphere_data(radius: f32, u: usize, v: usize) -> SphereData {
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
    (positions, normals, uvs)
}
