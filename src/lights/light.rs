use crate::core::math::Vec2;
use crate::vec3::Vec3;

/// Sample from a light source
pub struct LightSample {
    /// Position of the sampled point on the light
    pub position: Vec3,

    /// Normal at the sampled point (for area lights)
    pub normal: Vec3,

    /// Direction from shading point to light
    pub wi: Vec3,

    /// Radiance emitted towards the shading point
    pub radiance: Vec3,

    /// Probability density function value for this sample
    pub pdf: f64,

    /// Distance to the light
    pub distance: f64,
}

/// Light source trait
pub trait Light: Send + Sync {
    /// Sample the light from a given point
    ///
    /// # Arguments
    /// * `point` - The point being shaded
    /// * `u` - Random 2D sample for importance sampling
    ///
    /// # Returns
    /// * `Some(LightSample)` if the light is visible from the point
    /// * `None` if the light cannot contribute
    fn sample_li(&self, point: Vec3, u: Vec2) -> Option<LightSample>;

    /// Evaluate PDF for sampling a direction to the light
    fn pdf_li(&self, point: Vec3, wi: Vec3) -> f64;

    /// Get total power emitted by the light (for light selection)
    fn power(&self) -> f64;

    /// Check if this is a delta light (point, directional, etc.)
    /// Delta lights can only be sampled, not hit by rays
    fn is_delta(&self) -> bool {
        false
    }
}
