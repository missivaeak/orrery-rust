use cgmath::{Vector2, Vector3};

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
