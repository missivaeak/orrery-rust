use bytemuck::{Pod, Zeroable};
use cgmath::{Vector2, Vector3, Vector4};
use wgpu::{BufferAddress, VertexBufferLayout, VertexStepMode};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 4],
    pub normal: [f32; 4],
    pub uv: [f32; 4],
    pub colour: [f32; 4],
}

impl Vertex {
    pub fn new(
        position: Vector3<f32>,
        normal: Vector3<f32>,
        uv: Vector2<f32>,
        colour: Vector4<f32>,
    ) -> Self {
        Self {
            position: [position.x, position.y, position.z, 1.0],
            normal: [normal.x, normal.y, normal.z, 1.0],
            uv: [uv.x, uv.y, 1.0, 1.0],
            colour: [colour.x, colour.y, colour.z, colour.w],
        }
    }
    const ATTRIBUTES: [wgpu::VertexAttribute; 4] =
        wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4, 3 => Float32x4];
    pub fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }

    // pub fn lerp(&self, other: Vertex, t: f32) -> Vertex {
    //     let position = self.get_position().lerp(other.get_position(), t);
    //     let normal = self.get_normal().lerp(other.get_normal(), t).normalize();
    //     let uv = self.get_uv().lerp(other.get_uv(), t);
    //     let colour = self.get_colour().lerp(other.get_colour(), t);
    //
    //     Vertex::new(position, normal, uv, colour)
    // }
    //
    // pub fn slerp(&self, other: Vertex, t: f32) -> Vertex {
    //     let position = slerp(self.get_position(), other.get_position(), t);
    //     let normal = slerp(self.get_normal(), other.get_normal(), t);
    //     let uv = slerp(self.get_uv(), other.get_uv(), t);
    //     let colour = slerp(self.get_colour(), other.get_colour(), t);
    //
    //     Vertex::new(position, normal, uv, colour)
    // }
    //
    // pub fn get_position(&self) -> Vector3<f32> {
    //     Vector3::new(self.position[0], self.position[1], self.position[2])
    // }
    //
    // pub fn get_normal(&self) -> Vector3<f32> {
    //     Vector3::new(self.normal[0], self.normal[1], self.normal[2])
    // }
    //
    // pub fn get_uv(&self) -> Vector2<f32> {
    //     Vector2::new(self.uv[0], self.uv[1])
    // }
    //
    // pub fn get_colour(&self) -> Vector4<f32> {
    //     Vector4::new(
    //         self.colour[0],
    //         self.colour[1],
    //         self.colour[2],
    //         self.colour[3],
    //     )
    // }
}
