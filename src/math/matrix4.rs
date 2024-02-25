use std::ops::Mul;
use std::default::Default;
use crate::math::vector4f::Vec4F;

#[derive(Clone, Copy, Debug)]
pub struct Mat4 {
    pub m: [[f32; 4]; 4]
}

impl Default for Mat4 {
    fn default() -> Self {
        Mat4 {
            m: [[0.0; 4]; 4]
        }
    }
}

impl Mat4 {
    pub fn make_identity() -> Mat4 {
        Mat4 {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0]
            ]
        }
    }

    pub fn rotate_x(&self, angle: f32) -> Mat4 {
        let mut mat = Mat4::default();
        mat.m[0][0] = 1.0;
        mat.m[1][1] = angle.cos();
        mat.m[1][2] = angle.sin();
        mat.m[2][1] = -angle.sin();
        mat.m[2][2] = angle.cos();
        mat.m[3][3] = 1.0;

        mat
    }

    pub fn rotate_y(&self, angle: f32) -> Mat4 {
        let mut mat = Mat4::default();
        mat.m[0][0] = angle.cos();
        mat.m[0][2] = angle.sin();
        mat.m[2][0] = -angle.sin();
        mat.m[1][1] = 1.0;
        mat.m[2][2] = angle.cos();
        mat.m[3][3] = 1.0;

        mat
    }

    pub fn rotate_z(&self, angle: f32) -> Mat4 {
        let mut mat = Mat4::default();
        mat.m[0][0] = angle.cos();
        mat.m[0][1] = angle.sin();
        mat.m[1][0] = -angle.sin();
        mat.m[1][1] = angle.cos();
        mat.m[2][2] = 1.0;
        mat.m[3][3] = 1.0;

        mat
    }

    pub fn translate(&self, x: f32, y: f32, z: f32) -> Mat4 {
        let mut mat = Mat4::default();
        mat.m[0][0] = 1.0;
        mat.m[1][1] = 1.0;
        mat.m[2][2] = 1.0;
        mat.m[3][3] = 1.0;
        mat.m[3][0] = x;
        mat.m[3][1] = y;
        mat.m[3][2] = z;

        mat
    }

    pub fn project(fov_deg: f32, aspect_ratio: f32, near: f32, far: f32) -> Mat4 {
        let fov_rad = 1.0 / (fov_deg * 0.5 / 180.0 * std::f32::consts::PI).tan();
        let mut mat: Mat4 = Mat4::default();

        mat.m[0][0] = aspect_ratio * fov_rad;
        mat.m[1][1] = fov_rad;
        mat.m[2][2] = far / (far - near);
        mat.m[3][2] = -(far * near) / (far - near);
        mat.m[2][3] = 1.0;
        mat.m[3][3] = 0.0;

        mat
    }

    pub fn point_at(pos: Vec4F, target: Vec4F, up: Vec4F) -> Mat4 {
        let new_forward = (target - pos).normalize();

        let a = new_forward * up.dot_product(&new_forward);
        let new_up = (up - a).normalize();

        let new_right = new_up.cross_product(&new_forward);

        let mut matrix = Mat4::default();
        matrix.m[0][0] = new_right.x;   matrix.m[0][1] = new_right.y;   matrix.m[0][2] = new_right.z;   matrix.m[0][3] = 1.0;
        matrix.m[1][0] = new_up.x;      matrix.m[1][1] = new_up.y;      matrix.m[1][2] = new_up.z;      matrix.m[1][3] = 1.0;
        matrix.m[2][0] = new_forward.x; matrix.m[2][1] = new_forward.y; matrix.m[2][2] = new_forward.z; matrix.m[2][3] = 1.0;
        matrix.m[3][0] = pos.x; matrix.m[3][1] = pos.y; matrix.m[3][2] = pos.z; matrix.m[3][3] = 1.0;

        matrix
    }

    pub fn quick_inverse(&self) -> Mat4 {
        let mut mat = Mat4::default();
        mat.m[0][0] = self.m[0][0]; mat.m[0][1] = self.m[1][0]; mat.m[0][2] = self.m[2][0]; mat.m[0][3] = 0.0;
        mat.m[1][0] = self.m[0][1]; mat.m[1][1] = self.m[1][1]; mat.m[1][2] = self.m[2][1]; mat.m[1][3] = 0.0;
        mat.m[2][0] = self.m[0][2]; mat.m[2][1] = self.m[1][2]; mat.m[2][2] = self.m[2][2]; mat.m[2][3] = 0.0;
        mat.m[3][0] = -(self.m[3][0] * mat.m[0][0] + self.m[3][1] * mat.m[1][0] + self.m[3][2] * mat.m[2][0]);
        mat.m[3][1] = -(self.m[3][0] * mat.m[0][1] + self.m[3][1] * mat.m[1][1] + self.m[3][2] * mat.m[2][1]);
        mat.m[3][2] = -(self.m[3][0] * mat.m[0][2] + self.m[3][1] * mat.m[1][2] + self.m[3][2] * mat.m[2][2]);
        mat.m[3][3] = 1.0;

        mat
    }
}

impl Mul for Mat4 {
    type Output = Mat4;
    fn mul(self, rhs: Self) -> Self::Output {
        let mut mat = Mat4::default();
        for i in 0..4 {
            for j in 0..4 {
                mat.m[j][i] = self.m[j][0] * rhs.m[0][i] + self.m[j][1] * rhs.m[1][i] + self.m[j][2] * rhs.m[2][i] + self.m[j][3] * rhs.m[3][i];
            }
        }

        mat
    }
}

impl Mul<Vec4F> for Mat4 {
    type Output = Vec4F;

    fn mul(self, i: Vec4F) -> Self::Output {
        Vec4F {
            x: i.x * self.m[0][0] + i.y * self.m[1][0] + i.z * self.m[2][0] + self.m[3][0] * i.w,
            y: i.x * self.m[0][1] + i.y * self.m[1][1] + i.z * self.m[2][1] + self.m[3][1] * i.w,
            z: i.x * self.m[0][2] + i.y * self.m[1][2] + i.z * self.m[2][2] + self.m[3][2] * i.w,
            w: i.x * self.m[0][3] + i.y * self.m[1][3] + i.z * self.m[2][3] + self.m[3][3] * i.w,
        }
    }
}