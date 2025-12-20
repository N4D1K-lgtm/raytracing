use crate::core::math::Vec2;
use crate::vec3::Vec3;

/// Texture trait for surface properties
pub trait Texture: Send + Sync {
    /// Evaluate texture at UV coordinates
    fn evaluate(&self, uv: Vec2) -> Vec3;

    /// Evaluate texture at 3D point (for 3D textures/procedural)
    fn evaluate_3d(&self, point: Vec3) -> Vec3 {
        // Default: ignore 3D position, use UV
        self.evaluate(Vec2::ZERO)
    }
}
