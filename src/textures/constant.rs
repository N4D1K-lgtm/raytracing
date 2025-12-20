use super::Texture;
use crate::core::math::Vec2;
use crate::vec3::Vec3;

/// Constant color texture
pub struct ConstantTexture {
    pub color: Vec3,
}

impl ConstantTexture {
    pub fn new(color: Vec3) -> Self {
        ConstantTexture { color }
    }
}

impl Texture for ConstantTexture {
    fn evaluate(&self, _uv: Vec2) -> Vec3 {
        self.color
    }

    fn evaluate_3d(&self, _point: Vec3) -> Vec3 {
        self.color
    }
}
