use std::fmt::Display;
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec4F {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32
}

impl Default for Vec4F {
    fn default() -> Self {
        Vec4F { x: 0.0_f32, y: 0.0_f32, z: 0.0_f32, w: 1.0_f32 }
    }
}

impl Add for Vec4F {
    type Output = Vec4F;
    fn add(self, rhs: Self) -> Self::Output {
        Vec4F {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            ..Vec4F::default()
        }
    }
}

impl Sub for Vec4F {
    type Output = Vec4F;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec4F {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            ..Vec4F::default()
        }
    }
}

impl Mul for Vec4F {
    type Output = Vec4F;
    fn mul(self, rhs: Self) -> Self::Output {
        Vec4F {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
            ..Vec4F::default()
        }
    }
}

impl Mul<f32> for Vec4F {
    type Output = Vec4F;
    fn mul(self, rhs: f32) -> Self::Output {
        Vec4F {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
            ..Vec4F::default()
        }
    }
}

impl Div for Vec4F {
    type Output = Vec4F;
    fn div(self, rhs: Self) -> Self::Output {
        Vec4F {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
            ..Vec4F::default()
        }
    }
}

impl Div<f32> for Vec4F {
    type Output = Vec4F;
    fn div(self, rhs: f32) -> Self::Output {
        Vec4F {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
            ..Vec4F::default()
        }
    }
}

impl AddAssign for Vec4F {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl SubAssign for Vec4F {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Display for Vec4F {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.3}, {:.3}, {:.3}, {:.3}", self.x, self.y, self.z, self.w)
    }
}

impl Vec4F {
    pub fn length(&self) -> f32 {
        (self.dot_product(&self)).sqrt()
    }

    pub fn normalize(&mut self) -> Vec4F {
        let l = self.length();
        Vec4F { x: self.x / l, y: self.y / l, z: self.z / l, ..Vec4F::default() }
    }

    pub fn dot_product(&self, other: &Vec4F) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross_product(&self, other: &Vec4F) -> Vec4F {
        Vec4F {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
            ..Vec4F::default()
        }
    }

    pub fn intersects_plane(plane_p: &Vec4F, plane_n: &mut Vec4F, line_start: &Vec4F, line_end: &Vec4F) -> Vec4F {
        *plane_n = plane_n.normalize();
        let pd = -plane_n.dot_product(&plane_p);
        let ad = line_start.dot_product(plane_n);
        let bd = line_end.dot_product(plane_n);
        let t = (-pd - ad) / (bd - ad);
        let line_start_to_end = *line_end - *line_start;
        let line_to_intersect = line_start_to_end * t;

        *line_start + line_to_intersect
    }
}