use std::f32::consts::TAU;

use cgmath::{Angle, Deg, Matrix, Matrix4, Point3, Rad, SquareMatrix, Vector3, ortho, perspective};

// I want to use blender coordinates, right handed, Z up, Y forward, X right

// change from z{0,1} to z{-1,1}
#[rustfmt::skip]
pub const NORMALISATION_MATRIX: Matrix4<f32> = Matrix4::new(
  1.0, 0.0, 0.0, 0.0,
  0.0, 1.0, 0.0, 0.0,
  0.0, 0.0, 0.5, 0.0,
  0.0, 0.0, 0.5, 1.0
);

pub fn create_view(camera_position: Vector3<f32>, camera_direction: Vector3<f32>) -> Matrix4<f32> {
    let camera_position_point =
        Point3::new(camera_position.x, camera_position.y, camera_position.z);
    Matrix4::look_at_rh(
        camera_position_point,
        camera_position_point + camera_direction,
        Vector3::unit_z(),
    )
}

pub fn create_projection(aspect: f32, is_perspective: bool) -> Matrix4<f32> {
    match is_perspective {
        true => NORMALISATION_MATRIX * perspective(Rad(TAU / 10.0), aspect, 0.1, 10000.0),
        false => NORMALISATION_MATRIX * ortho(-4.0, 4.0, -3.0, 3.0, -1.0, 6.0),
    }
}

pub fn create_model(
    translation: Vector3<f32>,
    rotation: Vector3<f32>,
    scaling: Vector3<f32>,
) -> Matrix4<f32> {
    let trans_mat = Matrix4::from_translation(translation);
    let rotate_mat_x = Matrix4::from_angle_x(Rad(rotation.x));
    let rotate_mat_y = Matrix4::from_angle_y(Rad(rotation.y));
    let rotate_mat_z = Matrix4::from_angle_z(Rad(rotation.z));
    let scale_mat = Matrix4::from_nonuniform_scale(scaling.x, scaling.y, scaling.z);

    trans_mat * rotate_mat_z * rotate_mat_y * rotate_mat_x * scale_mat
}

pub fn it_mat4(mat4: Matrix4<f32>) -> Matrix4<f32> {
    mat4.clone()
        .invert()
        .expect("Unable to invert model matrix")
        .transpose()
}

#[allow(unused, dead_code)]
pub fn spherical_to_cartesian(radius: f32, theta: Deg<f32>, phi: Deg<f32>) -> Vector3<f32> {
    let x = radius * theta.sin() * phi.cos();
    let y = radius * theta.cos();
    let z = -radius * theta.sin() * phi.sin();

    Vector3::new(x, y, z)
}

#[allow(unused, dead_code)]
pub struct Aabb3 {
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
    pub z_min: f32,
    pub z_max: f32,
}

#[allow(unused, dead_code)]
pub struct Aabb2 {
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
}

fn ilerp_element(input: f32, min: f32, max: f32) -> f32 {
    let diff = max - min;
    let t = (input - min) * max;
    t / diff
}

#[allow(unused)]
pub fn ilerp(input: Vector3<f32>, min: f32, max: f32) -> Vector3<f32> {
    Vector3 {
        x: ilerp_element(input.x, min, max),
        y: ilerp_element(input.y, min, max),
        z: ilerp_element(input.z, min, max),
    }
}
