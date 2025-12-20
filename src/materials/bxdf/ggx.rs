use super::{BxDF, BxDFSample, BxDFType};
use crate::core::math::Vec2;
use crate::vec3::Vec3;

/// GGX microfacet BRDF (stub - will implement properly later)
pub struct GgxBxDF {
    pub albedo: Vec3,
    pub roughness: f64,
}

impl GgxBxDF {
    pub fn new(albedo: Vec3, roughness: f64) -> Self {
        GgxBxDF {
            albedo,
            roughness: roughness.max(0.01), // Clamp to avoid singularities
        }
    }
}

impl BxDF for GgxBxDF {
    fn bxdf_type(&self) -> BxDFType {
        BxDFType::Glossy
    }

    fn f(&self, _wo: Vec3, _wi: Vec3) -> Vec3 {
        // TODO: Implement GGX microfacet model
        Vec3::ZERO
    }

    fn sample_f(&self, _wo: Vec3, _u: Vec2) -> Option<BxDFSample> {
        // TODO: Implement GGX importance sampling
        None
    }

    fn pdf(&self, _wo: Vec3, _wi: Vec3) -> f64 {
        0.0
    }
}
