use super::Texture;
use crate::core::math::Vec2;
use crate::vec3::Vec3;

/// Checker pattern texture
pub struct CheckerTexture {
    pub color1: Vec3,
    pub color2: Vec3,
    pub scale: f64,
}

impl CheckerTexture {
    pub fn new(color1: Vec3, color2: Vec3, scale: f64) -> Self {
        CheckerTexture {
            color1,
            color2,
            scale,
        }
    }
}

impl Texture for CheckerTexture {
    fn evaluate(&self, uv: Vec2) -> Vec3 {
        let u = (uv.x * self.scale).floor() as i32;
        let v = (uv.y * self.scale).floor() as i32;

        if (u + v) % 2 == 0 {
            self.color1
        } else {
            self.color2
        }
    }

    fn evaluate_3d(&self, point: Vec3) -> Vec3 {
        let x = (point.x * self.scale).floor() as i32;
        let y = (point.y * self.scale).floor() as i32;
        let z = (point.z * self.scale).floor() as i32;

        if (x + y + z) % 2 == 0 {
            self.color1
        } else {
            self.color2
        }
    }
}
