use super::{BxDF, BxDFSample, BxDFType};
use crate::core::math::Vec2;
use crate::vec3::Vec3;
use std::f64::consts::PI;

/// Lambertian (perfectly diffuse) BRDF
pub struct LambertianBxDF {
    /// Albedo (surface color)
    pub albedo: Vec3,
}

impl LambertianBxDF {
    pub fn new(albedo: Vec3) -> Self {
        LambertianBxDF { albedo }
    }
}

impl BxDF for LambertianBxDF {
    fn bxdf_type(&self) -> BxDFType {
        BxDFType::Diffuse
    }

    fn f(&self, _wo: Vec3, wi: Vec3) -> Vec3 {
        // Lambertian BRDF = albedo / π
        // The cosine term is handled separately in the rendering equation
        if wi.z > 0.0 {
            self.albedo / PI
        } else {
            Vec3::ZERO
        }
    }

    fn sample_f(&self, _wo: Vec3, u: Vec2) -> Option<BxDFSample> {
        // Cosine-weighted hemisphere sampling
        let wi = Self::cosine_sample_hemisphere(u);
        let pdf = wi.z / PI; // cos(theta) / π

        Some(BxDFSample {
            wi,
            f: self.albedo / PI,
            pdf,
            bxdf_type: BxDFType::Diffuse,
        })
    }

    fn pdf(&self, _wo: Vec3, wi: Vec3) -> f64 {
        if wi.z > 0.0 {
            wi.z / PI
        } else {
            0.0
        }
    }
}

impl LambertianBxDF {
    /// Cosine-weighted hemisphere sampling
    fn cosine_sample_hemisphere(u: Vec2) -> Vec3 {
        let r = u.x.sqrt();
        let phi = 2.0 * PI * u.y;

        let x = r * phi.cos();
        let y = r * phi.sin();
        let z = (1.0 - u.x).max(0.0).sqrt();

        Vec3::new(x, y, z)
    }
}
