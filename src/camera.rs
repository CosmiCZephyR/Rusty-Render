use crate::math::{matrix4::Mat4, vector4f::Vec4F};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    // Projection
    pub near: f32,
    pub far: f32,
    pub fov: f32, // Can't be more than 180.
    pub aspect_ratio: f32,

    // View
    pub position: Vec4F,
    pub look_dir: Vec4F,
    pub target: Vec4F,
    pub up: Vec4F,

    // Rotation
    pub pitch: f32,
    pub yaw: f32,
}

impl Camera {
    pub fn new(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Camera {
        Camera {
            near,
            far,
            fov,
            aspect_ratio,
            position: Vec4F::default(),
            look_dir: Vec4F::default(),
            target: Vec4F {
                x: 0.0_f32,
                y: 0.0_f32,
                z: 1.0_f32,
                ..Vec4F::default()
            },
            up: Vec4F {
                x: 0.0_f32,
                y: 1.0_f32,
                z: 0.0_f32,
                ..Vec4F::default()
            },
            pitch: 0.0_f32,
            yaw: 0.0_f32,
        }
    }

    pub fn get_projection_matrix(&self) -> Mat4 {
        Mat4::project(self.fov, self.aspect_ratio, self.near, self.far)
    }

    pub fn get_view_matrix(&mut self) -> Mat4 {
        let mut mat_camera_rot = Mat4::default();
        mat_camera_rot = mat_camera_rot.rotate_y(self.yaw);
        self.look_dir = mat_camera_rot * self.target;
        let target = self.position + self.look_dir;

        let mat_camera = Mat4::point_at(self.position, target, self.up);

        let mat_view = mat_camera.quick_inverse();

        mat_view
    }
}

impl Default for Camera {
    fn default() -> Self {
        Camera::new(75.0_f32, 0.5625_f32, 0.05_f32, 4000.0_f32)
    }
}
