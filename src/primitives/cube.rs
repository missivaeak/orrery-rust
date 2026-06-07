use std::ops::Div;

use cgmath::{ElementWise, Vector2, Vector3};

use crate::helpers::rendering::{MeshData, Vertex};

pub fn get_cube_mesh_data() -> MeshData {
    let positions = cube_positions();
    let normals = cube_normals();
    let uvs = cube_uvs();
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
pub fn cube_positions() -> Vec<Vector3<f32>> {
    [
        // front (0.0, 0.0, 1.0)
        Vector3::new(-1.0, -1.0, 1.0),
        Vector3::new(1.0, -1.0, 1.0),
        Vector3::new(-1.0, 1.0, 1.0),
        Vector3::new(-1.0, 1.0, 1.0),
        Vector3::new(1.0, -1.0, 1.0),
        Vector3::new(1.0, 1.0, 1.0),
        // right (1.0, 0.0, 0.0)
        Vector3::new(1.0, -1.0, 1.0),
        Vector3::new(1.0, -1.0, -1.0),
        Vector3::new(1.0, 1.0, 1.0),
        Vector3::new(1.0, 1.0, 1.0),
        Vector3::new(1.0, -1.0, -1.0),
        Vector3::new(1.0, 1.0, -1.0),
        // back (0.0, 0.0, -1.0)
        Vector3::new(1.0, -1.0, -1.0),
        Vector3::new(-1.0, -1.0, -1.0),
        Vector3::new(1.0, 1.0, -1.0),
        Vector3::new(1.0, 1.0, -1.0),
        Vector3::new(-1.0, -1.0, -1.0),
        Vector3::new(-1.0, 1.0, -1.0),
        // left (-1.0, 0.0, 0.0)
        Vector3::new(-1.0, -1.0, -1.0),
        Vector3::new(-1.0, -1.0, 1.0),
        Vector3::new(-1.0, 1.0, -1.0),
        Vector3::new(-1.0, 1.0, -1.0),
        Vector3::new(-1.0, -1.0, 1.0),
        Vector3::new(-1.0, 1.0, 1.0),
        // top (0.0, 1.0, 0.0)
        Vector3::new(-1.0, 1.0, 1.0),
        Vector3::new(1.0, 1.0, 1.0),
        Vector3::new(-1.0, 1.0, -1.0),
        Vector3::new(-1.0, 1.0, -1.0),
        Vector3::new(1.0, 1.0, 1.0),
        Vector3::new(1.0, 1.0, -1.0),
        // bottom (0.0, -1.0, 0.0)
        Vector3::new(-1.0, -1.0, -1.0),
        Vector3::new(1.0, -1.0, -1.0),
        Vector3::new(-1.0, -1.0, 1.0),
        Vector3::new(-1.0, -1.0, 1.0),
        Vector3::new(1.0, -1.0, -1.0),
        Vector3::new(1.0, -1.0, 1.0),
    ]
    .to_vec()
}

pub fn cube_normals() -> Vec<Vector3<f32>> {
    [
        // front (0.0, 0.0, 1.0)
        Vector3::new(0.0, 0.0, 1.0),
        Vector3::new(0.0, 0.0, 1.0),
        Vector3::new(0.0, 0.0, 1.0),
        Vector3::new(0.0, 0.0, 1.0),
        Vector3::new(0.0, 0.0, 1.0),
        Vector3::new(0.0, 0.0, 1.0),
        // right (1.0, 0.0, 0.0)
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        // back (0.0, 0.0, -1.0)
        Vector3::new(0.0, 0.0, -1.0),
        Vector3::new(0.0, 0.0, -1.0),
        Vector3::new(0.0, 0.0, -1.0),
        Vector3::new(0.0, 0.0, -1.0),
        Vector3::new(0.0, 0.0, -1.0),
        Vector3::new(0.0, 0.0, -1.0),
        // left (-1.0, 0.0, 0.0)
        Vector3::new(-1.0, 0.0, 0.0),
        Vector3::new(-1.0, 0.0, 0.0),
        Vector3::new(-1.0, 0.0, 0.0),
        Vector3::new(-1.0, 0.0, 0.0),
        Vector3::new(-1.0, 0.0, 0.0),
        Vector3::new(-1.0, 0.0, 0.0),
        // top (0.0, 1.0, 0.0)
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        // bottom (0.0, -1.0, 0.0)
        Vector3::new(0.0, -1.0, 0.0),
        Vector3::new(0.0, -1.0, 0.0),
        Vector3::new(0.0, -1.0, 0.0),
        Vector3::new(0.0, -1.0, 0.0),
        Vector3::new(0.0, -1.0, 0.0),
        Vector3::new(0.0, -1.0, 0.0),
    ]
    .to_vec()
}

pub fn cube_uvs() -> Vec<Vector2<f32>> {
    [
        // front (0.0, 0.0, 1.0)
        Vector2::new(0.0, 0.0),
        Vector2::new(1.0, 0.0),
        Vector2::new(0.0, 1.0),
        Vector2::new(0.0, 1.0),
        Vector2::new(1.0, 0.0),
        Vector2::new(1.0, 1.0),
        // right (1.0, 0.0, 0.0)
        Vector2::new(0.0, 1.0),
        Vector2::new(0.0, 0.0),
        Vector2::new(1.0, 1.0),
        Vector2::new(1.0, 1.0),
        Vector2::new(0.0, 0.0),
        Vector2::new(1.0, 0.0),
        // back (0.0, 0.0, -1.0)
        Vector2::new(0.0, 1.0),
        Vector2::new(1.0, 1.0),
        Vector2::new(0.0, 0.0),
        Vector2::new(0.0, 0.0),
        Vector2::new(1.0, 1.0),
        Vector2::new(1.0, 0.0),
        // left (-1.0, 0.0, 0.0)
        Vector2::new(1.0, 1.0),
        Vector2::new(1.0, 0.0),
        Vector2::new(0.0, 1.0),
        Vector2::new(0.0, 1.0),
        Vector2::new(1.0, 0.0),
        Vector2::new(0.0, 0.0),
        // top (0.0, 1.0, 0.0)
        Vector2::new(0.0, 1.0),
        Vector2::new(1.0, 1.0),
        Vector2::new(0.0, 0.0),
        Vector2::new(0.0, 0.0),
        Vector2::new(1.0, 1.0),
        Vector2::new(1.0, 0.0),
        // bottom (0.0, -1.0, 0.0)
        Vector2::new(1.0, 1.0),
        Vector2::new(0.0, 1.0),
        Vector2::new(1.0, 0.0),
        Vector2::new(1.0, 0.0),
        Vector2::new(0.0, 1.0),
        Vector2::new(0.0, 0.0),
    ]
    .to_vec()
}

// pub fn cube_colors() -> Vec<Vector4<f32>> {
//     [
//         // front - blue
//         Vector4::new(0.0, 0.0, 1.0, 1.0),
//         Vector4::new(0.0, 0.0, 1.0, 1.0),
//         Vector4::new(0.0, 0.0, 1.0, 1.0),
//         Vector4::new(0.0, 0.0, 1.0, 1.0),
//         Vector4::new(0.0, 0.0, 1.0, 1.0),
//         Vector4::new(0.0, 0.0, 1.0, 1.0),
//         // right - red
//         Vector4::new(1.0, 0.0, 0.0, 1.0),
//         Vector4::new(1.0, 0.0, 0.0, 1.0),
//         Vector4::new(1.0, 0.0, 0.0, 1.0),
//         Vector4::new(1.0, 0.0, 0.0, 1.0),
//         Vector4::new(1.0, 0.0, 0.0, 1.0),
//         Vector4::new(1.0, 0.0, 0.0, 1.0),
//         // back - yellow
//         Vector4::new(1.0, 1.0, 0.0, 1.0),
//         Vector4::new(1.0, 1.0, 0.0, 1.0),
//         Vector4::new(1.0, 1.0, 0.0, 1.0),
//         Vector4::new(1.0, 1.0, 0.0, 1.0),
//         Vector4::new(1.0, 1.0, 0.0, 1.0),
//         Vector4::new(1.0, 1.0, 0.0, 1.0),
//         // left - aqua
//         Vector4::new(0.0, 1.0, 1.0, 1.0),
//         Vector4::new(0.0, 1.0, 1.0, 1.0),
//         Vector4::new(0.0, 1.0, 1.0, 1.0),
//         Vector4::new(0.0, 1.0, 1.0, 1.0),
//         Vector4::new(0.0, 1.0, 1.0, 1.0),
//         Vector4::new(0.0, 1.0, 1.0, 1.0),
//         // top - green
//         Vector4::new(0.0, 1.0, 0.0, 1.0),
//         Vector4::new(0.0, 1.0, 0.0, 1.0),
//         Vector4::new(0.0, 1.0, 0.0, 1.0),
//         Vector4::new(0.0, 1.0, 0.0, 1.0),
//         Vector4::new(0.0, 1.0, 0.0, 1.0),
//         Vector4::new(0.0, 1.0, 0.0, 1.0),
//         // bottom - fuchsia
//         Vector4::new(1.0, 0.0, 1.0, 1.0),
//         Vector4::new(1.0, 0.0, 1.0, 1.0),
//         Vector4::new(1.0, 0.0, 1.0, 1.0),
//         Vector4::new(1.0, 0.0, 1.0, 1.0),
//         Vector4::new(1.0, 0.0, 1.0, 1.0),
//         Vector4::new(1.0, 0.0, 1.0, 1.0),
//     ]
//     .to_vec()
// }
