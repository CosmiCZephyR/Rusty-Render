use std::fmt::Display;

pub struct Vec3F {
    pub u: f32,
    pub v: f32,
    pub w: f32
}

impl Default for Vec3F {
    fn default() -> Self {
        Vec3F { u: 0.0, v: 0.0, w: 1.0 }
    }
}

impl Display for Vec3F {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}, {}", self.u, self.v, self.w)
    }
}