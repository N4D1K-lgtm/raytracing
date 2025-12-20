use super::{BxDF, BxDFSample, BxDFType};
use crate::core::math::Vec2;
use crate::vec3::Vec3;

/// Dielectric BSDF for glass/water (stub - will implement properly later)
pub struct DielectricBxDF {
    pub ior: f64, // Index of refraction
}

impl DielectricBxDF {
    pub fn new(ior: f64) -> Self {
        DielectricBxDF { ior }
    }
}

impl BxDF for DielectricBxDF {
    fn bxdf_type(&self) -> BxDFType {
        BxDFType::Specular
    }

    fn f(&self, _wo: Vec3, _wi: Vec3) -> Vec3 {
        // Delta distribution - not evaluated directly
        Vec3::ZERO
    }

    fn sample_f(&self, _wo: Vec3, _u: Vec2) -> Option<BxDFSample> {
        // TODO: Implement Fresnel and refraction
        None
    }

    fn pdf(&self, _wo: Vec3, _wi: Vec3) -> f64 {
        0.0 // Delta distribution
    }
}
