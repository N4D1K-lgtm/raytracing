use crate::core::math::Vec2;
use crate::vec3::Vec3;

/// BxDF type flags
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BxDFType {
    Reflection,
    Transmission,
    Diffuse,
    Glossy,
    Specular,
}

/// Sample from a BxDF
pub struct BxDFSample {
    /// Sampled direction (in local shading space)
    pub wi: Vec3,

    /// BxDF value for this direction
    pub f: Vec3,

    /// Probability density function value
    pub pdf: f64,

    /// Type of scattering
    pub bxdf_type: BxDFType,
}

/// Bidirectional scattering distribution function
/// Describes how light scatters at a surface point
pub trait BxDF: Send + Sync {
    /// Get the type of this BxDF
    fn bxdf_type(&self) -> BxDFType;

    /// Evaluate f(wo, wi) - the scattering function
    ///
    /// # Arguments
    /// * `wo` - Outgoing direction (towards viewer)
    /// * `wi` - Incoming direction (from light)
    ///
    /// Both directions are in local shading space (normal = +Z)
    fn f(&self, wo: Vec3, wi: Vec3) -> Vec3;

    /// Sample an incoming direction given an outgoing direction
    ///
    /// # Arguments
    /// * `wo` - Outgoing direction (towards viewer)
    /// * `u` - Random 2D sample for importance sampling
    ///
    /// # Returns
    /// * `Some(BxDFSample)` with sampled direction, f value, and PDF
    /// * `None` if sampling failed
    fn sample_f(&self, wo: Vec3, u: Vec2) -> Option<BxDFSample>;

    /// Evaluate PDF for sampling wi given wo
    fn pdf(&self, wo: Vec3, wi: Vec3) -> f64;

    /// Check if this BxDF matches a type
    fn matches_type(&self, t: BxDFType) -> bool {
        self.bxdf_type() == t
    }
}

/// Helper functions for shading calculations
pub mod shading {
    use crate::vec3::Vec3;

    /// Check if vector is in same hemisphere as normal (in local space, normal = +Z)
    #[inline]
    pub fn same_hemisphere(w: Vec3, wp: Vec3) -> bool {
        w.z * wp.z > 0.0
    }

    /// Cosine of angle between vector and normal (in local space)
    #[inline]
    pub fn cos_theta(w: Vec3) -> f64 {
        w.z
    }

    /// Absolute cosine of angle
    #[inline]
    pub fn abs_cos_theta(w: Vec3) -> f64 {
        w.z.abs()
    }

    /// Sine squared of angle
    #[inline]
    pub fn sin2_theta(w: Vec3) -> f64 {
        (1.0 - cos_theta(w) * cos_theta(w)).max(0.0)
    }

    /// Sine of angle
    #[inline]
    pub fn sin_theta(w: Vec3) -> f64 {
        sin2_theta(w).sqrt()
    }

    /// Tangent of angle
    #[inline]
    pub fn tan_theta(w: Vec3) -> f64 {
        sin_theta(w) / cos_theta(w)
    }

    /// Cosine of phi angle (azimuthal)
    #[inline]
    pub fn cos_phi(w: Vec3) -> f64 {
        let sin_t = sin_theta(w);
        if sin_t == 0.0 {
            1.0
        } else {
            (w.x / sin_t).clamp(-1.0, 1.0)
        }
    }

    /// Sine of phi angle (azimuthal)
    #[inline]
    pub fn sin_phi(w: Vec3) -> f64 {
        let sin_t = sin_theta(w);
        if sin_t == 0.0 {
            0.0
        } else {
            (w.y / sin_t).clamp(-1.0, 1.0)
        }
    }
}
