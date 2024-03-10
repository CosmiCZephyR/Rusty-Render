use crate::math::{vector4f::Vec4F, matrix4::Mat4};

pub struct Camera {
    // Projection
    near: f32,
    far: f32,
    fov: f32, // Can't be more than 180.
    aspect_ratio: f32,

    // View
    pub position: Vec4F,
    pub target: Vec4F,
    pub up: Vec4F,

    // Rotation
    pub pitch: f32,
    pub yaw: f32
}

impl Camera {
    pub fn get_projection_matrix(&self) -> Mat4 {
        Mat4::project(self.fov, self.aspect_ratio, self.near, self.far)
    }

    // pub fn get_view_matrix(&self) -> Mat4 {

    // }
}